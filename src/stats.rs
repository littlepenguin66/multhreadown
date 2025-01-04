use std::time::SystemTime;
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};

#[derive(Debug)]
pub struct DownloadStats {
    pub start_time: SystemTime,
    pub end_time: Option<SystemTime>,
    pub total_bytes: AtomicU64,
    pub successful_downloads: AtomicUsize,
    pub failed_downloads: AtomicUsize,
    pub retry_count: AtomicUsize,
    pub average_speed: AtomicU64,
}

impl Default for DownloadStats {
    fn default() -> Self {
        Self {
            start_time: SystemTime::now(),
            end_time: None,
            total_bytes: AtomicU64::new(0),
            successful_downloads: AtomicUsize::new(0),
            failed_downloads: AtomicUsize::new(0),
            retry_count: AtomicUsize::new(0),
            average_speed: AtomicU64::new(0),
        }
    }
}

impl DownloadStats {
    pub fn record_success(&self, bytes: u64) {
        self.successful_downloads.fetch_add(1, Ordering::SeqCst);
        self.total_bytes.fetch_add(bytes, Ordering::SeqCst);
        self.update_speed();
    }

    pub fn record_failure(&self) {
        self.failed_downloads.fetch_add(1, Ordering::SeqCst);
    }

    pub fn record_retry(&self) {
        self.retry_count.fetch_add(1, Ordering::SeqCst);
    }

    fn update_speed(&self) {
        if let Ok(duration) = SystemTime::now().duration_since(self.start_time) {
            let seconds = duration.as_secs().max(1);
            let bytes = self.total_bytes.load(Ordering::SeqCst);
            let speed = bytes / seconds;
            self.average_speed.store(speed, Ordering::SeqCst);
        }
    }
} 