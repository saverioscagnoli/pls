use chrono::{DateTime, Local};
use serde::{Deserialize, Deserializer};
use serde_inline_default::serde_inline_default;
use std::{collections::HashMap, ops::Deref, str::FromStr};

#[serde_inline_default]
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
pub struct LsConfig {
    /// The format to use for the output.
    /// This is a list of strings, where each string is a format specifier.
    /// The available format specifiers are  :
    /// - `{icon}`: The icon for the file or directory.
    /// - `{name}`: The name of the file or directory.
    /// - `{permissions}`: The permissions of the file or directory.
    /// - `{size}`: The size of the file or directory in bytes.
    /// - `{last_modified}`: The last modified date and time of the file or directory.
    /// - `{git_status}`: The git status of the file or directory (if applicable).
    /// - `{nlink}`: The number of hard links to the file or directory.
    /// - `{link_target}`: The target of the symlink (if applicable).
    /// - `{depth}`: The depth of the file or directory in the tree.
    #[serde_inline_default(vec!["{ :depth} {icon}  {name}".to_string(), "{permissions}".to_string()])]
    pub format: Vec<String>,

    /// The padding between the columns in the output.
    /// This is the number of spaces to use for padding.
    /// The default value is `2`.
    #[serde_inline_default(2)]
    pub padding: usize,

    /// The format to use for the date and time in the output.
    /// This is a strftime format string.
    /// The default value is `"%Y/%m/%d %H:%M"`.
    ///
    /// List of available strftime format specifiers:
    /// - %a  - Abbreviated weekday name (e.g., "Mon")
    /// - %A  - Full weekday name (e.g., "Monday")
    /// - %b  - Abbreviated month name (e.g., "Jan")
    /// - %B  - Full month name (e.g., "January")
    /// - %c  - Date and time representation (e.g., "Tue Aug 16 21:30:00 2022")
    /// - %C  - Century number (year/100) as a 2-digit integer (e.g., "20")
    /// - %d  - Day of the month as a zero-padded decimal (01–31)
    /// - %D  - Equivalent to "%m/%d/%y"
    /// - %e  - Day of the month as a space-padded decimal ( 1–31)
    /// - %F  - Equivalent to "%Y-%m-%d" (ISO 8601 format)
    /// - %g  - Last two digits of the ISO 8601 week-based year
    /// - %G  - ISO 8601 week-based year
    /// - %h  - Equivalent to "%b"
    /// - %H  - Hour in 24h format (00–23)
    /// - %I  - Hour in 12h format (01–12)
    /// - %j  - Day of the year (001–366)
    /// - %k  - Hour (0–23), space-padded
    /// - %l  - Hour (1–12), space-padded
    /// - %m  - Month as a zero-padded decimal (01–12)
    /// - %M  - Minute (00–59)
    /// - %n  - Newline character
    /// - %p  - AM or PM designation
    /// - %P  - am or pm (lowercase)
    /// - %r  - Time in 12h format (e.g., "09:30:00 PM")
    /// - %R  - Time in 24h format without seconds (e.g., "21:30")
    /// - %s  - Seconds since the Unix Epoch
    /// - %S  - Second (00–60, accounts for leap seconds)
    /// - %t  - Tab character
    /// - %T  - Time in 24h format (e.g., "21:30:00")
    /// - %u  - ISO 8601 weekday number (1–7, Monday = 1)
    /// - %U  - Week number of the year, Sunday as the first day (00–53)
    /// - %V  - ISO 8601 week number (01–53)
    /// - %w  - Weekday number (0–6, Sunday = 0)
    /// - %W  - Week number, Monday as first day of week (00–53)
    /// - %x  - Date representation (e.g., "08/16/22")
    /// - %X  - Time representation (e.g., "21:30:00")
    /// - %y  - Year without century (00–99)
    /// - %Y  - Year with century (e.g., "2022")
    /// - %z  - +hhmm numeric time zone (e.g., "-0400")
    /// - %:z - +hh:mm numeric time zone (e.g., "-04:00")
    /// - %::z - +hh:mm:ss numeric time zone (rare)
    /// - %Z  - Time zone abbreviation (e.g., "EST")
    /// - %%  - Literal '%' character
    #[serde_inline_default("%Y/%m/%d %H:%M".to_string())]
    pub time_format: String,
}

#[serde_inline_default]
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
pub struct IndicatorsConfig {
    /// The icon map for directories.
    /// The key of the map is the full directory name,
    /// and the value is the icon to use for that directory.
    ///
    /// **Note: this is case-sensitive.**
    #[serde(default)]
    directories: HashMap<String, String>,

    /// The icon map for files.
    /// The key of the map is the extension of the file,
    /// and the value is the icon to use for that file.
    /// **Note: this is case-sensitive.**
    #[serde(default)]
    files: HashMap<String, String>,

    /// The icon to use for directories that are not specified in the `directories` map.
    #[serde_inline_default("󰉋".to_string())]
    default_dir_indicator: String,

    /// The icon to use for files that are not specified in the `files` map.
    #[serde_inline_default("󰈔".to_string())]
    default_file_indicator: String,

    /// The icon to use for files which file type is unknown or not specified.
    #[serde_inline_default("󰗼".to_string())]
    unknown_indicator: String,
}

impl IndicatorsConfig {
    /// Retrieves the icon for a directory or file based on its name
    /// and returns it as a `String`.
    /// It will use the default icon if the name is not found in the respective map.
    pub fn dir<T: AsRef<str>>(&self, name: T) -> String {
        self.directories
            .get(name.as_ref())
            .unwrap_or(&self.default_dir_indicator)
            .to_string()
    }

    /// Retrieves the icon for a file based on its name
    /// and returns it as a `String`.
    /// It will use the default icon if the name is not found in the files map.
    pub fn file<T: AsRef<str>>(&self, ext: T) -> String {
        self.files
            .get(ext.as_ref())
            .unwrap_or(&self.default_file_indicator)
            .to_string()
    }

    /// Returns the default icon for files with unknown or unspecified types.
    pub fn unknown(&self) -> String {
        self.unknown_indicator.to_string()
    }
}

#[serde_inline_default]
#[derive(Debug, Clone, Default, Deserialize)]
pub struct Config {
    /// The configuration for the `ls` command.
    /// This includes options like padding, time format, etc.
    #[serde(default)]
    pub ls: LsConfig,

    /// The configuration for the indicators used in the output.
    /// This includes icons for directories and files, as well as defaults.
    #[serde(default)]
    pub indicators: IndicatorsConfig,
}

impl Config {
    /// Parses the configuration from the `config.toml` file located in the user's config directory.
    /// If the file does not exist or cannot be read, it returns a default configuration.
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

        match toml::from_str(&config_str) {
            Ok(config) => config,
            Err(e) => {
                eprintln!("Failed to parse config file: {}", e);
                Config::default()
            }
        }
    }
}
