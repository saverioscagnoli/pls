use crate::{
    bytes::Size,
    config::Config,
    dir::{DetailedEntry, FileKind},
    git::{GitCache, GitStatus},
    table::Table,
    walk::SyncWalk,
};
use clap::Parser;
use colored::Colorize;
use std::path::PathBuf;

mod bytes;
mod config;
mod dir;
mod git;
mod table;
mod utils;
mod walk;

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
}

fn main() {
    let args = Args::parse();
    let config = Config::parse();
    let mut table = Table::new().padding(2);

    let git_cache = GitCache::new(&args.path);

    for (entry, depth) in SyncWalk::new(&args.path)
        .sort_by(|a, b| {
            // Directories first
            let a_is_dir = a.file_type().map(|ft| ft.is_dir()).unwrap_or(false);
            let b_is_dir = b.file_type().map(|ft| ft.is_dir()).unwrap_or(false);

            match (a_is_dir, b_is_dir) {
                (true, false) => std::cmp::Ordering::Less,
                (false, true) => std::cmp::Ordering::Greater,
                _ => a
                    .file_name()
                    .to_string_lossy()
                    .to_string()
                    .cmp(&b.file_name().to_string_lossy().to_string()),
            }
        })
        .max_depth(args.depth)
        .follow_symlinks(false)
        .skip_hidden(!args.all)
        .map(|(e, d)| (DetailedEntry::from(e), d))
    {
        let depth_padding = " ".repeat(depth - 1);
        let name = match entry.kind() {
            FileKind::File if entry.executable() => entry.name().green().bold(),
            FileKind::File => entry.name().white(),
            FileKind::Directory => entry.name().bright_blue().bold(),
            FileKind::Symlink => entry.name().yellow(),
            _ => entry.name().white(),
        };

        let name_padding = match name.starts_with('.') {
            true => "",
            false => " ",
        };

        let icon = match entry.kind() {
            FileKind::File => config.file_icon(entry.name()),
            FileKind::Directory => config.dir_icon(entry.name()),
            _ => config.unknown_icon(),
        };

        let name = format!("{}{} {}{}", depth_padding, &icon, name_padding, name);

        let timestamp = entry
            .timestamp()
            .map_or_else(|| "N/A".to_string(), |ts| ts.format("%D %H:%M").to_string())
            .custom_color((128, 128, 128));

        // Convert absolute path to relative path from git workdir
        let git_status = git_cache.as_ref().map_or(None, |c| {
            match c.get_status(&entry.path()).unwrap_or(&GitStatus::Clean) {
                GitStatus::Untracked => Some("U".green().bold()),
                GitStatus::Modified => Some("M".yellow()),
                GitStatus::Deleted => Some("D".red().bold()),
                GitStatus::Renamed => Some("R".blue()),
                GitStatus::Ignored => Some("I".custom_color((128, 128, 128))),
                GitStatus::Conflict => Some("C".magenta().bold()),
                GitStatus::Clean => Some(" ".to_string().white()),
            }
        });

        let mut rows = Vec::new();

        rows.push(name);
        rows.push(
            entry
                .permissions()
                .custom_color((128, 128, 128))
                .to_string(),
        );
        rows.push(Size(entry.size()).to_string());
        rows.push(timestamp.to_string());

        // Dont care about the aligment here,
        // since if the status, is None, it will be for all files.
        // while if it is Some, it will be for all files that are tracked by git.
        if let Some(status) = git_status {
            rows.push(status.to_string());
        }

        rows.push(format!("󱞫 {}", entry.nlink()));

        if let Some(target) = entry.link_target() {
            rows.push(target.to_string_lossy().yellow().to_string());
        }

        table.add_row(rows);
    }

    println!("total: {}", table.rows().len());
    println!("{}", table);
}
