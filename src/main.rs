mod bytes;
mod config;
mod dir;
mod error;
mod git;
mod table;
mod utils;
mod walk;

use crate::{
    config::Config,
    dir::{DetailedEntry, FileKind},
    git::GitCache,
    table::Table,
    walk::{SyncWalk, ThreadedWalk},
};

use clap::{Parser, Subcommand};
use figura::{DefaultParser, Template, Value};
use std::{cmp::Ordering, collections::HashMap, path::PathBuf, time::Instant};

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

type Walker<T> = Box<dyn Iterator<Item = T>>;

fn main() {
    let args = Args::parse();
    let config = Config::parse();

    match args.command {
        Some(Command::Find { name, path, all }) => find(name, path, all, &config),
        _ => {
            // If the user sets the max depth to >= 3, it makes more sense to use a multithreaded walker
            // to speed up the process
            // If not, use a single-threaded walker
            let walker: Box<dyn Iterator<Item = (DetailedEntry, usize)>> = match args.depth {
                d if d < 3 => Box::new(
                    SyncWalk::new(&args.path)
                        .skip_hidden(!args.all)
                        .max_depth(args.depth)
                        .sort_by(|a, b| {
                            let is_dir_a = a.file_type().map_or(false, |t| t.is_dir());
                            let is_dir_b = b.file_type().map_or(false, |t| t.is_dir());

                            match (is_dir_a, is_dir_b) {
                                (true, false) => Ordering::Less,
                                (false, true) => Ordering::Greater,
                                _ => a.file_name().cmp(&b.file_name()),
                            }
                        })
                        .map(|(entry, path)| {
                            let detailed_entry = DetailedEntry::from(entry);
                            (detailed_entry, path)
                        }),
                ),

                _ => Box::new(
                    ThreadedWalk::new(&args.path)
                        .skip_hidden(!args.all)
                        .max_depth(args.depth)
                        .map(|(path, depth)| {
                            let detailed_entry = DetailedEntry::from(path.as_path());
                            (detailed_entry, depth)
                        }),
                ),
            };

            ls(&args, &config, walker);
        }
    }
}

/// This is the command that lists the files and directories.
/// It's the default behavior if no subcommand is provided
fn ls(args: &Args, conf: &Config, walker: Walker<(DetailedEntry, usize)>) {
    // Table for pretty printing the output
    let mut table: Table<String> = Table::new().padding(conf.ls.padding);

    // If `args.path` is not a git repository, the default git cache will be empty.
    // see `GitCache::new`
    let git_cache = GitCache::new(&args.path).unwrap_or_default();

    let templates = conf
        .ls
        .format
        .iter()
        .map(|t| Template::<'{', '}'>::parse(&t))
        .filter_map(Result::ok)
        .collect::<Vec<_>>();

    for (entry, depth) in walker {
        let mut row = Vec::new();
        let mut context = HashMap::new();

        let name = entry.name();
        let ext = entry.ext().unwrap_or("");

        let icon = match entry.kind() {
            FileKind::Directory => conf.indicators.dir(&name),
            FileKind::File => conf.indicators.file(&ext),
            _ => conf.indicators.unknown(),
        };

        context.insert("depth", Value::Int(depth as i64 - 1));
        context.insert("icon", Value::String(icon));
        context.insert("name", Value::String(entry.name().to_string()));
        context.insert(
            "permissions",
            Value::String(entry.permissions().to_string()),
        );

        for t in &templates {
            match t.format(&context) {
                Ok(v) => row.push((v, t.alignment())),
                Err(e) => eprintln!("{}", e),
            }
        }

        // Skip empty rows
        if row.is_empty() {
            continue;
        }

        table.add_row(row);
    }

    println!("total: {}", table.rows().len());
    println!("{}", table);
}

fn find(name: String, path: PathBuf, all: bool, _conf: &Config) {
    let t0 = Instant::now();
    let mut c = 0;

    for (path, _) in ThreadedWalk::new(&path).skip_hidden(!all) {
        if path
            .file_name()
            .map(|os_str| os_str.to_string_lossy() == name)
            .unwrap_or(false)
        {
            c += 1;
            println!("{}", path.as_path().display());
        }
    }

    println!("Found {} entries in {:?}", c, t0.elapsed());
}
