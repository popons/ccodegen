use std::path::PathBuf;
use thiserror::Error;

/// Errors that can occur during code generation
#[derive(Error, Debug)]
pub enum CodeGenError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Failed to capture user section from file: {path}")]
    CaptureFailed {
        path: PathBuf,
        #[source]
        source: anyhow::Error,
    },

    #[error("Invalid user section: {0}")]
    InvalidSection(String),

    #[error("Nested user section at line {line}: already in section '{section}'")]
    NestedSection { line: usize, section: String },

    #[error("Mismatched user section at line {line}: expected '{expected}', found '{found}'")]
    MismatchedSection {
        line: usize,
        expected: String,
        found: String,
    },

    #[error("Unclosed user section at end of file: '{0}'")]
    UnclosedSection(String),

    #[error("Unknown user section: '{0}'")]
    UnknownSection(String),

    #[error("Regex error: {0}")]
    Regex(#[from] regex::Error),

    #[error("Other error: {0}")]
    Other(#[from] anyhow::Error),
}

/// Result type for code generation operations
pub type Result<T> = std::result::Result<T, CodeGenError>;
