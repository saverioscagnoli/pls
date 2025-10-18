use thiserror::Error;

#[derive(Debug, Error)]
pub enum PlsError {
    #[error("Configuration file not found")]
    ConfigNotFound,

    #[error("Configuration parsing error: {0}")]
    ParsingError(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}
