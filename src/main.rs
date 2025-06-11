use crate::{
    config::Config,
    dir::{DetailedEntry, FileKind},
    table::Table,
    walk::SyncWalk,
};
use clap::Parser;
use std::path::PathBuf;

mod config;
mod dir;
mod table;
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

    for (entry, depth) in SyncWalk::new(&args.path)
        .max_depth(args.depth)
        .skip_hidden(!args.all)
        .map(|(e, d)| (DetailedEntry::from(e), d))
    {
        let depth_padding = " ".repeat(depth);

        let name_padding = match entry.name().starts_with('.') {
            true => "",
            false => " ",
        };

        let icon = match entry.kind() {
            FileKind::File => config.file_icon(entry.name()),
            FileKind::Directory => config.dir_icon(entry.name()),
            _ => config.unknown_icon(),
        };

        let name = format!("{}{} {}{}", depth_padding, icon, name_padding, entry.name());

        table.add_row(vec![name]);
    }

    println!("total: {}", table.rows().len());
    println!("{}", table);
}
