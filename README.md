# Multhreadown

[![Crates.io](https://img.shields.io/crates/v/multhreadown.svg)](https://crates.io/crates/multhreadown)
[![Documentation](https://docs.rs/multhreadown/badge.svg)](https://docs.rs/multhreadown)
[![License: MIT OR Apache-2.0](https://img.shields.io/crates/l/multhreadown.svg)](LICENSE)

一个功能强大的多线程下载管理器，支持断点续传、暂停恢复和速率限制。

## 特性

- 🚀 **多线程下载**：利用多线程并行下载文件的不同部分，显著提高下载速度
- ⏸️ **暂停与恢复**：支持随时暂停下载并在之后恢复，无需重新开始
- 📊 **进度跟踪**：实时显示下载进度、速度和剩余时间
- 🔄 **断点续传**：自动从上次中断的位置继续下载
- 🚦 **速率限制**：可配置的下载速率限制，避免占用过多带宽
- 🛡️ **错误处理**：健壮的错误处理和自动重试机制

## 安装

将以下内容添加到您的 `Cargo.toml` 文件中：

```toml
[dependencies]
multhreadown = "0.1.0"
```

## 快速开始

### 基本用法

```rust
use multhreadown::{Config, download_all_files, DefaultEventHandler};
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    // 创建下载配置
    let config = Config::new()
        .with_threads(4)
        .with_chunk_size(1024 * 1024)
        .with_retry_attempts(3);

    // 要下载的文件列表
    let urls = vec![
        "https://example.com/file1.zip",
        "https://example.com/file2.zip",
    ];

    // 下载目标目录
    let output_dir = "/path/to/downloads";

    // 创建事件处理器
    let event_handler = DefaultEventHandler::new();

    // 开始下载
    download_all_files(&urls, output_dir, &config, event_handler)?;

    println!("所有文件下载完成！");
    Ok(())
}
```

### 自定义事件处理

您可以实现自己的事件处理器来响应下载过程中的各种事件：

```rust
use multhreadown::{events::{DownloadEventHandler, DownloadEvent}, DownloadStats};
use std::sync::Arc;

struct MyEventHandler;

impl DownloadEventHandler for MyEventHandler {
    fn on_event(&self, event: DownloadEvent) {
        match event {
            DownloadEvent::DownloadStarted(url) => {
                println!("开始下载: {}", url);
            }
            DownloadEvent::DownloadCompleted(url) => {
                println!("下载完成: {}", url);
            }
            DownloadEvent::ProgressUpdated(url, stats) => {
                println!(
                    "{}: 已下载 {}/{} ({:.2}%), 速度: {}/s",
                    url,
                    stats.bytes_downloaded,
                    stats.total_bytes,
                    stats.progress * 100.0,
                    format_bytes(stats.download_speed as usize)
                );
            }
            // 处理其他事件...
            _ => {}
        }
    }
}

fn format_bytes(bytes: usize) -> String {
    // 实现字节格式化逻辑
    // ...
}
```

### 使用缓存管理器

```rust
use multhreadown::{CacheManager, DownloadCache};

// 创建缓存管理器
let cache_manager = CacheManager::new("/path/to/cache/directory")?;

// 保存下载状态
let download_cache = DownloadCache {
    url: "https://example.com/file.zip".to_string(),
    file_path: "/path/to/downloads/file.zip".to_string(),
    total_size: 1024000,
    downloaded_chunks: vec![(0, 512000)], // 已下载的区块
};
cache_manager.save_download_state(&download_cache)?;

// 恢复下载状态
let cached_download = cache_manager.load_download_state("https://example.com/file.zip")?;
```

## 高级配置

```rust
use multhreadown::Config;

let config = Config::new()
    .with_threads(8)                      // 设置下载线程数
    .with_chunk_size(2 * 1024 * 1024)     // 设置每个块的大小（2MB）
    .with_retry_attempts(5)               // 设置重试次数
    .with_retry_delay(std::time::Duration::from_secs(3)) // 设置重试延迟
    .with_rate_limit(Some(1024 * 1024))   // 限制下载速度为1MB/s
    .with_connect_timeout(std::time::Duration::from_secs(30)) // 设置连接超时
    .with_user_agent("MyDownloader/1.0"); // 设置用户代理
```

## 命令行界面

Multhreadown 也提供了命令行界面：

```bash
# 安装命令行工具
cargo install multhreadown

# 使用命令行下载文件
multhreadown download https://example.com/file.zip --output /path/to/downloads --threads 4

# 查看帮助
multhreadown --help
```

## 贡献

欢迎贡献！请随时提交问题或拉取请求。

## 许可证

本项目采用 MIT 。
