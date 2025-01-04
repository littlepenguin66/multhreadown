use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use tokio::fs;

#[derive(Debug, Serialize, Deserialize)]
pub struct DownloadCache {
    pub url: String,
    pub file_size: u64,
    pub downloaded_size: u64,
    pub etag: Option<String>,
    pub last_modified: Option<String>,
    pub checksum: Option<String>,
}

pub struct CacheManager {
    cache_dir: PathBuf,
    cache: HashMap<String, DownloadCache>,
}

impl CacheManager {
    pub async fn new(cache_dir: impl AsRef<Path>) -> std::io::Result<Self> {
        let cache_dir = cache_dir.as_ref().to_owned();
        fs::create_dir_all(&cache_dir).await?;
        
        let cache_file = cache_dir.join("download_cache.json");
        let cache = if cache_file.exists() {
            let content = fs::read_to_string(&cache_file).await?;
            serde_json::from_str(&content).unwrap_or_default()
        } else {
            HashMap::new()
        };

        Ok(Self { cache_dir, cache })
    }

    pub fn get_cache(&self, url: &str) -> Option<&DownloadCache> {
        self.cache.get(url)
    }

    pub fn update_cache(&mut self, url: String, cache: DownloadCache) {
        self.cache.insert(url, cache);
    }

    pub async fn save(&self) -> std::io::Result<()> {
        let cache_file = self.cache_dir.join("download_cache.json");
        let content = serde_json::to_string_pretty(&self.cache)?;
        fs::write(cache_file, content).await
    }
} 