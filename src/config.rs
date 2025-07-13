use std::path::PathBuf;

use serde::Deserialize;
use serde_inline_default::serde_inline_default;
use smacro::s;

#[derive(Debug, Clone, Deserialize)]
#[serde_inline_default]
#[serde(default)]
pub struct LsConfig {
    pub padding: usize,

    pub headers: Vec<String>,
    pub templates: Vec<String>,

    pub time_format: String,
}

impl Default for LsConfig {
    fn default() -> Self {
        Self {
            padding: 3,
            headers: vec![],
            templates: vec![
                s!("{ :depth}{name}"),
                s!("{permissions^}"),
                s!("{size>} b"),
                s!("{last_modified^}"),
                s!("{nlink>} ->"),
            ],

            // American date format sucks
            // this is way better
            time_format: "%d/%m %H:%M".to_string(),
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    #[serde(default)]
    pub ls: LsConfig,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            ls: LsConfig::default(),
        }
    }
}

impl Config {
    pub const VARIABLES: [&'static str; 7] = [
        "name",
        "type",
        "depth",
        "permissions",
        "size",
        "last_modified",
        "nlink",
    ];

    /// Returns the path to the configuration file.
    /// /home/<user>/.config/pls/config.json
    fn path() -> Option<PathBuf> {
        dirs::config_dir().map(|p| p.join("pls").join("config.json"))
    }

    pub fn parse() -> Self {
        let Some(path) = Config::path() else {
            return Config::default();
        };

        let Ok(str) = std::fs::read_to_string(&path) else {
            return Config::default();
        };

        match serde_json::from_str(&str) {
            Ok(c) => c,
            Err(e) => {
                eprintln!("Failed to deserialize config: {}", e);
                Config::default()
            }
        }
    }
}
