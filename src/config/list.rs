use crate::{
    config::{ConfigKind, Parse},
    consts,
    util::{self, FileInfo},
};
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, fs};

#[derive(Debug, Clone)]
#[derive(Serialize, Deserialize)]
pub struct LsConfig {
    #[serde(default)]
    pub format: Vec<String>,

    #[serde(default = "default_padding")]
    pub padding: usize,

    #[serde(default)]
    pub size_unit: SizeUnit,

    #[serde(default = "default_date_format")]
    pub date_format: String,

    #[serde(default)]
    pub icon: IconConfig,

    #[serde(default)]
    pub style: StyleConfig,

    #[serde(default)]
    pub pattern: Vec<Pattern>,

    #[serde(default)]
    pub rules: Vec<Rule>,
}

#[derive(Debug, Clone)]
#[derive(Serialize, Deserialize)]
pub struct IconConfig {
    #[serde(default = "default_icon")]
    pub default: String,

    // Simple shortcuts for common cases
    #[serde(default)]
    pub directory: Option<String>,

    #[serde(default)]
    pub executable: Option<String>,

    #[serde(default)]
    pub symlink: Option<String>,

    #[serde(default)]
    pub broken_symlink: Option<String>,

    #[serde(default)]
    pub ext: HashMap<String, String>,

    #[serde(default)]
    pub name: HashMap<String, String>,

    // Conditional icon selection
    #[serde(default)]
    pub when: Vec<When<String>>,
}

impl LsConfig {
    pub fn resolve_icon(&self, file_info: &FileInfo) -> String {
        // 1. Check global rules first (most specific)
        for rule in &self.rules {
            if self.rule_matches(rule, file_info) {
                if let Some(ref icon) = rule.icon {
                    return icon.clone();
                }
            }
        }

        // 2. Check patterns
        for pattern in &self.pattern {
            if self.pattern_matches(pattern, file_info) {
                if let Some(ref icon) = pattern.icon {
                    return icon.clone();
                }
            }
        }

        // 3. Check exact name match
        if let Some(icon) = self.icon.name.get(&file_info.name) {
            return icon.clone();
        }

        // 4. Check conditional when
        for when in &self.icon.when {
            if self.when_matches(when, file_info) {
                return when.then.clone();
            }
        }

        // 5. Check extension
        if !file_info.ext.is_empty() {
            if let Some(icon) = self.icon.ext.get(&file_info.ext) {
                return icon.clone();
            }
        }

        // 6. Check file type shortcuts (in order of specificity)
        if file_info.is_broken_symlink {
            if let Some(ref icon) = self.icon.broken_symlink {
                return icon.clone();
            }
        }

        if file_info.is_symlink_directory || file_info.is_symlink_file {
            if let Some(ref icon) = self.icon.symlink {
                return icon.clone();
            }
        }

        if file_info.is_executable {
            if let Some(ref icon) = self.icon.executable {
                return icon.clone();
            }
        }

        if file_info.is_directory {
            if let Some(ref icon) = self.icon.directory {
                return icon.clone();
            }
        }

        // 7. Default fallback
        self.icon.default.clone()
    }

    fn when_matches(&self, when: &When<String>, file_info: &FileInfo) -> bool {
        let actual = self.get_var_value(&when.var, file_info);
        self.compare(&actual, &when.op, &when.val)
    }

    fn rule_matches(&self, rule: &Rule, file_info: &FileInfo) -> bool {
        let actual = self.get_var_value(&rule.var, file_info);
        self.compare(&actual, &rule.op, &rule.val)
    }

    fn pattern_matches(&self, pattern: &Pattern, file_info: &FileInfo) -> bool {
        if let Ok(re) = Regex::new(&pattern.pattern) {
            re.is_match(&file_info.name)
        } else {
            false
        }
    }

    fn get_var_value(&self, var: &str, file_info: &FileInfo) -> Value {
        match var {
            "name" => Value::String(file_info.name.clone()),
            "ext" => Value::String(file_info.ext.clone()),
            "size" => Value::Number(file_info.size as i64),
            "perms" => Value::String(file_info.perms.clone()),
            "is_directory" => Value::Bool(file_info.is_directory),
            "is_file" => Value::Bool(file_info.is_file),
            "is_executable" => Value::Bool(file_info.is_executable),
            "is_symlink" => {
                Value::Bool(file_info.is_symlink_directory || file_info.is_symlink_file)
            }
            "is_broken_symlink" => Value::Bool(file_info.is_broken_symlink),
            _ => Value::String(String::new()),
        }
    }

    fn compare(&self, actual: &Value, op: &Op, expected: &Value) -> bool {
        match (actual, expected) {
            (Value::String(a), Value::String(e)) => match op {
                Op::Eq => a == e,
                Op::Ne => a != e,
                Op::Contains => a.contains(e),
                Op::StartsWith => a.starts_with(e),
                Op::EndsWith => a.ends_with(e),
                Op::Matches => {
                    use regex::Regex;
                    Regex::new(e).map(|re| re.is_match(a)).unwrap_or(false)
                }
                _ => false,
            },
            (Value::Number(a), Value::Number(e)) => match op {
                Op::Eq => a == e,
                Op::Ne => a != e,
                Op::Gt => a > e,
                Op::Lt => a < e,
                Op::Gte => a >= e,
                Op::Lte => a <= e,
                _ => false,
            },
            (Value::Bool(a), Value::Bool(e)) => match op {
                Op::Eq => a == e,
                Op::Ne => a != e,
                _ => false,
            },
            _ => false,
        }
    }
}

#[derive(Debug, Clone)]
#[derive(Serialize, Deserialize)]
pub struct StyleConfig {
    #[serde(default)]
    pub name: FieldStyle,

    #[serde(default)]
    pub permissions: FieldStyle,

    #[serde(default)]
    pub size: FieldStyle,

    #[serde(default)]
    pub modified: FieldStyle,

    #[serde(default)]
    pub created: FieldStyle,

    #[serde(default)]
    pub accessed: FieldStyle,
}

#[derive(Debug, Clone)]
#[derive(Serialize, Deserialize)]
#[derive(Default)]
pub struct FieldStyle {
    #[serde(default)]
    pub default: Option<Style>,

    // Simple shortcuts
    #[serde(default)]
    pub directory: Option<Style>,

    #[serde(default)]
    pub executable: Option<Style>,

    #[serde(default)]
    pub symlink: Option<Style>,

    #[serde(default)]
    pub broken_symlink: Option<Style>,

    // Conditional styling
    #[serde(default)]
    pub when: Vec<When<Style>>,
}

// Generic conditional system
#[derive(Debug, Clone)]
#[derive(Serialize, Deserialize)]
pub struct When<T> {
    pub var: String,
    pub op: Op,
    pub val: Value,
    pub then: T,
}

#[derive(Debug, Clone)]
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Op {
    #[serde(alias = "==")]
    Eq,
    #[serde(alias = "!=")]
    Ne,
    #[serde(alias = ">")]
    Gt,
    #[serde(alias = "<")]
    Lt,
    #[serde(alias = ">=")]
    Gte,
    #[serde(alias = "<=")]
    Lte,
    Contains,
    StartsWith,
    EndsWith,
    Matches, // regex
}

#[derive(Debug, Clone)]
#[derive(Serialize, Deserialize)]
#[serde(untagged)]
pub enum Value {
    String(String),
    Number(i64),
    Float(f64),
    Bool(bool),
}

#[derive(Debug, Clone)]
#[derive(Serialize, Deserialize)]
pub struct Style {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fg: Option<Color>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub bg: Option<Color>,

    #[serde(default)]
    pub bold: bool,

    #[serde(default)]
    pub italic: bool,

    #[serde(default)]
    pub underline: bool,

    #[serde(default)]
    pub dim: bool,

    #[serde(default)]
    pub blink: bool,

    #[serde(default)]
    pub strikethrough: bool,

    #[serde(default)]
    pub inverse: bool,
}

#[derive(Debug, Clone)]
#[derive(Serialize, Deserialize)]
#[serde(untagged)]
pub enum Color {
    Named(String),
    Hex(String), // #RRGGBB
    Rgb([u8; 3]),
    Ansi(u8),
}

#[derive(Debug, Clone)]
#[derive(Serialize, Deserialize)]
#[derive(Default)]
#[serde(rename_all = "lowercase")]
pub enum SizeUnit {
    #[default]
    Auto,
    B,
    Kb,
    Mb,
    Gb,
    Tb,
}

impl SizeUnit {
    pub fn format(&self, bytes: u64) -> String {
        match self {
            SizeUnit::Auto => {
                for (divisor, suffix) in consts::UNITS {
                    if bytes >= divisor {
                        if divisor == 1 {
                            return format!("{}{}", bytes, suffix);
                        } else {
                            return format!("{:.1}{}", bytes as f64 / divisor as f64, suffix);
                        }
                    }
                }

                format!("0B")
            }
            SizeUnit::B => format!("{}B", bytes),
            SizeUnit::Kb => format!("{:.1}K", bytes as f64 / 1024.0),
            SizeUnit::Mb => format!("{:.1}M", bytes as f64 / 1024_f64.powi(2)),
            SizeUnit::Gb => format!("{:.1}G", bytes as f64 / 1024_f64.powi(3)),
            SizeUnit::Tb => format!("{:.1}T", bytes as f64 / 1024_f64.powi(4)),
        }
    }
}

#[derive(Debug, Clone)]
#[derive(Serialize, Deserialize)]
pub struct Pattern {
    #[serde(rename = "match")]
    pub pattern: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub icon: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub style: Option<Style>,
}

// Global rules (applied to any field)
#[derive(Debug, Clone)]
#[derive(Serialize, Deserialize)]
pub struct Rule {
    pub var: String,
    pub op: Op,
    pub val: Value,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub icon: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub style: Option<Style>,
}

// Defaults
fn default_padding() -> usize {
    2
}

fn default_icon() -> String {
    "󰈔".to_string()
}

fn default_date_format() -> String {
    "%b %d %H:%M".to_string()
}

impl Default for LsConfig {
    fn default() -> Self {
        Self {
            format: vec![
                String::from("{ :depth}{icon} {name}"),
                String::from("{perms}"),
                String::from("{size}"),
                String::from("{modified}"),
                // String::from(r"\u001B[33m-> {nlink}\u001B[0m"),
            ],

            padding: default_padding(),
            size_unit: SizeUnit::Auto,
            date_format: default_date_format(),
            icon: IconConfig::default(),
            style: StyleConfig::default(),
            pattern: Vec::new(),
            rules: Vec::new(),
        }
    }
}

impl Default for IconConfig {
    fn default() -> Self {
        Self {
            default: default_icon(),
            directory: Some("󰉋".to_string()),
            executable: Some("󰩋".to_string()),
            symlink: Some("󱅷".to_string()),
            broken_symlink: Some("󱅸".to_string()),
            ext: HashMap::new(),
            name: HashMap::new(),
            when: Vec::new(),
        }
    }
}

impl Default for StyleConfig {
    fn default() -> Self {
        Self {
            name: FieldStyle::default(),
            permissions: FieldStyle::default(),
            size: FieldStyle::default(),
            modified: FieldStyle::default(),
            created: FieldStyle::default(),
            accessed: FieldStyle::default(),
        }
    }
}

impl Parse for LsConfig {
    fn parse() -> Self {
        let Some(path) = util::config_path(ConfigKind::List) else {
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
