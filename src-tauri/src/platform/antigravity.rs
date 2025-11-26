use crate::path_utils::AppPaths;
use std::path::PathBuf;

/// 获取Antigravity应用数据目录（跨平台）
pub fn get_antigravity_data_dir() -> Option<PathBuf> {
    AppPaths::antigravity_data_dir()
}

/// 获取Antigravity状态数据库文件路径
/// 使用自动检测的路径
pub fn get_antigravity_db_path() -> Option<PathBuf> {
    get_antigravity_data_dir().map(|dir| dir.join("state.vscdb"))
}


/// 检查Antigravity是否安装并运行
pub fn is_antigravity_available() -> bool {
    get_antigravity_db_path()
        .map(|path| path.exists())
        .unwrap_or(false)
}

/// 搜索可能的Antigravity安装位置
pub fn find_antigravity_installations() -> Vec<PathBuf> {
    let mut possible_paths = Vec::new();

    // 用户数据目录
    if let Some(user_data) = dirs::data_dir() {
        possible_paths.push(user_data.join("Antigravity"));
    }

    // 配置目录
    if let Some(config_dir) = dirs::config_dir() {
        possible_paths.push(config_dir.join("Antigravity"));
    }

    possible_paths
}

/// 获取所有可能的Antigravity数据库路径
pub fn get_all_antigravity_db_paths() -> Vec<PathBuf> {
    let mut db_paths = Vec::new();

    // 主要路径
    if let Some(main_path) = get_antigravity_db_path() {
        db_paths.push(main_path);
    }

    // 搜索其他可能的位置
    for install_dir in find_antigravity_installations() {
        if install_dir.exists() {
            // 递归搜索state.vscdb文件
            if let Ok(entries) = std::fs::read_dir(&install_dir) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.is_file() && path.file_name().is_some_and(|name| name == "state.vscdb")
                    {
                        db_paths.push(path);
                    }
                }
            }
        }
    }

    db_paths
}
