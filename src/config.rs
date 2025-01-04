use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::str::FromStr;
use thiserror::Error;
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetryConfig {
    pub max_retries: u32,
    pub initial_delay: u64,
    pub max_delay: u64,
    pub backoff_factor: f64,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_retries: 3,
            initial_delay: 1,
            max_delay: 30,
            backoff_factor: 2.0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub download_dir: PathBuf,
    pub workers: usize,
    pub random_order: bool,
    pub urls: Vec<String>,
    pub rate_limit_kb: Option<u64>,
    pub retry: RetryConfig,
    pub concurrent_downloads: usize,
    pub connection_timeout: u64,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            download_dir: PathBuf::from("downloads"),
            workers: 4,
            random_order: false,
            urls: Vec::new(),
            rate_limit_kb: None,
            retry: RetryConfig::default(),
            concurrent_downloads: 4,
            connection_timeout: 30,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntegrityCheck {
    pub enabled: bool,
    pub algorithm: ChecksumAlgorithm,
    pub checksums: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ChecksumAlgorithm {
    MD5,
    SHA256,
    SHA512,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DownloadFilter {
    pub include_patterns: Vec<String>,
    pub exclude_patterns: Vec<String>,
    pub min_size: Option<u64>,
    pub max_size: Option<u64>,
}

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("Invalid download directory: {0}")]
    InvalidDownloadDir(String),
    #[error("Invalid number of workers: {0}")]
    InvalidWorkers(usize),
    #[error("No download URLs provided")]
    NoUrls,
    #[error("Invalid URL format: {0}")]
    InvalidUrl(String),
}

impl Config {
    pub fn validate(&self) -> Result<(), ConfigError> {
        // Validate download directory
        if !self.download_dir.to_str().map_or(false, |s| !s.is_empty()) {
            return Err(ConfigError::InvalidDownloadDir(
                self.download_dir.to_string_lossy().to_string(),
            ));
        }
        
        // Check if download directory is writable
        if let Err(e) = std::fs::create_dir_all(&self.download_dir) {
            return Err(ConfigError::InvalidDownloadDir(format!(
                "Cannot create directory: {}",
                e
            )));
        }

        // Validate workers count
        if self.workers == 0 || self.workers > 100 {
            return Err(ConfigError::InvalidWorkers(self.workers));
        }

        // Validate URLs
        if self.urls.is_empty() {
            return Err(ConfigError::NoUrls);
        }
        
        // Validate URL count matches expected file indices
        if self.urls.len() > 163 {
            return Err(ConfigError::InvalidUrl(format!(
                "Too many URLs (max 163), got {}",
                self.urls.len()
            )));
        }

        for (index, url) in self.urls.iter().enumerate() {
            if !url.starts_with("http://") && !url.starts_with("https://") {
                return Err(ConfigError::InvalidUrl(format!(
                    "Invalid URL format at index {}: {}",
                    index, url
                )));
            }
        }

        Ok(())
    }

    pub fn from_file(path: &PathBuf) -> Result<Self, Box<dyn std::error::Error>> {
        let content = std::fs::read_to_string(path)?;
        let config: Config = toml::from_str(&content)?;
        Ok(config)
    }
}

impl FromStr for Config {
    type Err = toml::de::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        toml::from_str(s)
    }
}
