use crate::{args::CommandWithConfig, consts, util};
use serde::{Deserialize, Serialize};
use std::fs;

pub trait Parse: Default + Sized {
    /// If the parsing of a configuration file fails,
    /// it should return a default one with an error message,
    /// hence only the `Self` return type
    fn parse() -> Self;
}

#[derive(Debug, Clone, Copy)]
#[derive(Default)]
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SizeUnit {
    #[default]
    Auto,

    #[serde(rename = "b")]
    Bytes,

    #[serde(rename = "kb")]
    KiloBytes,

    #[serde(rename = "mb")]
    MegaBytes,

    #[serde(rename = "gb")]
    GigaBytes,
}

#[derive(Debug, Clone)]
#[derive(Serialize, Deserialize)]
pub struct LsIconConfig {
    pub file: String,
    pub directory: String,
    pub symlink_file: String,
    pub symlink_directory: String,
}

impl Default for LsIconConfig {
    fn default() -> Self {
        Self {
            file: String::from("s"),
            directory: String::from("s"),
            symlink_file: String::from("l"),
            symlink_directory: String::from("l"),
        }
    }
}

#[derive(Debug, Clone)]
#[derive(Serialize, Deserialize)]
pub struct LsConfig {
    pub format: Vec<String>,
    pub padding: usize,
    pub size_unit: SizeUnit,
    pub date_format: String,
    pub icon: LsIconConfig,
}

impl Default for LsConfig {
    fn default() -> Self {
        Self {
            format: vec![String::from("{icon}"), String::from("{name}")],
            padding: 2,
            size_unit: SizeUnit::Bytes,
            date_format: String::from("%d/%m %H:%M"),
            icon: LsIconConfig::default(),
        }
    }
}

impl Parse for LsConfig {
    fn parse() -> Self {
        let Some(path) = util::config_path(CommandWithConfig::List) else {
            eprintln!("Failed to parse configuration: no file found.");
            return Self::default();
        };

        let Ok(content) = fs::read_to_string(&path) else {
            let config = Self::default();

            if let Err(e) = fs::write(&path, toml::to_string(&config).unwrap()) {
                eprintln!("Failed to write default config file: {}", e);
                return config;
            }

            return config;
        };

        match toml::from_str(&content) {
            Ok(c) => c,
            Err(e) => {
                eprintln!("Failed to parse configuration: {}", e);
                Self::default()
            }
        }
    }
}
