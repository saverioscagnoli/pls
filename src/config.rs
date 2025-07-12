use serde::Deserialize;
use smacro::s;
use std::path::PathBuf;

#[derive(Debug, Clone, Deserialize)]
pub struct LsConfig {
    #[serde(default = "LsConfig::default_templates")]
    pub templates: Vec<String>,

    #[serde(default = "LsConfig::default_time_format")]
    pub time_format: String,
}

impl LsConfig {
    fn default_templates() -> Vec<String> {
        vec![
            s!("{ :depth}{name}"),
            s!("{permissions}"),
            s!("{size}"),
            s!("{last_modified}"),
            s!("{nlink}"),
        ]
    }

    fn default_time_format() -> String {
        s!("%b %d %H:%M")
    }
}

impl Default for LsConfig {
    fn default() -> Self {
        Self {
            templates: Self::default_templates(),
            time_format: Self::default_time_format(),
        }
    }
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct Config {
    #[serde(default)]
    pub ls: LsConfig,
}

impl Config {
    fn path() -> Option<PathBuf> {
        dirs::config_dir().map(|dir| dir.join("pls").join("config.toml"))
    }

    pub fn parse() -> Self {
        let Some(path) = Config::path() else {
            return Config::default();
        };

        let Ok(config) = std::fs::read_to_string(&path) else {
            return Config::default();
        };

        match toml::from_str(&config) {
            Ok(config) => config,
            Err(e) => {
                eprintln!("Failed to parse config file: {}", e);
                Config::default()
            }
        }
    }
}
