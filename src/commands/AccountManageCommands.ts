import {invoke} from '@tauri-apps/api/core';
import type {BackupData, RestoreResult} from './types/backup.types';

/**
 * 账户与备份综合命令
 */
export class AccountManageCommands {
  static async collectBackupContents(): Promise<BackupData[]> {
    return invoke('collect_backup_contents');
  }

  static async restoreBackupFiles(backups: BackupData[]): Promise<RestoreResult> {
    return invoke('restore_backup_files', { accountFileData: backups });
  }

  static async deleteBackup(name: string): Promise<string> {
    return invoke('delete_backup', { name });
  }

  static async clearAllBackups(): Promise<string> {
    return invoke('clear_all_backups');
  }

  // ==== 配置加解密 ====
  static async encryptConfig(jsonData: string, password: string): Promise<string> {
    return invoke('encrypt_config_data', { jsonData: jsonData, password });
  }

  static async decryptConfig(encryptedData: string, password: string): Promise<string> {
    return invoke('decrypt_config_data', { encryptedData: encryptedData, password });
  }
}
