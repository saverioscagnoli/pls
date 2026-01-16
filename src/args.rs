use std::path::PathBuf;

#[derive(Debug, Clone)]
#[derive(clap::Subcommand)]
pub enum SubCommand {
    Find(FindArgs),
}

#[derive(Debug, Clone)]
#[derive(clap::Parser)]
pub struct Args {
    #[clap(flatten)]
    pub list: ListArgs,

    #[clap(subcommand)]
    pub subcommand: Option<SubCommand>,
}

/// The default behavior of the cli tool.
///
/// This will be flattened into the general args
#[derive(Debug, Clone)]
#[derive(clap::Parser)]
pub struct ListArgs {
    #[clap(index = 1, default_value = ".")]
    pub path: PathBuf,

    #[clap(short, long, default_value_t = false)]
    pub all: bool,

    #[clap(short, long, default_value_t = 1)]
    pub depth: u32,
}

#[derive(Debug, Clone)]
#[derive(clap::Parser)]
pub struct FindArgs {}
