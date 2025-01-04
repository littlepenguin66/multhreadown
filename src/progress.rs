use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use std::time::Duration;
use std::sync::atomic::{AtomicUsize, AtomicU64, Ordering};
use bytesize;

pub struct GlobalProgress {
    total_files: usize,
    completed_files: AtomicUsize,
    total_bytes: AtomicU64,
    downloaded_bytes: AtomicU64,
    multi_progress: MultiProgress,
    main_progress: ProgressBar,
}

impl GlobalProgress {
    pub fn new(total_files: usize) -> Self {
        let multi = MultiProgress::new();
        let main_pb = multi.add(ProgressBar::new(total_files as u64));
        
        main_pb.set_style(
            ProgressStyle::default_bar()
                .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {msg}")
                .unwrap()
                .progress_chars("=>-"),
        );
        
        main_pb.enable_steady_tick(Duration::from_millis(500));
        
        Self {
            total_files,
            completed_files: AtomicUsize::new(0),
            total_bytes: AtomicU64::new(0),
            downloaded_bytes: AtomicU64::new(0),
            multi_progress: multi,
            main_progress: main_pb,
        }
    }

    pub fn create_progress_bar(&self, len: u64) -> ProgressBar {
        let pb = self.multi_progress.add(ProgressBar::new(len));
        pb.set_style(
            ProgressStyle::default_bar()
                .template("{spinner:.green} [{elapsed_precise}] [{wide_bar:.cyan/blue}] {bytes}/{total_bytes} ({eta}) {msg}")
                .unwrap()
                .progress_chars("=>-"),
        );
        pb.enable_steady_tick(Duration::from_millis(100));
        pb.reset_elapsed();
        pb
    }

    pub fn update_progress(&self, bytes: u64) {
        self.downloaded_bytes.fetch_add(bytes, Ordering::SeqCst);
        self.update_display();
    }

    pub fn complete_file(&self) {
        let completed = self.completed_files.fetch_add(1, Ordering::SeqCst) + 1;
        if completed >= self.total_files {
            self.main_progress.set_position(self.total_files as u64);
            self.main_progress.finish_with_message("✨ All downloads completed successfully");
            self.multi_progress.clear().ok();
        } else {
            self.update_display();
        }
    }

    pub fn set_total_bytes(&self, bytes: u64) {
        self.total_bytes.store(bytes, Ordering::SeqCst);
        self.update_display();
    }

    fn update_display(&self) {
        let completed = self.completed_files.load(Ordering::SeqCst);
        let downloaded = self.downloaded_bytes.load(Ordering::SeqCst);
        let total = self.total_bytes.load(Ordering::SeqCst);
        
        self.main_progress.set_position(completed as u64);
        
        let total_str = if total > 0 { 
            bytesize::to_string(total, true) 
        } else { 
            "?".to_string() 
        };
        
        let percentage = if self.total_files > 0 {
            (completed as f64 / self.total_files as f64 * 100.0) as u32
        } else {
            0
        };
        
        let msg = format!(
            "Progress: {}/{} files ({}%) - {}/{}",
            completed,
            self.total_files,
            percentage,
            bytesize::to_string(downloaded, true),
            total_str
        );
        self.main_progress.set_message(msg);
    }
}

impl Drop for GlobalProgress {
    fn drop(&mut self) {
        self.multi_progress.clear().ok();
        self.main_progress.finish_and_clear();
    }
}
