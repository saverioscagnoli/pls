use clap::{Parser, Subcommand as ClapSubcommand};
use std::path::PathBuf;

#[derive(Debug, Clone, Parser)]
pub struct ListArgs {
    #[arg(index = 1, default_value = ".")]
    pub path: PathBuf,

    #[arg(short, long, default_value_t = false)]
    pub all: bool,
}

#[derive(Debug, Clone, Parser)]
pub struct FindArgs {
    #[arg(index = 1)]
    pub pattern: String,

    #[arg(index = 2, default_value = ".")]
    pub root: PathBuf,
}

#[derive(Debug, Clone, ClapSubcommand)]
pub enum Subcommand {
    Find(FindArgs),
}

#[derive(Debug, Parser)]
pub struct Args {
    #[clap(flatten)]
    pub list: ListArgs,

    #[command(subcommand)]
    pub subcommand: Option<Subcommand>,
}
