use crate::{
    style::{ConditionalRule, FieldStyle, Op, VariableStyle},
    util,
};
use figura::Value;
use serde::Deserialize;
use std::{
    collections::{HashMap, HashSet},
    str::FromStr,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ListVariable {
    Name,
    Extension,
    Path,
    Kind,
    Icon,
    Depth,
    Size,
    Permissions,
    Created,
    Modified,
    Accessed,
    Owner,
    Group,
    NLink,
}

impl FromStr for ListVariable {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "name" => Ok(ListVariable::Name),
            "extension" => Ok(ListVariable::Extension),
            "path" => Ok(ListVariable::Path),
            "kind" => Ok(ListVariable::Kind),
            "icon" => Ok(ListVariable::Icon),
            "depth" => Ok(ListVariable::Depth),
            "size" => Ok(ListVariable::Size),
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

#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct ListConfig {
    pub format: Vec<String>,
    pub padding: usize,
    pub style: HashMap<String, FieldStyle>,
    pub size_unit: SizeUnit,
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
            style: HashMap::new(),
            size_unit: SizeUnit::Auto,
        }
    }
}

impl ListConfig {
    /// Returns a set of format variables parsed from the format strings.
    /// This is used so that we can know which metadata to fetch for each file.
    pub fn format_variables(&self) -> HashSet<ListVariable> {
        let mut stripped = String::new();

        for t in &self.format {
            stripped.push_str(&util::keep_letters_whitespace(&t));
            stripped.push(' ');
        }

        stripped
            .split_whitespace()
            .filter_map(|s| ListVariable::from_str(s).ok())
            .collect()
    }

    pub fn apply_field_style(
        &self,
        field_name: &str,
        value: &str,
        ctx: &HashMap<&'static str, Value>,
    ) -> String {
        if let Some(field_style) = self.style.get(field_name) {
            // Evaluate conditions
            for rule in &field_style.conditions {
                if self.evaluate_condition(rule, ctx) {
                    return rule.style.apply(value);
                }
            }

            // Fall back to default if no condition matches
            if let Some(default_style) = &field_style.default {
                return default_style.apply(value);
            }
        }

        // No styling applied
        value.to_string()
    }

    /// Evaluate a conditional rule
    fn evaluate_condition(
        &self,
        rule: &ConditionalRule,
        ctx: &HashMap<&'static str, Value>,
    ) -> bool {
        let var_value = match ctx.get(rule.variable.as_str()) {
            Some(v) => v.to_string(),
            None => return false,
        };

        match rule.op {
            Op::Equal => var_value == rule.value,
            Op::Greater => {
                if let (Ok(a), Ok(b)) = (var_value.parse::<i64>(), rule.value.parse::<i64>()) {
                    a > b
                } else {
                    var_value > rule.value
                }
            }
            Op::Less => {
                if let (Ok(a), Ok(b)) = (var_value.parse::<i64>(), rule.value.parse::<i64>()) {
                    a < b
                } else {
                    var_value < rule.value
                }
            }
            Op::GreaterEqual => {
                if let (Ok(a), Ok(b)) = (var_value.parse::<i64>(), rule.value.parse::<i64>()) {
                    a >= b
                } else {
                    var_value >= rule.value
                }
            }
            Op::LessEqual => {
                if let (Ok(a), Ok(b)) = (var_value.parse::<i64>(), rule.value.parse::<i64>()) {
                    a <= b
                } else {
                    var_value <= rule.value
                }
            }
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
