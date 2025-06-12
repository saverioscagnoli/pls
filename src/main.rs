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
        .max_depth(args.depth)
        .skip_hidden(!args.all)
        .map(|(e, d)| (DetailedEntry::from(e), d))
    {
        let depth_padding = " ".repeat(depth - 1);
        let name_padding = match entry.name().starts_with('.') {
            true => "",
            false => " ",
        };

        let icon = match entry.kind() {
            FileKind::File => config.file_icon(entry.name()),
            FileKind::Directory => config.dir_icon(entry.name()),
            _ => config.unknown_icon(),
        };

        let name = format!(
            "{}{} {}{}",
            depth_padding,
            console::strip_ansi_codes(&icon),
            name_padding,
            entry.name()
        );

        let timestamp = entry
            .timestamp()
            .map_or_else(|| "N/A".to_string(), |ts| ts.format("%D %H:%M").to_string())
            .custom_color((128, 128, 128));

        // Convert absolute path to relative path from git workdir
        let git_status = match git_cache.get_status(&entry.path()) {
            Some(GitStatus::Untracked) => "U".green(),
            Some(GitStatus::Modified) => "M".yellow(),
            Some(GitStatus::Deleted) => "D".red(),
            Some(GitStatus::Renamed) => "R".yellow(),
            Some(GitStatus::Ignored) => "I".custom_color((128, 128, 128)),
            Some(GitStatus::Conflict) => "C".bright_yellow(),
            Some(GitStatus::Clean) => "-".custom_color((128, 128, 128)),
            None => " ".white(),
        };

        table.add_row(vec![
            name,
            entry.permissions().to_string(),
            Size(entry.size()).to_string(),
            timestamp.to_string(),
            git_status.to_string(),
        ]);
    }

    println!("total: {}", table.rows().len());
    println!("{}", table);
}
