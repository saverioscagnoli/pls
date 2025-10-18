mod args;
mod commands;
mod config;
mod err;
mod util;

use std::{collections::HashSet, f32::consts::E, path::PathBuf, str::FromStr, sync::LazyLock};

use crate::{
    args::{Args, Subcommand},
    config::Config,
    err::PlsError,
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
