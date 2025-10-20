use crate::{commands::list::FileKind, err::PlsError, util};
use serde::{Deserialize, de::DeserializeOwned};
use std::{
    collections::{HashMap, HashSet},
    str::FromStr,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ListVariable {
    Name,
    Path,
    Kind,
    Size,
    Depth,
    Icon,
    Permissions,
    Created,
    Modified,
    Accessed,
    Owner,
    Group,
    NLink,
}

impl<'de> Deserialize<'de> for ListVariable {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s: String = Deserialize::deserialize(deserializer)?;
        ListVariable::from_str(&s)
            .map_err(|_| serde::de::Error::custom(format!("invalid list variable: {}", s)))
    }
}

impl FromStr for ListVariable {
    type Err = ();

    fn from_str(input: &str) -> Result<ListVariable, Self::Err> {
        match input {
            "name" => Ok(ListVariable::Name),
            "path" => Ok(ListVariable::Path),
            "size" => Ok(ListVariable::Size),
            "kind" => Ok(ListVariable::Kind),
            "depth" => Ok(ListVariable::Depth),
            "icon" => Ok(ListVariable::Icon),
            "permissions" => Ok(ListVariable::Permissions),
            "created" => Ok(ListVariable::Created),
            "modified" => Ok(ListVariable::Modified),
            "accessed" => Ok(ListVariable::Accessed),
            "owner" => Ok(ListVariable::Owner),
            "group" => Ok(ListVariable::Group),
            "nlink" => Ok(ListVariable::NLink),
            _ => Err(()),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SizeUnit {
    Auto,
    Bytes,
    KB,
    MB,
    GB,
    TB,
}

impl SizeUnit {
    fn format_size_auto(bytes: u64) -> String {
        const KB: u64 = 1024;
        const MB: u64 = KB * 1024;
        const GB: u64 = MB * 1024;
        const TB: u64 = GB * 1024;

        match bytes {
            0..KB => format!("{} B", bytes),
            KB..MB => format!("{:.2} kB", bytes as f64 / KB as f64),
            MB..GB => format!("{:.2} MB", bytes as f64 / MB as f64),
            GB..TB => format!("{:.2} GB", bytes as f64 / GB as f64),
            _ => format!("{:.2} TB", bytes as f64 / TB as f64),
        }
    }

    pub fn format_bytes(&self, bytes: u64) -> String {
        match self {
            SizeUnit::Auto => SizeUnit::format_size_auto(bytes),
            SizeUnit::Bytes => format!("{} B", bytes),
            SizeUnit::KB => {
                let kb = bytes as f64 / 1024.0;
                format!("{:.2} kB", kb)
            }
            SizeUnit::MB => {
                let mb = bytes as f64 / (1024.0 * 1024.0);
                format!("{:.2} MB", mb)
            }
            SizeUnit::GB => {
                let gb = bytes as f64 / (1024.0 * 1024.0 * 1024.0);
                format!("{:.2} GB", gb)
            }
            SizeUnit::TB => {
                let tb = bytes as f64 / (1024.0 * 1024.0 * 1024.0 * 1024.0);
                format!("{:.2} TB", tb)
            }
        }
    }
}

impl Default for SizeUnit {
    fn default() -> Self {
        SizeUnit::Auto
    }
}

impl<'de> Deserialize<'de> for SizeUnit {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s: String = Deserialize::deserialize(deserializer)?;
        match s.to_lowercase().as_str() {
            "auto" => Ok(SizeUnit::Auto),
            "b" => Ok(SizeUnit::Bytes),
            "kb" => Ok(SizeUnit::KB),
            "mb" => Ok(SizeUnit::MB),
            "gb" => Ok(SizeUnit::GB),
            "tb" => Ok(SizeUnit::TB),
            _ => Err(serde::de::Error::custom(format!(
                "invalid size unit: {}",
                s
            ))),
        }
    }
}

pub trait Style {
    fn apply<S: AsRef<str>>(&self, val: S) -> String;
    fn reset() -> &'static str {
        "\x1b[0m"
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
pub enum Color {
    Named(String),   // "red", "blue", "green"
    Rgb(u8, u8, u8), // RGB values
    Hex(String),     // "#FF5733"
    Ansi(u8),        // ANSI 256 color code
}

impl Default for Color {
    fn default() -> Self {
        Color::Named("white".to_string())
    }
}

impl Color {
    pub fn to_ansi(&self) -> String {
        match self {
            Color::Named(name) => match name.to_lowercase().as_str() {
                "black" => "\x1b[30m".to_string(),
                "red" => "\x1b[31m".to_string(),
                "green" => "\x1b[32m".to_string(),
                "yellow" => "\x1b[33m".to_string(),
                "blue" => "\x1b[34m".to_string(),
                "magenta" => "\x1b[35m".to_string(),
                "cyan" => "\x1b[36m".to_string(),
                "white" => "\x1b[37m".to_string(),
                "bright_black" | "gray" => "\x1b[90m".to_string(),
                "bright_red" => "\x1b[91m".to_string(),
                "bright_green" => "\x1b[92m".to_string(),
                "bright_yellow" => "\x1b[93m".to_string(),
                "bright_blue" => "\x1b[94m".to_string(),
                "bright_magenta" => "\x1b[95m".to_string(),
                "bright_cyan" => "\x1b[96m".to_string(),
                "bright_white" => "\x1b[97m".to_string(),
                _ => String::new(),
            },

            Color::Rgb(r, g, b) => format!("\x1b[38;2;{};{};{}m", r, g, b),

            Color::Hex(hex) => {
                let hex = hex.trim_start_matches('#');

                if hex.len() == 6 {
                    if let (Ok(r), Ok(g), Ok(b)) = (
                        u8::from_str_radix(&hex[0..2], 16),
                        u8::from_str_radix(&hex[2..4], 16),
                        u8::from_str_radix(&hex[4..6], 16),
                    ) {
                        return format!("\x1b[38;2;{};{};{}m", r, g, b);
                    }
                }
                String::new()
            }

            Color::Ansi(code) => format!("\x1b[38;5;{}m", code),
        }
    }
}

impl Style for Color {
    fn apply<S: AsRef<str>>(&self, val: S) -> String {
        format!("{}{}{}", self.to_ansi(), val.as_ref(), Color::reset())
    }
}

#[derive(Debug, Clone, Deserialize)]
pub enum TextStyle {
    Normal,
    Bold,
    Italic,
    Underline,
    StrikeThrough,
    Blink,
    Inverse,
    Conceal,
    CrossedOut,
    DoubleUnderline,
}

impl Default for TextStyle {
    fn default() -> Self {
        TextStyle::Normal
    }
}

impl TextStyle {
    pub fn to_ansi(&self) -> &'static str {
        match self {
            TextStyle::Normal => "",
            TextStyle::Bold => "\x1b[1m",
            TextStyle::Italic => "\x1b[3m",
            TextStyle::Underline => "\x1b[4m",
            TextStyle::StrikeThrough => "\x1b[9m",
            TextStyle::Blink => "\x1b[5m",
            TextStyle::Inverse => "\x1b[7m",
            TextStyle::Conceal => "\x1b[8m",
            TextStyle::CrossedOut => "\x1b[9m",
            TextStyle::DoubleUnderline => "\x1b[21m",
        }
    }

    pub fn reset() -> &'static str {
        "\x1b[0m"
    }
}

impl Style for TextStyle {
    fn apply<S: AsRef<str>>(&self, val: S) -> String {
        let ansi = self.to_ansi();
        format!("{}{}{}", ansi, val.as_ref(), TextStyle::reset())
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct ListIconConfig {
    #[serde(default = "ListIconConfig::default_enabled")]
    pub enabled: bool,

    #[serde(default = "ListIconConfig::default_file_icon")]
    pub file: String,

    #[serde(default = "ListIconConfig::default_directory_icon")]
    pub directory: String,

    #[serde(default = "ListIconConfig::default_symlink_file_icon")]
    pub symlink_file: String,

    #[serde(default = "ListIconConfig::default_symlink_directory_icon")]
    pub symlink_directory: String,

    #[serde(default = "ListIconConfig::default_executable_icon")]
    pub executable: String,

    #[serde(default = "ListIconConfig::default_extensions_icons")]
    pub extensions: HashMap<String, String>,
}

impl ListIconConfig {
    pub fn default_enabled() -> bool {
        true
    }

    pub fn default_file_icon() -> String {
        String::from("󰈔")
    }

    pub fn default_directory_icon() -> String {
        String::from("󰉋")
    }

    pub fn default_symlink_file_icon() -> String {
        String::from("󰈕")
    }

    pub fn default_symlink_directory_icon() -> String {
        String::from("󰉒")
    }

    pub fn default_executable_icon() -> String {
        String::from("󰜢")
    }

    pub fn default_extensions_icons() -> HashMap<String, String> {
        HashMap::new()
    }
}

impl Default for ListIconConfig {
    fn default() -> Self {
        Self {
            enabled: Self::default_enabled(),
            file: Self::default_file_icon(),
            directory: Self::default_directory_icon(),
            symlink_file: Self::default_symlink_file_icon(),
            symlink_directory: Self::default_symlink_directory_icon(),
            executable: Self::default_executable_icon(),
            extensions: Self::default_extensions_icons(),
        }
    }
}

/// Enum representing color configuration per-variable,
/// so for example you could wanted to color the "name" variable based
/// on file type or extension,
#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
pub enum VariableStyleConfig<T: Default> {
    /// This will apply the said color to all instances
    /// of the variable across the listing.
    Simple(T),

    Complex {
        /// Color applied based on the entry kind.
        #[serde(default)]
        kinds: HashMap<FileKind, T>,

        /// Color applied based on the file extension.
        #[serde(default)]
        extensions: HashMap<String, T>,

        /// The fallback color if no other rule matches.
        /// The default color is white
        #[serde(default)]
        default: T,
    },
}

impl<T: Default> VariableStyleConfig<T> {
    pub fn resolve_style(&self, kind: &FileKind, extension: Option<&str>) -> &T {
        match self {
            VariableStyleConfig::Simple(t) => t,
            VariableStyleConfig::Complex {
                kinds,
                extensions,
                default,
            } => {
                // First try to match by extension if provided
                if let Some(ext) = extension {
                    if let Some(t) = extensions.get(ext) {
                        return t;
                    }
                }

                if let Some(t) = kinds.get(&kind) {
                    return t;
                }

                default
            }
        }
    }
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct StyleConfig<T: Default + Style> {
    #[serde(default)]
    pub enabled: bool,

    #[serde(default, flatten)]
    pub variables: HashMap<ListVariable, VariableStyleConfig<T>>,
}

impl<T: Default + Style> StyleConfig<T> {
    pub fn apply_style(
        &self,
        kind: &FileKind,
        ext: Option<&str>,
        var: &ListVariable,
        value: String,
    ) -> String {
        if !self.enabled {
            return value;
        }

        if let Some(conf) = self.variables.get(var) {
            let style = conf.resolve_style(kind, ext);
            return style.apply(value);
        }

        value
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct ListConfig {
    #[serde(default = "ListConfig::default_format")]
    pub format: Vec<String>,

    #[serde(default = "ListConfig::default_padding")]
    pub padding: usize,

    #[serde(default = "ListConfig::default_headers")]
    pub headers: Vec<String>,

    #[serde(default = "ListConfig::default_accessed_format")]
    pub accessed_format: String,

    #[serde(default = "ListConfig::default_modified_format")]
    pub modified_format: String,

    #[serde(default = "ListConfig::default_created_format")]
    pub created_format: String,

    #[serde(default)]
    pub size_unit: SizeUnit,

    #[serde(default)]
    pub icons: ListIconConfig,

    #[serde(default)]
    pub colors: StyleConfig<Color>,

    #[serde(default)]
    pub text: StyleConfig<TextStyle>,
}

impl ListConfig {
    pub fn default_format() -> Vec<String> {
        vec![
            String::from("{kind}"),
            String::from("{name}"),
            String::from("{size}"),
            String::from("{modified}"),
        ]
    }

    pub fn default_padding() -> usize {
        2
    }

    pub fn default_headers() -> Vec<String> {
        Vec::new()
    }

    pub fn default_accessed_format() -> String {
        String::from("%Y-%m-%d %H:%M")
    }

    pub fn default_modified_format() -> String {
        String::from("%Y-%m-%d %H:%M")
    }

    pub fn default_created_format() -> String {
        String::from("%Y-%m-%d %H:%M")
    }

    pub fn list_variables(&self) -> Vec<ListVariable> {
        let mut stripped = String::new();

        for t in &self.format {
            stripped.push_str(util::keep_ascii_letters_and_whitespace(t).as_str());
            stripped.push(' ');
        }

        stripped
            .split_whitespace()
            .filter_map(|var| ListVariable::from_str(var).ok())
            .collect::<HashSet<_>>()
            .into_iter()
            .collect()
    }
}

impl Default for ListConfig {
    fn default() -> Self {
        Self {
            format: Self::default_format(),
            padding: Self::default_padding(),
            headers: Self::default_headers(),
            accessed_format: Self::default_accessed_format(),
            modified_format: Self::default_modified_format(),
            created_format: Self::default_created_format(),
            size_unit: SizeUnit::default(),
            icons: ListIconConfig::default(),
            colors: StyleConfig::default(),
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct Config {
    pub ls: ListConfig,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            ls: ListConfig::default(),
        }
    }
}

impl Config {
    pub fn parse() -> Result<Self, PlsError> {
        let config_dir = dirs::config_dir()
            .ok_or_else(|| PlsError::ConfigNotFound)?
            .join("pls");

        let possble_paths = &[
            config_dir.join("config.toml"),
            config_dir.join("config.json"),
            config_dir.join("config.jsonc"),
            config_dir.join("config.json5"),
            config_dir.join("config.yaml"),
        ];

        let path = possble_paths
            .iter()
            .find(|p| p.exists())
            .ok_or_else(|| PlsError::ConfigNotFound)?;

        let content = std::fs::read_to_string(path)?;
        let config: Config =
            match path.extension().and_then(|s| s.to_str()) {
                Some("toml") => {
                    toml::from_str(&content).map_err(|e| PlsError::ParsingError(e.to_string()))?
                }

                Some("json") => serde_json::from_str(&content)
                    .map_err(|e| PlsError::ParsingError(e.to_string()))?,

                Some("jsonc") | Some("json5") => {
                    json5::from_str(&content).map_err(|e| PlsError::ParsingError(e.to_string()))?
                }

                Some("yaml") | Some("yml") => serde_yaml::from_str(&content)
                    .map_err(|e| PlsError::ParsingError(e.to_string()))?,

                _ => return Err(PlsError::ConfigNotFound),
            };

        Ok(config)
    }
}
