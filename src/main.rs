mod commands;
mod config;
mod style;
mod table;
mod util;
mod walk;

use crate::config::Config;
use clap::Parser;
use std::path::PathBuf;

#[derive(Debug, Clone, Parser)]
struct Args {
    #[arg(index = 1, default_value = ".")]
    path: PathBuf,

    #[arg(short, long, default_value_t = false)]
    all: bool,

    #[arg(short, long, default_value_t = 1)]
    depth: usize,
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

    commands::list::execute(&args, &config.ls);
}
