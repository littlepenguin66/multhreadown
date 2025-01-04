use crate::error::DownloadError;
use async_trait::async_trait;

#[async_trait]
pub trait DownloadEventHandler: Send + Sync {
    async fn on_download_start(&self, url: &str);
    async fn on_download_progress(&self, url: &str, progress: f64);
    async fn on_download_complete(&self, url: &str);
    async fn on_download_error(&self, url: &str, error: &DownloadError);
}

// 添加一个默认实现
pub struct DefaultEventHandler;

#[async_trait]
impl DownloadEventHandler for DefaultEventHandler {
    async fn on_download_start(&self, url: &str) {
        log::info!("Started downloading: {}", url);
    }

    async fn on_download_progress(&self, url: &str, progress: f64) {
        log::debug!("Download progress for {}: {:.1}%", url, progress * 100.0);
    }

    async fn on_download_complete(&self, url: &str) {
        log::info!("Completed downloading: {}", url);
    }

    async fn on_download_error(&self, url: &str, error: &DownloadError) {
        log::error!("Error downloading {}: {}", url, error);
    }
} 