mod config;
mod downloader;
mod error;
mod progress;
mod utils;

use clap::Parser;
use config::{Config, RetryConfig};
use downloader::download_all_files;
use error::DownloadError;
use log::info;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(name = "multhreadown")]
#[command(about = "A multi-threaded download tool", long_about = None)]
struct Cli {
    /// Path to config file
    #[arg(short, long, value_name = "FILE")]
    config: Option<PathBuf>,

    /// Number of worker threads
    #[arg(short, long, default_value_t = 4)]
    workers: usize,

    /// Download directory
    #[arg(short, long, value_name = "DIR")]
    download_dir: PathBuf,

    /// Enable random download order
    #[arg(short, long)]
    random_order: bool,

    /// Verbose output
    #[arg(short, long)]
    verbose: bool,

    /// URLs to download
    #[arg(short = 'u', long = "urls", value_name = "URLS", num_args = 1.., required = true)]
    urls: Vec<String>,
}

#[tokio::main]
async fn main() -> Result<(), DownloadError> {
    env_logger::Builder::from_default_env()
        .filter_level(log::LevelFilter::Info)
        .init();

    let cli = Cli::parse();

    if cli.verbose {
        env_logger::Builder::from_default_env()
            .filter_level(log::LevelFilter::Debug)
            .init();
    }

    let config = match cli.config {
        Some(path) => Config::from_file(&path)?,
        None => Config {
            download_dir: cli.download_dir,
            workers: cli.workers,
            random_order: cli.random_order,
            urls: cli.urls,
            rate_limit_kb: None,
            retry: RetryConfig::default(),
            concurrent_downloads: cli.workers,
            connection_timeout: 30,
        },
    };

    config.validate()?;
    
    info!("Starting download process with {} workers", config.workers);
    info!("Download directory: {}", config.download_dir.display());
    info!("Random order: {}", config.random_order);

    download_all_files(config).await?;

    info!("Download process completed successfully");
    Ok(())
}
