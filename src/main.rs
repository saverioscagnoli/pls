use jwalk::{DirEntry, WalkDirGeneric};
use std::{
    path::{Path, PathBuf},
    time::Instant,
};

use crate::tree::SyncWalk;
use clap::{Parser, Subcommand};

mod tree;

#[derive(Debug, Subcommand)]
enum Command {
    /// List directory contents (default)
    List {
        #[arg(default_value = ".")]
        path: PathBuf,
        #[arg(short, long, default_value_t = false)]
        all: bool,
    },
    /// Show directory size
    Size {
        #[arg(default_value = ".")]
        path: PathBuf,
        #[arg(short, long, default_value_t = false)]
        all: bool,
    },
}

impl Default for Command {
    fn default() -> Self {
        Command::List {
            path: PathBuf::from("."),
            all: false,
        }
    }
}

#[derive(Parser, Debug)]
#[command(arg_required_else_help = false)]
struct Args {
    #[command(subcommand)]
    command: Option<Command>,
}

fn main() {
    let args = Args::parse();

    match args.command.unwrap_or_default() {
        Command::List { path, all } => {
            println!("Listing contents of: {:?}", path);
            for entry in SyncWalk::new(&path).skip_hidden(!all).max_depth(2) {
                println!("{:?}", entry.path);
            }
        }

        Command::Size { path, all } => {}
    }
}
