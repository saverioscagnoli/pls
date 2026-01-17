use std::{fmt, path::PathBuf};

#[derive(Debug, Clone, Copy)]
pub enum CommandWithConfig {
    List,
    Find,
}

impl fmt::Display for CommandWithConfig {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            CommandWithConfig::List => write!(f, "list"),
            CommandWithConfig::Find => write!(f, "find"),
        }
    }
}

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

/// The default behavior of the cli tool.
///
/// This will be flattened into the general args
#[derive(Debug, Clone)]
#[derive(clap::Parser)]
pub struct LsArgs {
    #[clap(index = 1, default_value = ".")]
    pub path: PathBuf,

    #[clap(short, long, default_value_t = false)]
    pub all: bool,

    #[clap(short, long, default_value_t = 1)]
    pub depth: usize,
}

impl Default for LsArgs {
    fn default() -> Self {
        Self {
            path: PathBuf::from("."),
            all: false,
            depth: 1,
        }
    }
}

#[derive(Debug, Clone)]
#[derive(clap::Subcommand)]
pub enum ConfigCommand {
    /// Resets the config to a default state
    Reset(ConfigResetArgs),

    /// Outputs the current config
    Show(ConfigShowArgs),
}

#[derive(Debug, Clone)]
#[derive(clap::Parser)]
pub struct ConfigArgs {
    #[clap(subcommand)]
    pub subcommand: ConfigCommand,
}

#[derive(Debug, Clone)]
#[derive(clap::Parser)]
pub struct ConfigResetArgs {
    #[clap(short, long, default_value_t = false)]
    pub yes: bool,

    #[clap(short, long, default_value_t = false, group = "action")]
    pub list: bool,

    #[clap(short, long, default_value_t = false, group = "action")]
    pub find: bool,
}

#[derive(Debug, Clone)]
#[derive(clap::Parser)]
pub struct ConfigShowArgs {
    #[clap(short, long, default_value_t = false)]
    pub list: bool,

    #[clap(short, long, default_value_t = false, group = "action")]
    pub find: bool,
}

#[derive(Debug, Clone)]
#[derive(clap::Parser)]
pub struct FindArgs {}
