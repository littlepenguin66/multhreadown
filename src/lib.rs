//! Multhreadown - A multi-threaded download manager with resume capability
//!
//! This crate provides functionality for downloading files using multiple threads
//! with support for pause, resume, and rate limiting.

// Export public modules
pub mod cache;
pub mod cli;
pub mod config;
pub mod downloader;
pub mod error;
pub mod events;
pub mod progress;
pub mod stats;
pub mod utils;

pub use cache::{CacheManager, DownloadCache};
pub use config::Config;
pub use downloader::download_all_files;
pub use error::DownloadError;
pub use events::{DefaultEventHandler, DownloadEventHandler};
pub use progress::GlobalProgress;
pub use stats::DownloadStats;
