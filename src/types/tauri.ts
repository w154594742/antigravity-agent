// Tauri API 类型定义
export interface BackupProfileParams {
  name: string;
  source_path: string;
}

export interface RestoreProfileParams {
  name: string;
  target_path: string;
}

export interface DeleteBackupParams {
  name: string;
}

export interface ProfileInfo {
  name: string;
  source_path: string;
  backup_path: string;
  created_at: string;
  last_updated: string;
}

// Tauri 命令返回类型
export type BackupProfileResult = string;
export type RestoreProfileResult = string;
export type ListBackupsResult = string[];
export type DeleteBackupResult = string;
export type SwitchToAntigravityAccountResult = string;

// 切换账户参数
export interface SwitchToAntigravityAccountParams {
  account_name: string;
}

// Antigravity 路径检测相关类型
export interface AntigravityPathInfo {
  found: boolean;
  path: string | null;
  isCustomPath: boolean;
}

// Antigravity 可执行文件路径信息
export interface AntigravityExecutableInfo {
  found: boolean;
  path: string | null;
  isCustomPath: boolean;
}

// 当前配置的路径信息
export interface CurrentPaths {
  executablePath: string | null;
}

// 错误类型
export type TauriError = string;
// Antigravity 当前用户信息类型
export interface AntigravityCurrentUserInfo {
  email: string;
  apiKey?: string;
  userStatusProtoBinaryBase64?: string;
  [key: string]: any;
}

// 备份当前账户参数类型
export interface BackupCurrentAccountParams {
  email: string;
}

// 备份当前账户结果类型
export type BackupCurrentAccountResult = string;
