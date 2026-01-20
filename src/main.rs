#![warn(clippy::use_self)]

mod commands;
mod config;
mod consts;
mod table;
mod util;
mod walk;

use crate::{
    commands::{
        config::{ConfigArgs, ConfigCommand},
        find::FindArgs,
        list::LsArgs,
    },
    config::{LsConfig, Parse},
};
use clap::Parser;

#[derive(Debug, Clone)]
#[derive(clap::Subcommand)]
pub enum Command {
    Find(FindArgs),
    Config(ConfigArgs),
}

#[derive(Debug, Clone)]
#[derive(clap::Parser)]
pub struct Args {
    #[clap(flatten)]
    pub list: LsArgs,

    #[clap(subcommand)]
    pub command: Option<Command>,
}

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
