use criterion::{criterion_group, criterion_main, Criterion};
use multhreadown::{
    config::{Config, RetryConfig},
    downloader,
};
use tempfile::tempdir;
use std::time::Duration;

fn download_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("download");
    
    // 减少测试时间以加快基准测试
    group.sample_size(5);  // 减少样本数量
    group.measurement_time(Duration::from_secs(10));  // 减少测量时间
    group.warm_up_time(Duration::from_secs(3));  // 减少预热时间
    
    // 使用更小的测试文件
    let test_files = vec![
        // 使用单个小文件进行测试
        "https://raw.githubusercontent.com/rust-lang/rust/master/README.md",
    ];

    // 并行下载测试
    group.bench_function("parallel_download_3_workers", |b| {
        b.iter_with_setup(
            || {
                let temp_dir = tempdir().unwrap();
                let config = Config {
                    download_dir: temp_dir.path().to_path_buf(),
                    workers: 3,
                    random_order: false,
                    urls: test_files.iter().map(|&s| s.to_string()).collect(),
                    rate_limit_kb: None,
                    retry: RetryConfig {
                        max_retries: 2,  // 减少重试次数
                        initial_delay: 1,
                        max_delay: 5,    // 减少最大延迟
                        backoff_factor: 2.0,
                    },
                    concurrent_downloads: 3,
                    connection_timeout: 10,  // 减少超时时间
                };
                (config, temp_dir)
            },
            |(config, _temp_dir)| {
                let rt = tokio::runtime::Builder::new_current_thread()  // 使用单线程运行时
                    .enable_all()
                    .build()
                    .unwrap();
                rt.block_on(async {
                    let _ = tokio::time::timeout(  // 添加超时处理
                        Duration::from_secs(15),
                        downloader::download_all_files(config)
                    ).await;
                });
            },
        );
    });

    // 串行下载测试
    group.bench_function("sequential_download", |b| {
        b.iter_with_setup(
            || {
                let temp_dir = tempdir().unwrap();
                let config = Config {
                    download_dir: temp_dir.path().to_path_buf(),
                    workers: 1,
                    random_order: false,
                    urls: test_files.iter().map(|&s| s.to_string()).collect(),
                    rate_limit_kb: None,
                    retry: RetryConfig {
                        max_retries: 2,
                        initial_delay: 1,
                        max_delay: 5,
                        backoff_factor: 2.0,
                    },
                    concurrent_downloads: 1,
                    connection_timeout: 10,
                };
                (config, temp_dir)
            },
            |(config, _temp_dir)| {
                let rt = tokio::runtime::Builder::new_current_thread()
                    .enable_all()
                    .build()
                    .unwrap();
                rt.block_on(async {
                    let _ = tokio::time::timeout(
                        Duration::from_secs(15),
                        downloader::download_all_files(config)
                    ).await;
                });
            },
        );
    });

    group.finish();
}

criterion_group! {
    name = benches;
    config = Criterion::default()
        .significance_level(0.1)
        .noise_threshold(0.05)
        .configure_from_args();
    targets = download_benchmark
}
criterion_main!(benches);
