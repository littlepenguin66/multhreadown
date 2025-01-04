use std::error::Error as StdError;
use std::io;
use tokio::sync::AcquireError;
use tokio::task::JoinError;
use thiserror::Error;
use crate::config::ConfigError;
use url;

#[derive(Debug, Error)]
pub enum DownloadError {
    #[error("IO error: {0}")]
    Io(#[from] io::Error),

    #[error("HTTP error: {0}")]
    Reqwest(#[from] reqwest::Error),

    #[error("HTTP error for file {0}: {1} {2}")]
    HttpError(u32, u16, String),

    #[error("Network error for file {0}: {1}")]
    NetworkError(u32, String),

    #[error("Checksum mismatch for file {0}: expected {1}, got {2}")]
    ChecksumMismatch(u32, String, String),

    #[error("Invalid URL: {0}")]
    InvalidUrl(String),

    #[error("URL parse error: {0}")]
    UrlParseError(#[from] url::ParseError),

    #[error("Configuration error: {0}")]
    ConfigError(String),

    #[error("Other error: {0}")]
    Other(String),

    #[error("Task join error: {0}")]
    JoinError(#[from] JoinError),

    #[error("Semaphore acquire error: {0}")]
    AcquireError(#[from] AcquireError),
}

impl From<ConfigError> for DownloadError {
    fn from(err: ConfigError) -> Self {
        DownloadError::ConfigError(err.to_string())
    }
}

impl From<Box<dyn StdError>> for DownloadError {
    fn from(err: Box<dyn StdError>) -> Self {
        DownloadError::Other(err.to_string())
    }
}
