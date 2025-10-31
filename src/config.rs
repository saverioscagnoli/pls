use figura::Value;
use serde::Deserialize;
use std::{
    collections::HashMap, fmt::Display, hash::Hash, os::unix::fs::PermissionsExt, path::Path,
};

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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FileKind {
    File,
    Directory,
    SymlinkFile,
    SymlinkDirectory,
    Executable,
    BrokenSymlink,
}

impl Display for FileKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let kind_str = match self {
            FileKind::File => "file",
            FileKind::Directory => "directory",
            FileKind::SymlinkFile => "symlink_file",
            FileKind::SymlinkDirectory => "symlink_directory",
            FileKind::Executable => "executable",
            FileKind::BrokenSymlink => "broken_symlink",
        };

        write!(f, "{}", kind_str)
    }
}

impl<'de> Deserialize<'de> for FileKind {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s: &str = serde::Deserialize::deserialize(deserializer)?;

        match s {
            "file" => Ok(FileKind::File),
            "directory" => Ok(FileKind::Directory),
            "symlink_file" => Ok(FileKind::SymlinkFile),
            "symlink_directory" => Ok(FileKind::SymlinkDirectory),
            "executable" => Ok(FileKind::Executable),
            "broken_symlink" => Ok(FileKind::BrokenSymlink),
            _ => Err(serde::de::Error::custom(format!(
                "Unknown file kind: {}",
                s
            ))),
        }
    }
}

impl FileKind {
    pub fn from_path<P: AsRef<Path>>(path: P) -> (Self, std::fs::Metadata) {
        let metadata = std::fs::symlink_metadata(&path).unwrap();

        if metadata.file_type().is_symlink() {
            // Try to follow the symlink - if it fails, the symlink is broken
            match std::fs::metadata(&path) {
                Ok(target_metadata) => {
                    if target_metadata.is_dir() {
                        (FileKind::SymlinkDirectory, metadata)
                    } else {
                        (FileKind::SymlinkFile, metadata)
                    }
                }

                Err(_) => (FileKind::BrokenSymlink, metadata),
            }
        } else if metadata.is_dir() {
            (FileKind::Directory, metadata)
        } else if metadata.permissions().mode() & 0o111 != 0 {
            (FileKind::Executable, metadata)
        } else {
            (FileKind::File, metadata)
        }
    }
}

#[derive(Debug, Clone)]
pub enum Op {
    Equal,
    Greater,
    Less,
    GreaterEqual,
    LessEqual,
}

impl<'de> Deserialize<'de> for Op {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s: String = Deserialize::deserialize(deserializer)?;
        match s.to_lowercase().as_str() {
            "=" | "eq" => Ok(Op::Equal),
            ">" | "gt" => Ok(Op::Greater),
            "<" | "lt" => Ok(Op::Less),
            ">=" | "gte" => Ok(Op::GreaterEqual),
            "<=" | "lte" => Ok(Op::LessEqual),
            _ => Err(serde::de::Error::custom(format!("invalid operator: {}", s))),
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct Condition<T> {
    pub variable: String,
    pub op: Op,
    pub value: String,
    pub result: T,
}

impl<T> Condition<T> {
    fn evaluate(&self, ctx: &HashMap<&'static str, Value>) -> bool {
        let value = match ctx.get(self.variable.as_str()) {
            Some(v) => v.to_string(),
            None => return false,
        };

        match self.op {
            Op::Equal => value == self.value,
            Op::Greater => {
                if let (Ok(a), Ok(b)) = (value.parse::<i64>(), self.value.parse::<i64>()) {
                    a > b
                } else {
                    value > self.value
                }
            }

            Op::Less => {
                if let (Ok(a), Ok(b)) = (value.parse::<i64>(), self.value.parse::<i64>()) {
                    a < b
                } else {
                    value < self.value
                }
            }

            Op::GreaterEqual => {
                if let (Ok(a), Ok(b)) = (value.parse::<i64>(), self.value.parse::<i64>()) {
                    a >= b
                } else {
                    value >= self.value
                }
            }

            Op::LessEqual => {
                if let (Ok(a), Ok(b)) = (value.parse::<i64>(), self.value.parse::<i64>()) {
                    a <= b
                } else {
                    value <= self.value
                }
            }
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct ListIconConfig {
    #[serde(default)]
    default: String,

    #[serde(default)]
    conditions: Vec<Condition<String>>,
}

impl Default for ListIconConfig {
    fn default() -> Self {
        Self {
            default: String::from("f"),
            conditions: vec![Condition {
                variable: String::from("kind"),
                op: Op::Equal,
                value: String::from("directory"),
                result: "d".to_string(),
            }],
        }
    }
}

impl ListIconConfig {
    pub fn resolve(&self, ctx: &HashMap<&'static str, Value>) -> String {
        for rule in &self.conditions {
            if rule.evaluate(ctx) {
                return rule.result.to_string();
            }
        }

        self.default.to_string()
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
pub enum Color {
    Named(String),   // "red" "blue" "green"
    RGB(u8, u8, u8), // [255, 0, 0]
    Hex(String),     // #FF5733
    Ansi(u8),        // ANSI 256 color code
}

impl Default for Color {
    fn default() -> Self {
        Self::Named("white".to_string())
    }
}

impl Color {
    pub fn to_ansi_foreground(&self) -> String {
        self.to_ansi_with_prefix(38)
    }

    pub fn to_ansi_background(&self) -> String {
        self.to_ansi_with_prefix(48)
    }

    fn to_ansi_with_prefix(&self, prefix: u8) -> String {
        match self {
            Self::Named(name) => {
                let code = match name.to_lowercase().as_str() {
                    "black" => 0,
                    "red" => 1,
                    "green" => 2,
                    "yellow" => 3,
                    "blue" => 4,
                    "magenta" => 5,
                    "cyan" => 6,
                    "white" => 7,
                    "bright black" | "gray" | "grey" => 60,
                    "bright red" => 61,
                    "bright green" => 62,
                    "bright yellow" => 63,
                    "bright blue" => 64,
                    "bright magenta" => 65,
                    "bright cyan" => 66,
                    "bright white" => 67,
                    _ => return String::new(),
                };

                let base = if code >= 60 {
                    if prefix == 48 { 100 - 60 } else { 90 - 60 }
                } else {
                    if prefix == 48 { 40 } else { 30 }
                };

                format!("\x1b[{}m", base + code)
            }
            Self::RGB(r, g, b) => format!("\x1b[{};2;{};{};{}m", prefix, r, g, b),
            Self::Hex(hex) => {
                let hex = hex.trim_start_matches('#');
                if hex.len() == 6 {
                    if let (Ok(r), Ok(g), Ok(b)) = (
                        u8::from_str_radix(&hex[0..2], 16),
                        u8::from_str_radix(&hex[2..4], 16),
                        u8::from_str_radix(&hex[4..6], 16),
                    ) {
                        return format!("\x1b[{};2;{};{};{}m", prefix, r, g, b);
                    }
                }
                String::new()
            }
            Self::Ansi(code) => format!("\x1b[{};5;{}m", prefix, code),
        }
    }
}

/// Enum representing font styling.
#[derive(Debug, Clone)]
pub enum TextStyle {
    Normal,
    Bold,
    Italic,
    Underline,
    Dim,
    Strikethrough,
    Blink,
    Inverse,
    Conceal,
    CrossedOut,
    DoubleUnderline,
}

impl Default for TextStyle {
    fn default() -> Self {
        Self::Normal
    }
}

impl TextStyle {
    pub fn to_ansi(&self) -> &'static str {
        match self {
            Self::Normal => "",
            Self::Bold => "\x1b[1m",
            Self::Italic => "\x1b[3m",
            Self::Underline => "\x1b[4m",
            Self::Dim => "\x1b[2m",
            Self::Strikethrough => "\x1b[9m",
            Self::Blink => "\x1b[5m",
            Self::Inverse => "\x1b[7m",
            Self::Conceal => "\x1b[8m",
            Self::CrossedOut => "\x1b[9m",
            Self::DoubleUnderline => "\x1b[21m",
        }
    }
}

impl<'de> Deserialize<'de> for TextStyle {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s: String = Deserialize::deserialize(deserializer)?;
        match s.to_lowercase().as_str() {
            "normal" => Ok(TextStyle::Normal),
            "bold" => Ok(TextStyle::Bold),
            "italic" => Ok(TextStyle::Italic),
            "underline" => Ok(TextStyle::Underline),
            "dim" => Ok(TextStyle::Dim),
            "strikethrough" => Ok(TextStyle::Strikethrough),
            "blink" => Ok(TextStyle::Blink),
            "inverse" => Ok(TextStyle::Inverse),
            "conceal" => Ok(TextStyle::Conceal),
            "crossed out" => Ok(TextStyle::CrossedOut),
            "double underline" => Ok(TextStyle::DoubleUnderline),
            _ => Err(serde::de::Error::custom(format!(
                "invalid text style: {}",
                s
            ))),
        }
    }
}

#[derive(Debug, Default, Clone, Deserialize)]
#[serde(default)]
pub struct Style {
    pub foreground: Option<Color>,
    pub background: Option<Color>,
    pub text: Option<Vec<TextStyle>>,
}

impl Style {
    pub fn apply<S: AsRef<str>>(&self, s: S) -> String {
        let mut out = String::new();
        let mut applied = false;

        if let Some(fg) = &self.foreground {
            let code = fg.to_ansi_foreground();

            if !code.is_empty() {
                out.push_str(&code);
                applied = true;
            }
        }

        if let Some(bg) = &self.background {
            let code = bg.to_ansi_background();

            if !code.is_empty() {
                out.push_str(&code);
                applied = true;
            }
        }

        if let Some(text_styles) = &self.text {
            for style in text_styles {
                let code = style.to_ansi();

                if !code.is_empty() {
                    out.push_str(code);
                    applied = true;
                }
            }
        }

        out.push_str(s.as_ref());

        if applied {
            out.push_str("\x1b[0m");
        }

        out
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct FieldStyle {
    #[serde(default)]
    pub default: Option<Style>,

    #[serde(default)]
    pub conditions: Vec<Condition<Style>>,
}

impl FieldStyle {
    pub fn resolve<S: AsRef<str>>(&self, value: S, ctx: &HashMap<&'static str, Value>) -> String {
        for rule in &self.conditions {
            if rule.evaluate(ctx) {
                return rule.result.apply(value);
            }
        }

        self.default
            .as_ref()
            .unwrap_or(&Style::default())
            .apply(value)
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct ListConfig {
    pub format: Vec<String>,
    pub padding: usize,
    pub icons: ListIconConfig,
    pub style: HashMap<String, FieldStyle>,
    pub size_unit: SizeUnit,
    pub created_fmt: String,
    pub modified_fmt: String,
    pub accessed_fmt: String,
}

impl Default for ListConfig {
    fn default() -> Self {
        Self {
            format: vec![
                String::from("{kind}"),
                String::from("{name}"),
                String::from("{permissions}"),
                String::from("{size}"),
                String::from("{modified}"),
            ],
            padding: 2,
            icons: ListIconConfig::default(),
            style: HashMap::new(),
            size_unit: SizeUnit::Auto,
            created_fmt: String::from("%b %d %H:%M"),
            modified_fmt: String::from("%b %d %H:%M"),
            accessed_fmt: String::from("%b %d %H:%M"),
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    #[serde(default)]
    pub ls: ListConfig,
}

impl Config {
    const SCHEMA: &str = include_str!("../config.schema.json");
    const DEFAULT: &str = include_str!("../config.default.json");

    pub fn parse() -> Result<Self, Box<dyn std::error::Error>> {
        let config_dir = dirs::config_dir()
            .ok_or("Could not determine config directory")?
            .join("pls");

        let schema_file = config_dir.join("config.schema.json");
        let config_file = config_dir.join("config.json");

        std::fs::create_dir_all(&config_dir)?;

        if !schema_file.exists() {
            std::fs::write(&schema_file, Self::SCHEMA)?;
        }

        if !config_file.exists() {
            std::fs::write(&config_file, Self::DEFAULT)?;
        }

        let config_content = std::fs::read_to_string(&config_file)?;
        let config: Config = serde_json::from_str(&config_content)?;

        Ok(config)
    }
}
