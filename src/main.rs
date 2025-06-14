use crate::{
    bytes::Size,
    config::Config,
    dir::{DetailedEntry, FileKind},
    git::{GitCache, GitStatus},
    table::{Alignment, Table},
    utils::format_template,
    walk::{SyncWalk, ThreadedWalk},
};
use clap::{Parser, Subcommand};
use colored::Colorize;
use std::{
    cmp::Ordering, collections::HashMap, hash::Hash, path::PathBuf, sync::LazyLock, time::Instant,
};

fn default_format() -> Vec<&'static str> {
    Vec::from([
        "{depth:<2} {icon} {name}",
        "{persmissions}",
        "{size}",
        "{last_modified}",
        "{git_status}",
        "{nlink}",
        "{link_target}",
    ])
}

mod bytes;
mod config;
mod dir;
mod git;
mod table;
mod utils;
mod walk;

#[derive(Debug, Subcommand)]
pub enum Command {
    /// This is the default command, it will trigger if no subcommand is provided.
    /// Its args are defined in `Args`.
    List,

    /// Finds a file or directory by name.
    Find {
        /// The name of the file or directory to search for.
        #[clap(index = 1)]
        name: String,

        /// The root path to start the search from.
        /// Defaults to the current directory.
        #[clap(default_value = ".", index = 2)]
        path: PathBuf,

        /// Show all files, including hidden ones.
        /// Defaults to `false`.
        #[clap(short, long, default_value = "false")]
        all: bool,
    },
}

#[derive(Debug, Parser)]
struct Args {
    /// Path to walk
    #[clap(default_value = ".", index = 1)]
    path: PathBuf,

    // The maximum depth to walk
    #[clap(short, long, default_value = "1")]
    depth: usize,

    /// Show all files, including hidden ones
    #[clap(short, long, default_value = "false")]
    all: bool,

    /// The command to run
    /// If not provided, the default command is `List`.
    #[clap(subcommand)]
    command: Option<Command>,
}

fn main() {
    // let args = Args::parse();
    // let config = Config::parse();

    // match args.command {
    //     Some(Command::Find { name, path, all }) => find(name, path, all, &config),
    //     _ => ls(&args, &config),
    // }

    let template = "Hello i am {{name}}!!";

    let values = HashMap::from([("name", "Saverio")]);

    println!("{}", format_template(template, &values));
}

fn ls(args: &Args, conf: &Config) {
    let mut table = Table::new().padding(conf.ls.padding);
    let git_cache = GitCache::new(&args.path);

    for (entry, depth) in SyncWalk::new(&args.path)
        .sort_by(|a, b| {
            let is_dir_a = a.file_type().map_or(false, |t| t.is_dir());
            let is_dir_b = b.file_type().map_or(false, |t| t.is_dir());

            match (is_dir_a, is_dir_b) {
                (true, false) => Ordering::Less,
                (false, true) => Ordering::Greater,
                _ => a.file_name().cmp(&b.file_name()),
            }
        })
        .skip_hidden(!args.all)
        .max_depth(args.depth)
        .follow_symlinks(false)
        .map(|(e, d)| (DetailedEntry::from(e), d))
    {
        let mut values = HashMap::new();

        values.insert("depth", &format!("{:>2}", depth));
        // Create the row array for the table
        // This will hold the formatted strings for each column
        let mut row = Vec::new();

        // First, format the name part, which by default includes the depth padding,
        // the icon, name padding (if doesnt start with a dot), and the name itself.
        // -1 because the depth start at 1
        let depth_padding = " ".repeat(depth - 1);
        let name = entry.name();

        let icon = match entry.kind() {
            FileKind::File => conf.indicators.file(&entry.extension().unwrap_or(name)),
            FileKind::Directory => conf.indicators.dir(&name),
            _ => conf.indicators.unknown(),
        };

        // If the name starts with a dot, we don't add padding,
        // otherwise we add a space after the icon.
        // This is to ensure that the first alphanumeric character stays aligned.
        let name_padding = match (args.all, name.starts_with('.')) {
            (true, false) => " ",
            _ => "",
        };

        let name = format!("{}{} {}{}", depth_padding, icon, name_padding, name);
        let name = match entry.kind() {
            FileKind::Directory => name.bright_blue().bold(),
            FileKind::File if entry.executable() => name.green().bold(),
            FileKind::File => name.white(),
            FileKind::Symlink => name.yellow(),
            _ => name.white(),
        };

        row.push((name.to_string(), Alignment::Left));

        let permissions = entry
            .permissions()
            .custom_color((128, 128, 128))
            .to_string();

        row.push((permissions, Alignment::Center));

        let size = Size(entry.size()).to_string();

        row.push((size, Alignment::Right));

        let last_modified = entry
            .timestamp()
            .map_or_else(
                || "N/A".to_string(),
                |ts| ts.format(&conf.ls.time_format).to_string(),
            )
            .custom_color((128, 128, 128))
            .to_string();

        row.push((last_modified, Alignment::Center));

        if let Some(ref cache) = git_cache {
            if let Some(status) = cache.get_status(&entry.path()) {
                let status_indicator = match status {
                    GitStatus::Untracked => "U".green().bold(),
                    GitStatus::Modified => "M".yellow(),
                    GitStatus::Deleted => "D".red().bold(),
                    GitStatus::Renamed => "R".blue(),
                    GitStatus::Ignored => "I".custom_color((128, 128, 128)),
                    GitStatus::Conflict => "C".magenta().bold(),
                    GitStatus::Clean => "-".to_string().white(),
                };

                row.push((status_indicator.to_string(), Alignment::Center));
            } else {
                row.push(("".to_string(), Alignment::Center));
            }
        }

        let nlink = format!("{} 󱞫", entry.nlink());

        row.push((nlink, Alignment::Right));

        if let Some(target) = entry.link_target() {
            row.push((
                format!("󱦰 {}", target.display())
                    .yellow()
                    .bold()
                    .to_string(),
                Alignment::Left,
            ));
        } else {
            row.push(("".to_string(), Alignment::Left));
        }

        table.add_row(row);
    }

    println!("total: {}", table.rows().len());
    println!("{}", table);
}

fn find(name: String, path: PathBuf, all: bool, _conf: &Config) {
    let t0 = Instant::now();
    let mut c = 0;

    for entry in ThreadedWalk::new(&path).skip_hidden(!all) {
        if entry
            .file_name()
            .map(|os_str| os_str.to_string_lossy() == name)
            .unwrap_or(false)
        {
            c += 1;
            println!("{}", entry.as_path().display());
        }
    }

    println!("Found {} entries in {:?}", c, t0.elapsed());
}
