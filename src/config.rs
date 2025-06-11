use std::collections::HashMap;

use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    #[serde(default)]
    directories: HashMap<String, String>,

    #[serde(default)]
    files: HashMap<String, String>,

    #[serde(default = "default_dir_icon")]
    default_dir_icon: String,

    #[serde(default = "default_file_icon")]
    default_file_icon: String,

    #[serde(default = "default_unknown_icon")]
    unknown_icon: String,
}

fn default_dir_icon() -> String {
    "󰉋".to_string()
}

fn default_file_icon() -> String {
    "󰈔".to_string()
}

fn default_unknown_icon() -> String {
    "󰗼".to_string()
}

impl Default for Config {
    fn default() -> Self {
        Self {
            directories: HashMap::new(),
            files: HashMap::new(),
            default_dir_icon: default_dir_icon(),
            default_file_icon: default_file_icon(),
            unknown_icon: default_unknown_icon(),
        }
    }
}

impl Config {
    pub fn parse() -> Self {
        let Some(config_dir) = dirs::config_dir() else {
            return Config::default();
        };

        let config_path = config_dir.join("pls").join("config.toml");

        if !config_path.exists() {
            return Config::default();
        }

        let Ok(config_str) = std::fs::read_to_string(&config_path) else {
            return Config::default();
        };

        let Ok(config) = toml::from_str::<Config>(&config_str) else {
            eprintln!("Failed to parse config file at: {}", config_path.display());
            return Config::default();
        };

        config
    }

    pub fn dir_icon(&self, name: &str) -> String {
        self.directories
            .get(name)
            .unwrap_or(&self.default_dir_icon)
            .to_string()
    }

    pub fn file_icon(&self, name: &str) -> String {
        self.files
            .get(name)
            .unwrap_or(&self.default_file_icon)
            .to_string()
    }

    pub fn unknown_icon(&self) -> String {
        self.unknown_icon.to_string()
    }
}
