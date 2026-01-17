use crate::{
    args::{Command, CommandWithConfig},
    consts,
};
use std::{
    fs,
    io::{self, Write},
    path::PathBuf,
};

pub fn config_path(cmd: CommandWithConfig) -> Option<PathBuf> {
    let Some(dir) = dirs::config_dir() else {
        return None;
    };

    fs::create_dir_all(&dir).ok()?;

    let dir = dir.join(consts::PLS_CONFIG_DIRNAME);

    match cmd {
        CommandWithConfig::List => Some(dir.join(consts::LS_CONFIG_FILENAME)),
        CommandWithConfig::Find => Some(dir.join(consts::FIND_CONFIG_FILENAME)),
    }
}

pub fn prompt(message: &str) -> io::Result<String> {
    print!("{}", message);
    io::stdout().flush()?;

    let mut input = String::new();

    io::stdin().read_line(&mut input)?;

    Ok(input.trim().to_string())
}
