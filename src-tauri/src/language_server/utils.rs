use std::cmp::min;
use std::path::PathBuf;

use anyhow::{anyhow, Result};
use regex::Regex;
use sysinfo::{System};
use walkdir::WalkDir;
use read_process_memory as _;

use super::cache::get_cache_manager;
use super::types::{PortInfo, CacheInitResult};

#[cfg(target_os = "windows")]
use crate::language_server::windows::scan_process_for_token;

#[cfg(target_os = "linux")]
use crate::language_server::linux::scan_process_for_token;

#[cfg(target_os = "macos")]
use crate::language_server::macos::scan_process_for_token;

pub(crate) const SCAN_AHEAD: usize = 200;
pub(crate) const CHUNK_SIZE: usize = 512 * 1024; // 512KB 分块读取，降低单次读耗时
pub(crate) const MAX_REGION_BYTES: usize = 64 * 1024 * 1024; // 每个区域最多扫描 64MB，加速


/// 查找最新的 Antigravity.log（按修改时间）
pub fn find_latest_antigravity_log() -> Option<PathBuf> {
    let mut candidates = Vec::new();
    if let Some(dir) = dirs::data_dir() {
        candidates.push(dir.join("Antigravity").join("logs"));
    }
    if let Some(dir) = dirs::config_dir() {
        candidates.push(dir.join("Antigravity").join("logs"));
    }

    let mut newest: Option<(PathBuf, std::time::SystemTime)> = None;

    for root in candidates {
        if !root.exists() {
            continue;
        }
        if let Ok(entries) = WalkDir::new(root).max_depth(6).into_iter().collect::<Result<Vec<_>, _>>() {
            for entry in entries {
                let path = entry.path();
                if path.file_name().is_some_and(|n| n == "Antigravity.log") && path.is_file() {
                    if let Ok(meta) = path.metadata() {
                        if let Ok(modified) = meta.modified() {
                            match &newest {
                                Some((_, ts)) if *ts >= modified => {}
                                _ => newest = Some((path.to_path_buf(), modified)),
                            }
                        }
                    }
                }
            }
        }
    }

    newest.map(|(p, _)| p)
}

/// 从日志内容解析 HTTPS/HTTP/extension 端口
pub fn parse_ports_from_log(content: &str) -> (Option<u16>, Option<u16>, Option<u16>) {
    let https_re = Regex::new(r"random port at (\d+) for HTTPS").unwrap();
    let http_re = Regex::new(r"random port at (\d+) for HTTP").unwrap();
    let ext_re = Regex::new(r"extension server client at port (\d+)").unwrap();

    let https_port = https_re
        .captures_iter(content)
        .last()
        .and_then(|c| c.get(1)?.as_str().parse::<u16>().ok());
    let http_port = http_re
        .captures_iter(content)
        .last()
        .and_then(|c| c.get(1)?.as_str().parse::<u16>().ok());
    let extension_port = ext_re
        .captures_iter(content)
        .last()
        .and_then(|c| c.get(1)?.as_str().parse::<u16>().ok());

    (https_port, http_port, extension_port)
}

/// 进程匹配：忽略大小写，允许 .exe 后缀
fn collect_target_pids() -> Vec<u32> {
    let mut system = System::new();
    system.refresh_processes();

    let mut pids = system
        .processes()
        .iter()
        .filter_map(|(pid, proc_)| {
            let name = proc_.name().to_string();
            if is_target_process(&name) {
                Some(pid.as_u32())
            } else {
                None
            }
        })
        .collect::<Vec<_>>();

    // PID 倒序：优先扫描最新启动的渲染/子进程
    pids.sort_unstable_by(|a, b| b.cmp(a));
    pids
}

fn is_target_process(name: &str) -> bool {
    let normalized = name
        .trim()
        .to_ascii_lowercase()
        .trim_end_matches(".exe")
        .to_string();
    normalized.contains("antigravity") || normalized.contains("windsurf")
}

fn get_patterns() -> (Vec<u8>, Vec<u8>) {
    let key = "x-codeium-csrf-token";
    let pat_utf8 = key.as_bytes().to_vec();
    let pat_utf16: Vec<u8> = key.encode_utf16().flat_map(|c| c.to_le_bytes()).collect();
    (pat_utf8, pat_utf16)
}

pub(crate) fn find_all_positions(haystack: &[u8], needle: &[u8]) -> Vec<usize> {
    if needle.is_empty() || haystack.len() < needle.len() {
        return Vec::new();
    }
    let mut positions = Vec::new();
    let first_byte = needle[0];
    let mut i = 0;
    let max_search_idx = haystack.len() - needle.len();

    while i <= max_search_idx {
        if haystack[i] == first_byte {
            if &haystack[i..i + needle.len()] == needle {
                positions.push(i);
                i += needle.len();
                continue;
            }
        }
        i += 1;
    }
    positions
}

pub(crate) fn search_bytes_for_token(data: &[u8], uuid_re: &Regex, patterns: &(Vec<u8>, Vec<u8>)) -> Option<String> {
    let (pat_utf8, pat_utf16) = patterns;

    for pat in [pat_utf8, pat_utf16] {
        for pos in find_all_positions(data, pat) {
            let start = pos + pat.len();
            if start >= data.len() {
                continue;
            }
            let end = min(start + SCAN_AHEAD, data.len());
            let window = &data[start..end];

            // 尝试 UTF-8
            let utf8_text = String::from_utf8_lossy(window);
            if let Some(mat) = uuid_re.find(&utf8_text) {
                return Some(mat.as_str().to_string());
            }

            // 尝试 UTF-16LE 解码
            let utf16_units: Vec<u16> = window
                .chunks_exact(2)
                .map(|pair| u16::from_le_bytes([pair[0], pair[1]]))
                .collect();
            let utf16_text = String::from_utf16_lossy(&utf16_units);
            if let Some(mat) = uuid_re.find(&utf16_text) {
                return Some(mat.as_str().to_string());
            }
        }
    }

    None
}


/// 带 CSRF token 缓存的获取函数（使用 moka）
pub async fn get_csrf_token_with_cache() -> Result<String> {
    let cache = get_cache_manager();
    let cache_key = "csrf_token".to_string();

    // 先尝试从缓存获取
    if let Some(cached_token) = cache.get_csrf_token(&cache_key).await {
        tracing::info!("使用缓存的 CSRF token");
        return Ok(cached_token);
    }

    // 缓存无效，重新获取
    tracing::info!("缓存无效，重新扫描获取 CSRF token");
    let start_time = std::time::Instant::now();

    // 在 blocking 任务中执行 CPU 密集型操作
    let token = tokio::task::spawn_blocking(move || {
        // 直接调用原始函数，不在闭包内定义
        find_csrf_token_from_memory_direct()
    }).await??;

    let scan_duration = start_time.elapsed();
    tracing::info!("CSRF token 扫描完成，耗时: {:?}", scan_duration);

    // 更新缓存
    cache.set_csrf_token(&cache_key, token.clone()).await;

    Ok(token)
}

/// 直接获取 CSRF token（不使用缓存的底层函数）
fn find_csrf_token_from_memory_direct() -> Result<String> {
    // 将原始函数的逻辑复制到这里，避免在闭包中调用
    let uuid_re = Regex::new(r"[0-9a-fA-F]{8}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{12}")
        .expect("valid uuid regex");
    let patterns = get_patterns();

    let pids = collect_target_pids();
    if pids.is_empty() {
        return Err(anyhow!("未找到运行中的 Antigravity/Windsurf 进程"));
    }

    for pid in pids {
        match scan_process_for_token(pid, &uuid_re, &patterns) {
            Ok(Some(token)) => return Ok(token),
            Ok(None) => continue,
            Err(e) => {
                tracing::warn!(pid, error = %e, "扫描进程失败");
                continue;
            }
        }
    }

    Err(anyhow!("未在运行中的 Antigravity/Windsurf 进程内存中找到 CSRF token"))
}

/// 获取端口信息（带缓存）
pub async fn get_ports_with_cache() -> Result<PortInfo> {
    let cache = get_cache_manager();
    let cache_key = "ports_info".to_string();

    // 先尝试从缓存获取
    if let Some(cached_ports) = cache.get_ports(&cache_key).await {
        tracing::info!("使用缓存的端口信息");
        return Ok(cached_ports);
    }

    // 缓存无效，重新获取
    tracing::info!("缓存无效，重新解析端口信息");
    let start_time = std::time::Instant::now();

    // 在 blocking 任务中执行 I/O 密集型操作
    let port_info = tokio::task::spawn_blocking(move || -> Result<PortInfo> {
        // 1) 查找最新的日志文件
        let log_path = find_latest_antigravity_log()
            .ok_or_else(|| anyhow!("未找到 Antigravity.log，无法确定端口"))?;

        // 2) 读取日志内容
        let content = std::fs::read_to_string(&log_path)
            .map_err(|e| anyhow!("读取日志失败: {e}"))?;

        // 3) 解析端口信息
        let (https_port, http_port, extension_port) = parse_ports_from_log(&content);

        Ok(PortInfo {
            https_port,
            http_port,
            extension_port,
            log_path: Some(log_path.to_string_lossy().to_string()),
        })
    }).await??;

    let parse_duration = start_time.elapsed();
    tracing::info!("端口信息解析完成，耗时: {:?}", parse_duration);

    // 更新缓存
    cache.set_ports(&cache_key, port_info.clone()).await;

    Ok(port_info)
}



/// 初始化锁，防止并发初始化
static INIT_LOCK: std::sync::OnceLock<tokio::sync::Mutex<()>> = std::sync::OnceLock::new();

/// 初始化缓存（预热缓存）
pub async fn initialize_cache() -> CacheInitResult {
    // 获取或创建初始化锁
    let lock = INIT_LOCK.get_or_init(|| tokio::sync::Mutex::new(()));

    // 尝试获取锁，如果已经被占用则直接返回当前状态
    if let Ok(_guard) = lock.try_lock() {
        // 获得锁，执行初始化
        perform_cache_initialization().await
    } else {
        // 锁被占用，说明正在初始化，返回当前缓存状态
        tracing::info!("缓存正在初始化中，跳过重复初始化");

        let cache = get_cache_manager();
        let stats = cache.get_stats();

        CacheInitResult {
            success: stats.csrf_cache_size > 0 || stats.ports_cache_size > 0,
            message: "缓存正在初始化中，请稍后再试".to_string(),
            csrf_cache_preloaded: stats.csrf_cache_size > 0,
            ports_cache_preloaded: stats.ports_cache_size > 0,
        }
    }
}

/// 执行实际的缓存初始化
async fn perform_cache_initialization() -> CacheInitResult {
    let start_time = std::time::Instant::now();
    tracing::info!("开始执行缓存初始化...");
    let mut csrf_loaded = false;
    let mut ports_loaded = false;
    let mut errors = Vec::new();

    tracing::info!("开始初始化缓存...");

    // 1. 尝试预热 CSRF token
    match get_csrf_token_with_cache().await {
        Ok(_) => {
            csrf_loaded = true;
            tracing::info!("CSRF token 缓存预热成功");
        }
        Err(e) => {
            let error_msg = format!("CSRF token 缓存预热失败: {}", e);
            tracing::warn!("{}", error_msg);
            errors.push(error_msg);
        }
    }

    // 2. 尝试预热端口信息
    match get_ports_with_cache().await {
        Ok(_) => {
            ports_loaded = true;
            tracing::info!("端口信息缓存预热成功");
        }
        Err(e) => {
            let error_msg = format!("端口信息缓存预热失败: {}", e);
            tracing::warn!("{}", error_msg);
            errors.push(error_msg);
        }
    }

    let duration = start_time.elapsed();
    let duration_ms = duration.as_millis() as u64;

    let success = csrf_loaded || ports_loaded; // 至少一个成功就算成功
    let message = if success {
        if csrf_loaded && ports_loaded {
            format!("缓存初始化成功 ({}ms)", duration_ms)
        } else {
            format!("缓存部分初始化成功 ({}ms) - CSRF: {}, 端口: {}",
                duration_ms, csrf_loaded, ports_loaded)
        }
    } else {
        format!("缓存初始化失败 ({}ms) - {}", duration_ms, errors.join("; "))
    };

    tracing::info!("缓存初始化完成: {}", message);

    CacheInitResult {
        success,
        message,
        csrf_cache_preloaded: csrf_loaded,
        ports_cache_preloaded: ports_loaded,
    }
}
