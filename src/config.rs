use serde::Deserialize;
use serde_inline_default::serde_inline_default;
use smacro::s;
use std::path::PathBuf;

#[derive(Debug, Clone, Deserialize)]
#[serde_inline_default]
#[serde(default)]
pub struct LsConfig {
    /// Padding between columns
    /// Default: 3
    /// This is used to align the columns in the output
    pub padding: usize,

    /// Headers for the output
    /// These are used to display the column names in the output
    /// Default: []
    pub headers: Vec<String>,

    /// Actual output
    /// Variables are used to display the file information
    /// Possible variables:
    /// - name: The name of the file
    /// - type: The type of the file (directory, executable, file, symlink)
    /// - depth: The depth of the file in the directory tree
    /// - permissions: The permissions of the file
    /// - size: The size of the file in bytes
    /// - last_modified: The last modified time of the file
    /// - nlink: The number of hard links to the file
    /// 
    /// See https://crates.io/crates/figura for more information on the syntax
    /// when using the templates.
    /// 
    /// Supports conditional formatting based on a variable, pattern repeating, etc.
    pub templates: Vec<String>,

    /// Time format for the last modified time
    /// Default: "%d/%m %H:%M"
    /// See chrono::format::strftime for more information on the format
    pub time_format: String,
}

impl Default for LsConfig {
    fn default() -> Self {
        Self {
            padding: 3,
            headers: vec![],
            templates: vec![
                s!(
                    "{ :depth}{[type](directory:\x1b[34m󰉋\x1b[0m)(executable:\x1b[32m󰈔\x1b[0m)(file:󰈔)(symlink:\x1b[33m󱅷\x1b[0m)} {[type](directory:\x1b[1;34m)(executable:\x1b[1;32m)(file:)(symlink:\x1b[1;33m)}{[type](directory:name)(executable:name)(file:name)(symlink:name)}{[type](directory:\x1b[0m)(executable:\x1b[0m)(file:)(symlink:\x1b[0m)}"
                ),
                s!("\x1b[90m{permissions^}\x1b[0m"),
                s!("{size>} b"),
                s!("\x1b[90m{last_modified^}\x1b[0m"),
                s!("\x1b[33m󱞩 {nlink>}\x1b[0m"),
            ],
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
