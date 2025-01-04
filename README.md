# Multhreadown

一个功能强大的多线程下载工具，使用 Rust 编写，专注于性能和可靠性。

## ✨ 功能特性

### 核心功能

- 🚀 多线程并发下载
- 🔄 断点续传支持
- 📊 实时进度显示
- 🔍 文件完整性校验
- 🔄 智能重试机制

### 高级特性

- 🎮 交互式命令控制
- 📈 下载统计和分析
- 💾 智能缓存管理
- 🎯 灵活的过滤规则
- 🎚️ 下载速度限制

## 🚀 快速开始

### 安装

```bash
# 从源码安装
git clone https://github.com/little_penguin66/multhreadown.git
cd multhreadown
cargo install --path .
```

### 基本使用

```bash
# 简单下载
multhreadown --download-dir ./downloads --urls "https://example.com/file1.zip"

# 多文件下载
multhreadown --download-dir ./downloads --workers 4 \
    --urls "https://example.com/file1.zip" "https://example.com/file2.tar.gz"

# 使用配置文件
multhreadown --config config.toml
```

## ⚙️ 配置选项

### 命令行参数

```bash
multhreadown [OPTIONS]

Options:
  -c, --config <FILE>       配置文件路径
  -d, --download-dir <DIR>  下载目录 [required]
  -w, --workers <NUM>       工作线程数 [default: 4]
  -r, --random-order       随机顺序下载
  -v, --verbose            详细输出
  -u, --urls <URLS>...     下载链接列表
  -h, --help               显示帮助信息
```

### 配置文件 (config.toml)

```toml
# 基本配置
download_dir = "./downloads"
workers = 4
random_order = true
rate_limit_kb = 1024  # 1MB/s
concurrent_downloads = 4
connection_timeout = 30

# 重试配置
[retry]
max_retries = 3
initial_delay = 1
max_delay = 30
backoff_factor = 2.0

# 完整性检查
[integrity_check]
enabled = true
algorithm = "MD5"

# 下载过滤器
[filter]
include_patterns = ["*.zip", "*.tar.gz"]
exclude_patterns = ["*.exe"]
min_size = 1024  # 1KB
max_size = 1073741824  # 1GB
```

## 🎮 交互式命令

下载过程中支持以下命令：

- `pause`: 暂停下载
- `resume`: 恢复下载
- `cancel`: 取消下载
- `progress`: 显示当前进度
- `limit <KB>`: 设置下载速度限制

## 💻 编程接口

```rust
use multhreadown::{config::Config, downloader};

#[tokio::main]
async fn main() {
    let config = Config {
        download_dir: PathBuf::from("downloads"),
        workers: 4,
        urls: vec!["https://example.com/file1.zip".to_string()],
        // ... 其他配置
    };

    if let Err(e) = downloader::download_all_files(config).await {
        eprintln!("Download failed: {}", e);
    }
}
```

## 🧪 开发

### 运行测试

```bash
# 运行所有测试
cargo test

# 运行特定测试
cargo test test_basic_download

# 运行带输出的测试
cargo test -- --nocapture
```

### 运行基准测试

```bash
# 运行所有基准测试
cargo bench

# 运行特定基准测试
cargo bench -- parallel_download_3_workers

# 生成HTML报告
cargo bench --bench download_bench -- --baseline main

# 比较与基准的差异
cargo bench --bench download_bench -- --baseline main
```

基准测试包括：

- 并行下载测试（3 个工作线程）
- 串行下载测试（单线程）

测试配置：

- 样本大小：10
- 测量时间：20 秒
- 预热时间：5 秒
- 显著性水平：0.1
- 噪声阈值：0.05

测试文件：

- README.md（小文件）
- COPYRIGHT（小文件）
- LICENSE-MIT（小文件）

基准测试结果将保存在 `target/criterion` 目录下，包含详细的性能报告和图表。

## 📊 性能

- 支持并发下载，显著提升下载速度
  - 并行下载比串行下载平均快 2-3 倍
  - 自动调整并发数以优化性能
- 智能的内存管理
  - 使用异步 I/O 减少内存占用
  - 自动清理临时文件
- 高效的资源利用
  - 支持断点续传减少带宽浪费
  - 智能重试机制处理网络波动

## 🤝 贡献

欢迎提交 Issue 和 Pull Request！

1. Fork 本仓库
2. 创建你的特性分支 (`git checkout -b feature/AmazingFeature`)
3. 提交你的改动 (`git commit -m 'Add some AmazingFeature'`)
4. 推送到分支 (`git push origin feature/AmazingFeature`)
5. 开启一个 Pull Request

## 📝 更新日志

### v0.1.0 (2025-01-04)

- ✨ 初始版本发布
- 🚀 支持多线程下载
- 💾 添加断点续传
- 📊 添加下载统计
- 🎮 添加交互式控制

## 📄 许可证

本项目采用 MIT 许可证 - 查看 [LICENSE](LICENSE) 文件了解详情

## 🙏 致谢

- [tokio](https://tokio.rs/) - 异步运行时
- [reqwest](https://docs.rs/reqwest/) - HTTP 客户端
- [indicatif](https://docs.rs/indicatif/) - 进度显示
