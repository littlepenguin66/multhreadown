use multhreadown::{
    cache::CacheManager,
    cli::{Command, DownloadStatus, InteractiveMode},
    config::{Config, RetryConfig},
    downloader,
    events::DefaultEventHandler,
    stats::DownloadStats,
};
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::mpsc;

#[tokio::main]
async fn main() {
    // 初始化日志
    env_logger::init();

    // 创建基本配置
    let config = Config {
        download_dir: PathBuf::from("downloads"),
        workers: 4,
        random_order: false,
        urls: vec![
            // 使用稳定的小文件作为示例
            "https://raw.githubusercontent.com/rust-lang/rust/master/README.md".to_string(),
            "https://raw.githubusercontent.com/rust-lang/rust/master/LICENSE-MIT".to_string(),
            "https://raw.githubusercontent.com/rust-lang/rust/master/COPYRIGHT".to_string(),
        ],
        rate_limit_kb: Some(1024),  // 限速 1MB/s
        retry: RetryConfig::default(),
        concurrent_downloads: 4,
        connection_timeout: 30,
    };

    // 初始化缓存管理器
    let cache_manager = CacheManager::new("./cache").await.unwrap();
    
    // 初始化下载统计
    let stats = Arc::new(DownloadStats::default());
    let stats_clone = stats.clone();
    
    // 创建事件处理器
    let event_handler = Arc::new(DefaultEventHandler);

    // 创建交互模式
    let (mut interactive_mode, status_tx, command_rx) = InteractiveMode::new();

    // 在单独的任务中运行交互模式
    let interactive_handle = tokio::spawn(async move {
        interactive_mode.run().await;
    });

    // 创建命令处理通道
    let (cmd_tx, mut cmd_rx) = mpsc::channel(10);
    let cmd_tx_clone = cmd_tx.clone();

    // 处理用户输入的命令
    tokio::spawn(async move {
        while let Some(cmd) = command_rx.recv().await {
            match cmd {
                Command::Pause => println!("下载已暂停"),
                Command::Resume => println!("下载已恢复"),
                Command::Cancel => {
                    println!("下载已取消");
                    cmd_tx.send(Command::Cancel).await.ok();
                }
                Command::ShowProgress => {
                    let total = stats_clone.total_bytes.load(std::sync::atomic::Ordering::SeqCst);
                    let success = stats_clone.successful_downloads.load(std::sync::atomic::Ordering::SeqCst);
                    let failed = stats_clone.failed_downloads.load(std::sync::atomic::Ordering::SeqCst);
                    println!("下载进度：");
                    println!("总大小：{} bytes", total);
                    println!("成功：{} 个文件", success);
                    println!("失败：{} 个文件", failed);
                }
                _ => {}
            }
        }
    });

    // 启动下载
    let download_handle = tokio::spawn(async move {
        match downloader::download_all_files(config).await {
            Ok(_) => {
                status_tx.send(DownloadStatus::Completed).await.ok();
                println!("所有文件下载完成！");
                
                // 显示最终统计信息
                println!("\n下载统计：");
                println!("总下载量：{} bytes", stats.total_bytes.load(std::sync::atomic::Ordering::SeqCst));
                println!("成功数量：{}", stats.successful_downloads.load(std::sync::atomic::Ordering::SeqCst));
                println!("失败数量：{}", stats.failed_downloads.load(std::sync::atomic::Ordering::SeqCst));
                println!("重试次数：{}", stats.retry_count.load(std::sync::atomic::Ordering::SeqCst));
                
                // 保存缓存
                if let Err(e) = cache_manager.save().await {
                    eprintln!("缓存保存失败: {}", e);
                }
            }
            Err(e) => {
                eprintln!("下载失败: {}", e);
                status_tx.send(DownloadStatus::Failed(e.to_string())).await.ok();
            }
        }
    });

    // 等待下载完成或用户取消
    tokio::select! {
        _ = interactive_handle => {
            println!("交互模式已关闭");
        }
        _ = download_handle => {
            println!("下载任务已完成");
        }
        Some(Command::Cancel) = cmd_rx.recv() => {
            println!("正在取消下载...");
            cmd_tx_clone.send(Command::Cancel).await.ok();
        }
        _ = tokio::time::sleep(std::time::Duration::from_secs(300)) => {
            println!("下载超时（5分钟）");
        }
    }
}
