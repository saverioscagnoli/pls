mod args;
mod commands;
mod config;
mod err;
mod table;
mod util;
mod walk;

use crate::{
    args::{Args, Subcommand},
    config::Config,
};
use clap::Parser;

fn main() {
    let args = Args::parse();
    let config = Config::parse().unwrap_or_else(|e| {
        eprintln!("Warning: {}. Using default configuration.", e);
        Config::default()
    });

    match args.subcommand {
        Some(Subcommand::Find(args)) => {
            println!("Finding pattern: {} in path: {:?}", args.pattern, args.root);
        }

        None => {
            // Default to list behavior
            if let Err(e) = commands::list::execute(&args.list, &config.ls) {
                eprintln!("{}", e);
            }
        }
    }
}
