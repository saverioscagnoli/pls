use std::{fs, io};

use crate::{
    args::{CommandWithConfig, ConfigResetArgs, ConfigShowArgs},
    config::{LsConfig, Parse},
    consts, util,
};

fn rewrite_config(commands: &[CommandWithConfig]) {
    for command in commands {
        let Some(path) = util::config_path(*command) else {
            eprintln!(
                "Failed to reset config for command '{}', config does not exist",
                command
            );
            return;
        };

        match command {
            CommandWithConfig::List => {
                let config = LsConfig::default();
                let content = toml::to_string_pretty(&config).unwrap();

                if let Err(e) = fs::write(&path, content) {
                    eprintln!("Failed to write config file: {}", e);
                    return;
                }

                println!("Successfully reset config at '{}'.", path.display());
            }
            _ => todo!(),
        }
    }
}

pub fn print_config(command: CommandWithConfig) {
    match command {
        CommandWithConfig::List => {
            let config = LsConfig::parse();
            let content = toml::to_string_pretty(&config).unwrap();

            if let Some(path) = util::config_path(command) {
                println!("path: {}", path.display());
                println!();
            }

            println!("{}", content);
        }

        CommandWithConfig::Find => todo!(),
    }
}

pub fn reset(args: ConfigResetArgs) {
    if !args.list && !args.find {
        eprintln!("You must select at least one configuration file with '--list' or '--find'");
        return;
    }

    let mut commands = Vec::new();

    if args.list {
        commands.push(CommandWithConfig::List);
    }

    if args.find {
        commands.push(CommandWithConfig::Find);
    }

    if args.yes {
        rewrite_config(&commands);
    } else {
        let Ok(response) = util::prompt(consts::RESET_CONFIG_PROMPT) else {
            eprintln!("I/O Error while prompting the user. Aborting.");
            return;
        };

        match response.to_lowercase().as_str() {
            "y" | "yes" => {
                rewrite_config(&commands);
            }
            _ => println!("Aborting reset."),
        }
    }
}

pub fn show(args: ConfigShowArgs) {
    if (!args.list && !args.find) || (args.list && args.find) {
        eprintln!("You must select at most one configuration file with '--list' or '--find'");
        return;
    }

    if args.list {
        print_config(CommandWithConfig::List);
    }

    if args.find {
        print_config(CommandWithConfig::Find);
    }
}
