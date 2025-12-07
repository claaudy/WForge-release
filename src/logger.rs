use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use chrono::Local;

#[derive(Clone)]
pub struct LogEntry {
    pub time: String,
    pub message: String,
    pub is_error: bool,
}

#[derive(Clone)]
pub struct Logger {
    entries: Arc<Mutex<Vec<LogEntry>>>,
    log_file: Option<PathBuf>,
}

impl Logger {
    pub fn new() -> Self {
        let log_dir = Self::get_log_dir();
        let log_file = if let Some(dir) = &log_dir {
            let _ = fs::create_dir_all(dir);
            let filename = format!("wforge_{}.log", Local::now().format("%Y%m%d"));
            Some(dir.join(filename))
        } else {
            None
        };

        Self {
            entries: Arc::new(Mutex::new(Vec::new())),
            log_file,
        }
    }

    fn get_log_dir() -> Option<PathBuf> {
        dirs::data_local_dir().map(|p| p.join("WForge").join("Logs"))
    }

    pub fn log(&self, message: &str) {
        self.add_entry(message, false);
    }

    pub fn error(&self, message: &str) {
        self.add_entry(message, true);
    }

    fn add_entry(&self, message: &str, is_error: bool) {
        let time = Local::now().format("%H:%M:%S").to_string();
        let entry = LogEntry {
            time: time.clone(),
            message: message.to_string(),
            is_error,
        };

        if let Ok(mut entries) = self.entries.lock() {
            entries.push(entry);
            // 保留最近 1000 条
            if entries.len() > 1000 {
                entries.remove(0);
            }
        }

        // 写入文件
        if let Some(path) = &self.log_file {
            if let Ok(mut file) = OpenOptions::new()
                .create(true)
                .append(true)
                .open(path)
            {
                let level = if is_error { "ERROR" } else { "INFO" };
                let _ = writeln!(file, "[{}] [{}] {}", time, level, message);
            }
        }
    }

    pub fn get_entries(&self) -> Vec<LogEntry> {
        self.entries.lock().map(|e| e.clone()).unwrap_or_default()
    }

    pub fn clear(&self) {
        if let Ok(mut entries) = self.entries.lock() {
            entries.clear();
        }
    }

    pub fn open_log_dir(&self) {
        if let Some(dir) = Self::get_log_dir() {
            let _ = open::that(dir);
        }
    }
}

impl Default for Logger {
    fn default() -> Self {
        Self::new()
    }
}
