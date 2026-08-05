use thiserror::Error;
use std::io;

#[derive(Error, Debug)]
pub enum ConversionError {
    #[error("IO error: {0}")]
    IoError(#[from] io::Error),

    #[error("Invalid configuration: {0}")]
    InvalidConfig(String),

    #[error("Conversion failed for {file}: {reason}")]
    ConversionFailed { file: String, reason: String },

    #[error("Unsupported format: {0}")]
    UnsupportedFormat(String),

    #[error("File not found: {0}")]
    FileNotFound(String),

    #[error("Invalid pattern: {0}")]
    InvalidPattern(String),

    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),

    #[error("Unknown error: {0}")]
    Unknown(String),
}

pub type ConversionResult<T> = Result<T, ConversionError>;
