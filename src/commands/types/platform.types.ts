/**
 * 平台相关类型定义
 */

/**
 * 平台信息
 */
export interface PlatformInfo {
  /** 操作系统类型 (windows/macos/linux) */
  os: string;

  /** 系统架构 (x86_64/aarch64) */
  arch: string;

  /** 系统家族 (unix/windows) */
  family: string;

  /** Antigravity 是否可用 */
  antigravity_available: boolean;

  /** Antigravity 可能的数据库路径列表 */
  antigravity_paths: string[];

  /** 配置目录路径 */
  config_dir?: string;

  /** 数据目录路径 */
  data_dir?: string;

  /** 用户主目录路径 */
  home_dir?: string;
}

/**
 * Antigravity 检测结果
 */
export interface DetectionResult {
  /** 是否找到 */
  found: boolean;

  /** 路径（如果找到） */
  path: string | null;

  /** 是否为用户自定义路径 */
  isCustomPath: boolean;
}

/**
 * 路径配置
 */
export interface PathConfig {
  /** 可执行文件路径 */
  executablePath?: string | null;
}
