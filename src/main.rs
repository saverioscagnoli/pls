#![warn(clippy::use_self)]

mod args;
mod commands;
mod config;
mod consts;
mod table;
mod util;
mod walk;

use crate::{
    args::{Args, Command, ConfigCommand},
    config::{LsConfig, Parse},
};
use clap::Parser;

fn main() {
    let args = Args::parse();

    match args.command {
        Some(Command::Config(config)) => match config.subcommand {
            ConfigCommand::Reset(reset) => commands::config::reset(reset),
            ConfigCommand::Show(show) => commands::config::show(show),
        },
        Some(Command::Find(find)) => {}
        None => commands::list::exec(args.list, LsConfig::parse()),
    }
}
