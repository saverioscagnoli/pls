mod commands;
mod config;
mod table;
mod util;
mod walk;

use crate::config::Config;
use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Debug, Clone, Parser)]
pub struct FindArgs {
    #[arg(index = 1)]
    pattern: String,

    #[arg(index = 2, default_value = ".")]
    root: PathBuf,

    #[arg(short, long, default_value_t = false)]
    all: bool,

    #[arg(short, long, default_value_t = usize::MAX)]
    depth: usize,

    #[arg(short, long, default_value_t = false)]
    follow_symlinks: bool,

    #[arg(short, long, default_value_t = false)]
    exact: bool,

    #[arg(short, long, default_value_t = false)]
    timed: bool,
}

#[derive(Debug, Clone, Subcommand)]
enum Command {
    Find(FindArgs),
}

#[derive(Debug, Clone, Parser)]
struct Args {
    #[arg(index = 1, default_value = ".")]
    path: PathBuf,

    #[arg(short, long, default_value_t = false)]
    all: bool,

    #[arg(short, long, default_value_t = 1)]
    depth: usize,

    #[arg(short, long, default_value_t = false)]
    follow_symlinks: bool,

    #[arg(short, long, default_value_t = false)]
    pad_names: bool,

    #[command(subcommand)]
    subcommand: Option<Command>,
}

fn main() {
    let args = Args::parse();
    let config = match Config::parse() {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Error parsing config: {}", e);
            return;
        }
    };

    match args.subcommand {
        Some(Command::Find(args)) => commands::find::execute(&args),
        _ => commands::list::execute(&args, &config.ls),
    }
}
