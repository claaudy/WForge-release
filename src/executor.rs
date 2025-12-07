use std::collections::HashMap;
use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::PathBuf;
use std::process::Command;
use std::sync::{Arc, Mutex};
use std::thread;

use crate::logger::Logger;
use crate::model::Action;

/// 下载进度 (0.0 - 1.0)
pub type DownloadProgress = Arc<Mutex<HashMap<String, f32>>>;

pub struct Executor {
    pub progress: DownloadProgress,
    download_dir: PathBuf,
}

impl Executor {
    pub fn new() -> Self {
        let download_dir = Self::get_download_dir();
        let _ = fs::create_dir_all(&download_dir);

        Self {
            progress: Arc::new(Mutex::new(HashMap::new())),
            download_dir,
        }
    }

    fn get_download_dir() -> PathBuf {
        // 优先使用 exe 所在目录
        if let Ok(exe_path) = std::env::current_exe() {
            if let Some(exe_dir) = exe_path.parent() {
                let download_dir = exe_dir.join("Downloads");
                if fs::create_dir_all(&download_dir).is_ok() {
                    // 测试是否可写
                    let test_file = download_dir.join(".write_test");
                    if fs::write(&test_file, "test").is_ok() {
                        let _ = fs::remove_file(test_file);
                        return download_dir;
                    }
                }
            }
        }

        // 备用：临时目录
        std::env::temp_dir().join("WForge").join("Downloads")
    }

    pub fn get_download_dir_path(&self) -> &PathBuf {
        &self.download_dir
    }

    pub fn execute(&self, item_id: &str, action: &Action, require_admin: bool, logger: &Logger) {
        match action {
            Action::Download { url, filename, sha256 } => {
                self.download(item_id, url, *filename, *sha256, logger);
            }
            Action::GitHubRelease { repo, asset_pattern } => {
                self.download_github_release(item_id, repo, asset_pattern, logger);
            }
            Action::WebScrape { page_url, link_regex } => {
                self.download_web_scrape(item_id, page_url, link_regex, logger);
            }
            Action::PowerShell { script, hidden } => {
                self.run_powershell(script, *hidden, require_admin, logger);
            }
        }
    }

    fn download_github_release(&self, item_id: &str, repo: &str, asset_pattern: &str, logger: &Logger) {
        let item_id = item_id.to_string();
        let repo = repo.to_string();
        let asset_pattern = asset_pattern.to_string();
        let progress = self.progress.clone();
        let download_dir = self.download_dir.clone();
        let logger = logger.clone();

        // 标记开始下载
        if let Ok(mut p) = progress.lock() {
            p.insert(item_id.clone(), 0.0);
        }

        thread::spawn(move || {
            let result = Self::do_github_release_download(&item_id, &repo, &asset_pattern, &download_dir, &progress, &logger);

            // 下载完成，移除进度
            if let Ok(mut p) = progress.lock() {
                p.remove(&item_id);
            }

            match result {
                Ok(path) => {
                    logger.log(&format!("[GitHub] 下载完成 | id={} | path={}", item_id, path.display()));
                }
                Err(e) => {
                    logger.error(&format!("[GitHub] 下载失败 | id={} | error={}", item_id, e));
                }
            }
        });
    }

    fn do_github_release_download(
        item_id: &str,
        repo: &str,
        asset_pattern: &str,
        download_dir: &PathBuf,
        progress: &DownloadProgress,
        logger: &Logger,
    ) -> Result<PathBuf, String> {
        logger.log(&format!(
            "[GitHub] 获取最新版本 | id={} | repo={} | pattern={}",
            item_id, repo, asset_pattern
        ));

        let client = reqwest::blocking::Client::builder()
            .timeout(std::time::Duration::from_secs(300))
            .user_agent("WForge/1.0")
            .build()
            .map_err(|e| format!("创建HTTP客户端失败: {}", e))?;

        // 获取最新 Release 信息
        let api_url = format!("https://api.github.com/repos/{}/releases/latest", repo);
        let response = client.get(&api_url).send().map_err(|e| format!("GitHub API 请求失败: {}", e))?;

        if !response.status().is_success() {
            return Err(format!("GitHub API 返回错误: {}", response.status()));
        }

        let json: serde_json::Value = response.json().map_err(|e| format!("解析 JSON 失败: {}", e))?;

        // 查找匹配的 asset
        let assets = json["assets"].as_array().ok_or("未找到 assets 字段")?;

        let asset = assets.iter().find(|a| {
            if let Some(name) = a["name"].as_str() {
                pattern_match(asset_pattern, name)
            } else {
                false
            }
        }).ok_or_else(|| format!("未找到匹配 '{}' 的文件", asset_pattern))?;

        let download_url = asset["browser_download_url"].as_str().ok_or("未找到下载链接")?;
        let filename = asset["name"].as_str().ok_or("未找到文件名")?;

        logger.log(&format!(
            "[GitHub] 找到文件 | name={} | url={}",
            filename, download_url
        ));

        // 下载文件
        Self::do_download(item_id, download_url, Some(filename), None, download_dir, progress, logger)
    }

    fn download_web_scrape(&self, item_id: &str, page_url: &str, link_regex: &str, logger: &Logger) {
        let item_id = item_id.to_string();
        let page_url = page_url.to_string();
        let link_regex = link_regex.to_string();
        let progress = self.progress.clone();
        let download_dir = self.download_dir.clone();
        let logger = logger.clone();

        // 标记开始下载
        if let Ok(mut p) = progress.lock() {
            p.insert(item_id.clone(), 0.0);
        }

        thread::spawn(move || {
            let result = Self::do_web_scrape_download(&item_id, &page_url, &link_regex, &download_dir, &progress, &logger);

            // 下载完成，移除进度
            if let Ok(mut p) = progress.lock() {
                p.remove(&item_id);
            }

            match result {
                Ok(path) => {
                    logger.log(&format!("[WebScrape] 下载完成 | id={} | path={}", item_id, path.display()));
                }
                Err(e) => {
                    logger.error(&format!("[WebScrape] 下载失败 | id={} | error={}", item_id, e));
                }
            }
        });
    }

    fn do_web_scrape_download(
        item_id: &str,
        page_url: &str,
        link_regex: &str,
        download_dir: &PathBuf,
        progress: &DownloadProgress,
        logger: &Logger,
    ) -> Result<PathBuf, String> {
        logger.log(&format!(
            "[WebScrape] 获取下载页面 | id={} | url={} | regex={}",
            item_id, page_url, link_regex
        ));

        let client = reqwest::blocking::Client::builder()
            .timeout(std::time::Duration::from_secs(60))
            .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36")
            .build()
            .map_err(|e| format!("创建HTTP客户端失败: {}", e))?;

        // 获取页面内容
        let response = client.get(page_url).send().map_err(|e| format!("获取页面失败: {}", e))?;

        if !response.status().is_success() {
            return Err(format!("HTTP 状态错误: {}", response.status()));
        }

        let html = response.text().map_err(|e| format!("读取页面内容失败: {}", e))?;

        // 使用正则表达式提取下载链接
        let re = regex::Regex::new(link_regex).map_err(|e| format!("正则表达式错误: {}", e))?;

        let captures = re.captures(&html).ok_or("未找到匹配的下载链接")?;

        let download_url = captures.get(1)
            .map(|m| m.as_str())
            .ok_or("正则表达式缺少捕获组")?;

        // 处理相对路径
        let full_url = if download_url.starts_with("http") {
            download_url.to_string()
        } else if download_url.starts_with('/') {
            // 提取 base URL
            let base = page_url.split('/').take(3).collect::<Vec<_>>().join("/");
            format!("{}{}", base, download_url)
        } else {
            format!("{}/{}", page_url.rsplit_once('/').map(|(b, _)| b).unwrap_or(page_url), download_url)
        };

        logger.log(&format!(
            "[WebScrape] 找到下载链接 | url={}",
            full_url
        ));

        // 下载文件
        Self::do_download(item_id, &full_url, None, None, download_dir, progress, logger)
    }

    fn download(&self, item_id: &str, url: &str, filename: Option<&str>, sha256: Option<&str>, logger: &Logger) {
        let item_id = item_id.to_string();
        let url = url.to_string();
        let filename = filename.map(|s| s.to_string());
        let sha256 = sha256.map(|s| s.to_string());
        let progress = self.progress.clone();
        let download_dir = self.download_dir.clone();
        let logger = logger.clone();

        // 标记开始下载
        if let Ok(mut p) = progress.lock() {
            p.insert(item_id.clone(), 0.0);
        }

        thread::spawn(move || {
            let result = Self::do_download(&item_id, &url, filename.as_deref(), sha256.as_deref(), &download_dir, &progress, &logger);

            // 下载完成，移除进度
            if let Ok(mut p) = progress.lock() {
                p.remove(&item_id);
            }

            match result {
                Ok(path) => {
                    logger.log(&format!("[Download] 下载完成 | id={} | path={}", item_id, path.display()));
                }
                Err(e) => {
                    logger.error(&format!("[Download] 下载失败 | id={} | error={}", item_id, e));
                }
            }
        });
    }

    fn do_download(
        item_id: &str,
        url: &str,
        filename: Option<&str>,
        sha256: Option<&str>,
        download_dir: &PathBuf,
        progress: &DownloadProgress,
        logger: &Logger,
    ) -> Result<PathBuf, String> {
        logger.log(&format!(
            "[Download] 开始下载 | id={} | url={} | filename={:?}",
            item_id, url, filename
        ));

        let client = reqwest::blocking::Client::builder()
            .timeout(std::time::Duration::from_secs(300))
            .build()
            .map_err(|e| format!("创建HTTP客户端失败: {}", e))?;

        let mut response = client.get(url).send().map_err(|e| format!("HTTP请求失败: {}", e))?;

        if !response.status().is_success() {
            return Err(format!("HTTP状态码错误: {}", response.status()));
        }

        // 确定文件名
        let final_filename = filename
            .map(|s| s.to_string())
            .or_else(|| {
                response.headers()
                    .get("content-disposition")
                    .and_then(|v| v.to_str().ok())
                    .and_then(|v| {
                        v.split("filename=").nth(1)
                            .map(|s| s.trim_matches('"').to_string())
                    })
            })
            .or_else(|| {
                url.split('/').last()
                    .map(|s| s.split('?').next().unwrap_or(s).to_string())
            })
            .unwrap_or_else(|| "download".to_string());

        let file_path = download_dir.join(&final_filename);
        let total_size = response.content_length().unwrap_or(0);

        let mut file = File::create(&file_path).map_err(|e| format!("创建文件失败: {}", e))?;

        // 分块下载，实时更新进度
        let mut downloaded: u64 = 0;
        let mut buffer = [0u8; 8192]; // 8KB 缓冲区
        let mut hasher = if sha256.is_some() {
            use sha2::{Sha256, Digest};
            Some(Sha256::new())
        } else {
            None
        };

        loop {
            let n = response.read(&mut buffer).map_err(|e| format!("读取数据失败: {}", e))?;
            if n == 0 {
                break;
            }

            file.write_all(&buffer[..n]).map_err(|e| format!("写入文件失败: {}", e))?;

            // 更新哈希
            if let Some(ref mut h) = hasher {
                use sha2::Digest;
                h.update(&buffer[..n]);
            }

            downloaded += n as u64;

            // 更新进度
            if total_size > 0 {
                if let Ok(mut p) = progress.lock() {
                    p.insert(item_id.to_string(), downloaded as f32 / total_size as f32);
                }
            }
        }

        // SHA256 校验
        if let Some(expected) = sha256 {
            if let Some(h) = hasher {
                use sha2::Digest;
                let result = hex::encode(h.finalize());
                if result.to_lowercase() != expected.to_lowercase() {
                    let _ = fs::remove_file(&file_path);
                    return Err(format!("SHA256 校验失败: 期望 {}, 实际 {}", expected, result));
                }
            }
        }

        Ok(file_path)
    }

    fn run_powershell(&self, script: &str, hidden: bool, require_admin: bool, logger: &Logger) {
        let script = script.to_string();
        let logger = logger.clone();

        thread::spawn(move || {
            let start_time = std::time::Instant::now();

            // 执行前日志
            logger.log(&format!(
                "[PowerShell] 开始执行 | hidden={} | require_admin={} | script={}",
                hidden, require_admin, script
            ));

            let mut cmd = Command::new("powershell");
            cmd.args(["-ExecutionPolicy", "Bypass", "-Command", &script]);

            if hidden {
                #[cfg(windows)]
                {
                    use std::os::windows::process::CommandExt;
                    cmd.creation_flags(0x08000000); // CREATE_NO_WINDOW
                }
                logger.log("[PowerShell] 已设置隐藏窗口模式");
            }

            // 管理员权限需要通过 runas
            if require_admin {
                #[cfg(windows)]
                {
                    // 使用 Base64 编码脚本，避免引号转义问题
                    use std::io::Write;
                    let mut encoded = Vec::new();
                    for c in script.encode_utf16() {
                        let _ = encoded.write_all(&c.to_le_bytes());
                    }
                    let base64_script = base64_encode(&encoded);

                    let ps_script = format!(
                        "Start-Process powershell -Verb RunAs -ArgumentList '-ExecutionPolicy Bypass -EncodedCommand {}'",
                        base64_script
                    );

                    logger.log(&format!(
                        "[PowerShell] 管理员模式 | encoded_script_len={} | 完整命令=powershell -ExecutionPolicy Bypass -Command \"{}\"",
                        base64_script.len(),
                        ps_script.replace('"', "\\\"")
                    ));

                    let mut admin_cmd = Command::new("powershell");
                    admin_cmd.args(["-ExecutionPolicy", "Bypass", "-Command", &ps_script]);

                    match admin_cmd.output() {
                        Ok(output) => {
                            let elapsed = start_time.elapsed();
                            let stdout = String::from_utf8_lossy(&output.stdout);
                            let stderr = String::from_utf8_lossy(&output.stderr);
                            let exit_code = output.status.code().map(|c| c.to_string()).unwrap_or_else(|| "unknown".to_string());

                            if output.status.success() {
                                logger.log(&format!(
                                    "[PowerShell] 执行成功 | exit_code={} | elapsed={:.2}s",
                                    exit_code, elapsed.as_secs_f64()
                                ));
                            } else {
                                logger.error(&format!(
                                    "[PowerShell] 执行失败 | exit_code={} | elapsed={:.2}s",
                                    exit_code, elapsed.as_secs_f64()
                                ));
                            }

                            if !stdout.is_empty() {
                                logger.log(&format!("[PowerShell] stdout: {}", stdout.trim()));
                            }
                            if !stderr.is_empty() {
                                logger.error(&format!("[PowerShell] stderr: {}", stderr.trim()));
                            }
                        }
                        Err(e) => {
                            let elapsed = start_time.elapsed();
                            logger.error(&format!(
                                "[PowerShell] 启动失败 | error={} | elapsed={:.2}s",
                                e, elapsed.as_secs_f64()
                            ));
                        }
                    }
                    return;
                }
            }

            // 非管理员模式执行
            logger.log(&format!(
                "[PowerShell] 普通模式 | 完整命令=powershell -ExecutionPolicy Bypass -Command \"{}\"",
                script.replace('"', "\\\"")
            ));

            match cmd.output() {
                Ok(output) => {
                    let elapsed = start_time.elapsed();
                    let stdout = String::from_utf8_lossy(&output.stdout);
                    let stderr = String::from_utf8_lossy(&output.stderr);
                    let exit_code = output.status.code().map(|c| c.to_string()).unwrap_or_else(|| "unknown".to_string());

                    if output.status.success() {
                        logger.log(&format!(
                            "[PowerShell] 执行成功 | exit_code={} | elapsed={:.2}s",
                            exit_code, elapsed.as_secs_f64()
                        ));
                    } else {
                        logger.error(&format!(
                            "[PowerShell] 执行失败 | exit_code={} | elapsed={:.2}s",
                            exit_code, elapsed.as_secs_f64()
                        ));
                    }

                    if !stdout.is_empty() {
                        logger.log(&format!("[PowerShell] stdout: {}", stdout.trim()));
                    }
                    if !stderr.is_empty() {
                        logger.error(&format!("[PowerShell] stderr: {}", stderr.trim()));
                    }
                }
                Err(e) => {
                    let elapsed = start_time.elapsed();
                    logger.error(&format!(
                        "[PowerShell] 启动失败 | error={} | elapsed={:.2}s",
                        e, elapsed.as_secs_f64()
                    ));
                }
            }
        });
    }

    pub fn is_downloading(&self, item_id: &str) -> bool {
        self.progress.lock()
            .map(|p| p.contains_key(item_id))
            .unwrap_or(false)
    }

    pub fn get_progress(&self, item_id: &str) -> Option<f32> {
        self.progress.lock()
            .ok()
            .and_then(|p| p.get(item_id).copied())
    }
}

impl Default for Executor {
    fn default() -> Self {
        Self::new()
    }
}

/// 简单的通配符匹配 (仅支持 *)
fn pattern_match(pattern: &str, text: &str) -> bool {
    let parts: Vec<&str> = pattern.split('*').collect();

    if parts.len() == 1 {
        // 没有通配符，精确匹配
        return pattern == text;
    }

    let mut pos = 0;
    for (i, part) in parts.iter().enumerate() {
        if part.is_empty() {
            continue;
        }

        if i == 0 {
            // 第一部分必须从开头匹配
            if !text.starts_with(part) {
                return false;
            }
            pos = part.len();
        } else if i == parts.len() - 1 {
            // 最后一部分必须在结尾匹配
            if !text.ends_with(part) {
                return false;
            }
        } else {
            // 中间部分只需存在
            if let Some(found) = text[pos..].find(part) {
                pos += found + part.len();
            } else {
                return false;
            }
        }
    }

    true
}

/// Base64 编码（避免引入额外依赖）
fn base64_encode(data: &[u8]) -> String {
    const ALPHABET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut result = String::new();
    for chunk in data.chunks(3) {
        let b0 = chunk[0] as usize;
        let b1 = chunk.get(1).copied().unwrap_or(0) as usize;
        let b2 = chunk.get(2).copied().unwrap_or(0) as usize;
        result.push(ALPHABET[b0 >> 2] as char);
        result.push(ALPHABET[((b0 & 0x03) << 4) | (b1 >> 4)] as char);
        if chunk.len() > 1 {
            result.push(ALPHABET[((b1 & 0x0f) << 2) | (b2 >> 6)] as char);
        } else {
            result.push('=');
        }
        if chunk.len() > 2 {
            result.push(ALPHABET[b2 & 0x3f] as char);
        } else {
            result.push('=');
        }
    }
    result
}
