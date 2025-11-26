//! Language Server 相关的数据结构定义
//!
//! 将所有数据结构集中管理，便于性能追踪和优化

use serde::{Deserialize, Serialize};

/// 端口信息数据结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortInfo {
    pub https_port: Option<u16>,
    pub http_port: Option<u16>,
    pub extension_port: Option<u16>,
    pub log_path: Option<String>,
}

impl Default for PortInfo {
    fn default() -> Self {
        Self {
            https_port: None,
            http_port: None,
            extension_port: None,
            log_path: None,
        }
    }
}

/// 缓存统计信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheStats {
    pub csrf_cache_size: u64,
    pub ports_cache_size: u64,
}

/// 缓存初始化结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheInitResult {
    pub success: bool,
    pub message: String,
    pub csrf_cache_preloaded: bool,
    pub ports_cache_preloaded: bool,
}

/// Language Server 请求元数据
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RequestMetadata {
    pub ide_name: String,
    pub api_key: String,
    pub locale: String,
    pub ide_version: String,
    pub extension_name: String,
}

impl Default for RequestMetadata {
    fn default() -> Self {
        Self {
            ide_name: "antigravity".to_string(),
            api_key: String::new(),
            locale: "en".to_string(),
            ide_version: "1.11.5".to_string(),
            extension_name: "antigravity".to_string(),
        }
    }
}

/// Language Server API 请求体
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UserStatusRequest {
    pub metadata: RequestMetadata,
}



/// 缓存配置常量
pub struct CacheConfig {
    pub max_cache_entries: u64,
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            max_cache_entries: 100,
        }
    }
}

/// HTTP 请求配置
pub struct HttpConfig {
    pub request_timeout_ms: u64,
}

impl Default for HttpConfig {
    fn default() -> Self {
        Self {
            request_timeout_ms: 4000,
        }
    }
}