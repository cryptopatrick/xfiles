//! Error types for xfiles

use thiserror::Error;

/// Result type alias for xfiles operations
pub type Result<T> = std::result::Result<T, XFilesError>;

/// Main error type for xfiles
#[derive(Error, Debug)]
pub enum XFilesError {
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Invalid encoding: {0}")]
    InvalidEncoding(String),

    #[error("Commit not found: {0}")]
    CommitNotFound(String),

    #[error("File not found: {0}")]
    FileNotFound(String),

    #[error("Invalid path: {0}")]
    InvalidPath(String),

    #[error("Rate limit exceeded")]
    RateLimitExceeded,

    #[error("Twitter API error: {0}")]
    TwitterApi(String),

    #[error("Content too large: {0} bytes")]
    ContentTooLarge(usize),

    #[error("Hash mismatch: expected {expected}, got {actual}")]
    HashMismatch { expected: String, actual: String },

    #[error("Merge conflict")]
    MergeConflict,

    #[error("{0}")]
    Other(String),
}
