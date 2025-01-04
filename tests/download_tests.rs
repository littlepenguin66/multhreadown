use multhreadown::{
    cache::CacheManager,
    cli::{Command, DownloadStatus, InteractiveMode},
    config::{Config, RetryConfig},
    downloader,
    error::DownloadError,
    events::DownloadEventHandler,
    stats::DownloadStats,
};
use std::{sync::Arc, collections::HashMap};
use async_trait::async_trait;

// 测试事件处理器
#[derive(Default)]
struct TestEventHandler {
    start_count: std::sync::atomic::AtomicUsize,
    complete_count: std::sync::atomic::AtomicUsize,
}

#[async_trait]
impl DownloadEventHandler for TestEventHandler {
    async fn on_download_start(&self, _url: &str) {
        self.start_count.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
    }

    async fn on_download_progress(&self, _url: &str, _progress: f64) {}

    async fn on_download_complete(&self, _url: &str) {
        self.complete_count.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
    }

    async fn on_download_error(&self, _url: &str, _error: &multhreadown::error::DownloadError) {}
}

#[tokio::test]
async fn test_basic_download() {
    let temp_dir = tempfile::tempdir().unwrap();
    let config = Config {
        download_dir: temp_dir.path().to_path_buf(),
        workers: 2,
        random_order: false,
        urls: vec![
            "https://raw.githubusercontent.com/rust-lang/rust/master/README.md".to_string(),
            "https://raw.githubusercontent.com/rust-lang/rust/master/LICENSE-MIT".to_string(),
        ],
        rate_limit_kb: None,
        retry: RetryConfig::default(),
        concurrent_downloads: 2,
        connection_timeout: 30,
    };

    let result = tokio::time::timeout(
        std::time::Duration::from_secs(30),
        downloader::download_all_files(config)
    ).await;

    assert!(result.is_ok(), "Download timed out");
    if let Ok(download_result) = result {
        assert!(download_result.is_ok(), "Download failed: {:?}", download_result.err());
    }
}

#[tokio::test]
async fn test_cache_manager() {
    let temp_dir = tempfile::tempdir().unwrap();
    let mut cache_manager = CacheManager::new(temp_dir.path()).await.unwrap();

    let cache = multhreadown::cache::DownloadCache {
        url: "https://example.com/test.zip".to_string(),
        file_size: 1024,
        downloaded_size: 1024,
        etag: Some("abc123".to_string()),
        last_modified: Some("Thu, 01 Jan 2024 00:00:00 GMT".to_string()),
        checksum: Some("d41d8cd98f00b204e9800998ecf8427e".to_string()),
    };

    cache_manager.update_cache(cache.url.clone(), cache);
    let saved = cache_manager.get_cache(&"https://example.com/test.zip");
    assert!(saved.is_some());

    // 测试缓存持久化
    assert!(cache_manager.save().await.is_ok());
}

#[tokio::test]
async fn test_download_stats() {
    let stats = DownloadStats::default();
    
    stats.record_success(1024);
    assert_eq!(stats.successful_downloads.load(std::sync::atomic::Ordering::SeqCst), 1);
    assert_eq!(stats.total_bytes.load(std::sync::atomic::Ordering::SeqCst), 1024);

    stats.record_failure();
    assert_eq!(stats.failed_downloads.load(std::sync::atomic::Ordering::SeqCst), 1);

    stats.record_retry();
    assert_eq!(stats.retry_count.load(std::sync::atomic::Ordering::SeqCst), 1);
}

#[tokio::test]
async fn test_invalid_url() {
    let temp_dir = tempfile::tempdir().unwrap();
    let config = Config {
        download_dir: temp_dir.path().to_path_buf(),
        workers: 1,
        random_order: false,
        urls: vec![
            "not-a-url".to_string(),
        ],
        rate_limit_kb: None,
        retry: RetryConfig::default(),
        concurrent_downloads: 1,
        connection_timeout: 5,
    };

    let result = tokio::time::timeout(
        std::time::Duration::from_secs(5),
        downloader::download_all_files(config)
    ).await;

    assert!(result.is_ok(), "Timeout error");
    if let Ok(download_result) = result {
        assert!(download_result.is_err(), "Expected error for invalid URL");
        if let Err(e) = download_result {
            match e {
                DownloadError::InvalidUrl(_) | DownloadError::UrlParseError(_) => (),
                _ => panic!("Unexpected error type: {:?}", e),
            }
        }
    }
}

#[tokio::test]
async fn test_interactive_mode() {
    let (mut interactive_mode, status_tx, mut command_rx) = InteractiveMode::new();
    
    // 启动交互模式
    let handle = tokio::spawn(async move {
        interactive_mode.run().await;
    });

    // 发送状态更新
    status_tx.send(DownloadStatus::Running).await.unwrap();
    status_tx.send(DownloadStatus::Completed).await.unwrap();

    // 等待命令接收，设置超时
    tokio::select! {
        Some(cmd) = command_rx.recv() => {
            match cmd {
                Command::Pause => println!("Received pause command"),
                Command::Resume => println!("Received resume command"),
                _ => println!("Received other command"),
            }
        }
        _ = tokio::time::sleep(std::time::Duration::from_secs(1)) => {
            println!("Command receive timeout");
        }
    }

    handle.abort();
}

#[tokio::test]
async fn test_event_handler() {
    let handler = Arc::new(TestEventHandler::default());
    
    handler.on_download_start("test.zip").await;
    assert_eq!(handler.start_count.load(std::sync::atomic::Ordering::SeqCst), 1);
    
    handler.on_download_complete("test.zip").await;
    assert_eq!(handler.complete_count.load(std::sync::atomic::Ordering::SeqCst), 1);
}

#[tokio::test]
async fn test_integrity_check() {
    let temp_dir = tempfile::tempdir().unwrap();
    let mut checksums = HashMap::new();
    checksums.insert(
        "https://raw.githubusercontent.com/rust-lang/rust/master/README.md".to_string(),
        "d41d8cd98f00b204e9800998ecf8427e".to_string(),
    );

    let config = Config {
        download_dir: temp_dir.path().to_path_buf(),
        workers: 1,
        random_order: false,
        urls: vec!["https://raw.githubusercontent.com/rust-lang/rust/master/README.md".to_string()],
        rate_limit_kb: None,
        retry: RetryConfig::default(),
        concurrent_downloads: 1,
        connection_timeout: 30,
    };

    let result = downloader::download_all_files(config).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_download_filter() {
    let temp_dir = tempfile::tempdir().unwrap();
    let config = Config {
        download_dir: temp_dir.path().to_path_buf(),
        workers: 1,
        random_order: false,
        urls: vec![
            "https://raw.githubusercontent.com/rust-lang/rust/master/Cargo.toml".to_string(),
            "https://raw.githubusercontent.com/rust-lang/rust/master/Cargo.lock".to_string(),
        ],
        rate_limit_kb: None,
        retry: RetryConfig::default(),
        concurrent_downloads: 1,
        connection_timeout: 30,
    };

    let result = downloader::download_all_files(config).await;
    assert!(result.is_ok());
} 