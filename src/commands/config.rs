use crate::{
    config::{ConfigKind, LsConfig, Parse},
    consts, util,
};
use std::fs;

#[derive(Debug, Clone)]
#[derive(clap::Subcommand)]
pub enum ConfigCommand {
    Reset(ConfigResetArgs),
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

fn get_selected_configs(list: bool, find: bool) -> Vec<ConfigKind> {
    let mut configs = Vec::new();

    if list {
        configs.push(ConfigKind::List);
    }

    if find {
        configs.push(ConfigKind::Find);
    }

    configs
}

fn rewrite_config(kinds: &[ConfigKind]) {
    for kind in kinds {
        let Some(path) = util::config_path(*kind) else {
            eprintln!(
                "Failed to reset config for command '{}', config does not exist",
                kind
            );

            continue;
        };

        match kind {
            ConfigKind::List => {
                let config = LsConfig::default();
                let content = toml::to_string_pretty(&config).unwrap();

                if let Err(e) = fs::write(&path, content) {
                    eprintln!("Failed to write config file: {}", e);
                    continue;
                }

                println!("Successfully reset config at '{}'.", path.display());
            }
            _ => todo!(),
        }
    }
}

fn print_config(kind: ConfigKind) {
    match kind {
        ConfigKind::List => {
            let config = LsConfig::parse();
            let content = toml::to_string_pretty(&config).unwrap();

            if let Some(path) = util::config_path(kind) {
                println!("path: {}", path.display());
                println!();
            }

            println!("{}", content);
        }
        ConfigKind::Find => todo!(),
    }
}

fn confirm_reset() -> bool {
    match util::prompt(consts::RESET_CONFIG_PROMPT) {
        Ok(response) => matches!(response.to_lowercase().as_str(), "y" | "yes"),
        Err(_) => {
            eprintln!("I/O Error while prompting the user. Aborting.");
            false
        }
    }
}

pub fn reset(args: ConfigResetArgs) {
    let configs = get_selected_configs(args.list, args.find);

    if configs.is_empty() {
        eprintln!("You must select at least one configuration file with '--list' or '--find'");
        return;
    }

    if args.yes || confirm_reset() {
        rewrite_config(&configs);
    } else {
        println!("Aborting reset.");
    }
}

pub fn show(args: ConfigShowArgs) {
    let configs = get_selected_configs(args.list, args.find);

    match configs.len() {
        0 => eprintln!("You must select one configuration file with '--list' or '--find'"),
        1 => print_config(configs[0]),
        _ => eprintln!("You must select at most one configuration file with '--list' or '--find'"),
    }
}
