use std::{collections::HashSet, str::FromStr};
use serde::Deserialize;
use crate::{err::PlsError, util};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ListVariable {
    Name,
    Path,
    Size,
    Permissions,
    Created,
    Modified,
    Accessed,
}

impl FromStr for ListVariable {
    type Err = ();

    fn from_str(input: &str) -> Result<ListVariable, Self::Err> {
        match input {
            "name" => Ok(ListVariable::Name),
            "path" => Ok(ListVariable::Path),
            "size" => Ok(ListVariable::Size),
            "permissions" => Ok(ListVariable::Permissions),
            "created" => Ok(ListVariable::Created),
            "modified" => Ok(ListVariable::Modified),
            "accessed" => Ok(ListVariable::Accessed),
            _ => Err(()),
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct ListConfig {
    #[serde(default = "ListConfig::default_format")]
    pub format: Vec<String>,
}

impl Default for ListConfig {
    fn default() -> Self {
        Self {
            format: Self::default_format(),
        }
    }
}

impl ListConfig {
    pub fn default_format() -> Vec<String> {
        vec![
            String::from("{name}"),
            String::from("{size}"),
            String::from("{modified}"),
        ]
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
