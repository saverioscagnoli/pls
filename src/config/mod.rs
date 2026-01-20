mod list;

pub use list::LsConfig;

use std::fmt;

#[derive(Debug, Clone, Copy)]
pub enum ConfigKind {
    List,
    Find,
}

impl fmt::Display for ConfigKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::List => write!(f, "list"),
            Self::Find => write!(f, "find"),
        }
    }
}

pub trait Parse: Default + Sized {
    /// If the parsing of a configuration file fails,
    /// it should return a default one with an error message,
    /// hence only the `Self` return type
    fn parse() -> Self;
}
