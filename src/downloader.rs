use crate::config::Config;
use crate::error::DownloadError;
use crate::progress::GlobalProgress;
use crate::utils::calculate_md5;
use futures_util::StreamExt;
use rand::seq::SliceRandom;
use reqwest::Client;
use std::path::Path;
use std::sync::Arc;
use tokio::fs::File;
use tokio::io::AsyncWriteExt;
use tokio::sync::Semaphore;

pub async fn download_all_files(config: Config) -> Result<(), DownloadError> {
    let client = Client::new();
    let semaphore = Arc::new(Semaphore::new(config.workers));
    let mut handles = vec![];
    
    let global_progress = Arc::new(GlobalProgress::new(config.urls.len()));
    let total_size: u64 = 0;
    global_progress.set_total_bytes(total_size);

    let mut file_indices: Vec<usize> = (0..config.urls.len()).collect();
    if config.random_order {
        file_indices.as_mut_slice().shuffle(&mut rand::thread_rng());
    }

    let mut errors = Vec::new();  // 收集所有错误
    for index in file_indices {
        let permit = semaphore.clone().acquire_owned().await?;
        let client = client.clone();
        let config = config.clone();
        let global_progress = global_progress.clone();

        let handle = tokio::spawn(async move {
            let _permit = permit;
            let result = download_file(&client, index as u32, &config, &global_progress).await;
            if let Err(e) = result {
                log::error!("Error downloading file {}: {}", index, e);
                return Err(e);
            }
            global_progress.complete_file();
            Ok(())
        });
        handles.push(handle);
    }

    // 等待所有下载完成并收集错误
    for handle in handles {
        if let Err(e) = handle.await? {
            errors.push(e);
        }
    }

    // 如果有任何错误，返回第一个错误
    if let Some(first_error) = errors.into_iter().next() {
        return Err(first_error);
    }

    Ok(())
}

async fn download_file(
    client: &Client, 
    file_index: u32, 
    config: &Config,
    global_progress: &GlobalProgress,
) -> Result<(), DownloadError> {
    let max_retries = 5;
    let mut retry_count = 0;
    
    let file_url = config.urls.get(file_index as usize)
        .ok_or_else(|| DownloadError::InvalidUrl(format!("No URL found for index {}", file_index)))?;
        
    // 加强 URL 验证
    if !file_url.starts_with("http://") && !file_url.starts_with("https://") {
        return Err(DownloadError::InvalidUrl(format!("Invalid URL scheme: {}", file_url)));
    }

    // 验证 URL 格式
    if let Err(e) = reqwest::Url::parse(file_url) {
        return Err(DownloadError::InvalidUrl(format!("Invalid URL format: {}", e)));
    }

    let file_name = file_url
        .split('/')
        .last()
        .map(|s| s.to_string())
        .unwrap_or_else(|| format!("file_{}", file_index));
    let file_path = Path::new(&config.download_dir).join(&file_name);
    
    if let Some(parent) = file_path.parent() {
        tokio::fs::create_dir_all(parent).await?;
    }

    let progress_bar = global_progress.create_progress_bar(0);
    progress_bar.set_message(format!("Downloading {}", file_name));

    // 检查是否存在部分下载的文件
    let mut downloaded_size = 0u64;
    if file_path.exists() {
        downloaded_size = file_path.metadata()?.len();
    }

    loop {
        // 创建请求构建器
        let mut request = client.get(file_url);
        
        // 如果有已下载的部分，添加 Range 头
        if downloaded_size > 0 {
            request = request.header("Range", format!("bytes={}-", downloaded_size));
        }

        let response = match request.send().await {
            Ok(res) => {
                if !res.status().is_success() {
                    if retry_count >= max_retries {
                        return Err(DownloadError::HttpError(
                            file_index,
                            res.status().as_u16(),
                            res.status().to_string(),
                        ));
                    }
                    retry_count += 1;
                    tokio::time::sleep(tokio::time::Duration::from_secs(2u64.pow(retry_count))).await;
                    continue;
                }
                res
            },
            Err(e) => {
                if retry_count >= max_retries {
                    return Err(DownloadError::NetworkError(file_index, e.to_string()));
                }
                retry_count += 1;
                tokio::time::sleep(tokio::time::Duration::from_secs(2u64.pow(retry_count))).await;
                continue;
            }
        };

        let total_size = response.content_length().unwrap_or(0);
        progress_bar.set_length(total_size);

        let mut file = if downloaded_size > 0 {
            tokio::fs::OpenOptions::new()
                .append(true)
                .open(&file_path)
                .await?
        } else {
            File::create(&file_path).await?
        };

        let mut stream = response.bytes_stream();
        let mut downloaded = downloaded_size;
        
        while let Some(chunk) = stream.next().await {
            let chunk = chunk?;
            file.write_all(&chunk).await?;
            downloaded += chunk.len() as u64;
            progress_bar.inc(chunk.len() as u64);
            global_progress.update_progress(chunk.len() as u64);
            
            if downloaded > 0 && total_size > 0 {
                let percent = (downloaded as f64 / total_size as f64 * 100.0) as u32;
                progress_bar.set_message(format!("Downloading {} ({}%)", file_name, percent));
            }
        }

        if let Some(expected_md5) = get_expected_md5(file_index) {
            let actual_md5 = calculate_md5(&file_path)?;
            if expected_md5 != actual_md5 {
                return Err(DownloadError::ChecksumMismatch(
                    file_index,
                    expected_md5,
                    actual_md5,
                ));
            }
        }

        progress_bar.finish_with_message(format!("Downloaded {}", file_name));
        return Ok(());
    }
}

fn get_expected_md5(_file_index: u32) -> Option<String> {
    None
}
