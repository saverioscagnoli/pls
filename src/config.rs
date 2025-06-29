use std::path::PathBuf;

use serde::Deserialize;
use serde_inline_default::serde_inline_default;
use smacro::s;

#[serde_inline_default]
#[derive(Debug, Clone, Default, Deserialize)]
pub struct LsConfig {
    #[serde_inline_default(vec![
        s!("{ :depth}{name}"),
        s!("{permissions}"),
        s!("{size>}"),
        s!("{last_modified}"),
        s!("{nlink>} 󱞫"),
    ])]
    pub templates: Vec<String>,

    #[serde_inline_default(s!("%b %d %H:%M"))]
    pub time_format: String,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct Config {
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

        let Ok(config) = toml::from_str(&config) else {
            return Config::default();
        };

        config
    }
}
