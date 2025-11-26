import { invoke } from '@tauri-apps/api/core';
import type { AntigravityPathInfo, AntigravityExecutableInfo, CurrentPaths } from '../types/tauri';
import { logger } from '../utils/logger';

/**
 * Antigravity 路径服务
 * 封装路径检测、验证和保存相关操作
 */
export class AntigravityPathService {
    /**
     * 检测 Antigravity 安装路径
     */
    static async detectAntigravityPath(): Promise<AntigravityPathInfo> {
        try {
            const result = await invoke<AntigravityPathInfo>('detect_antigravity_installation');
            return result;
        } catch (error) {
            logger.error('检测 Antigravity 路径失败', {
                module: 'AntigravityPathService',
                action: 'detect_path_failed',
                error: error instanceof Error ? error.message : String(error)
              });
            throw new Error(`检测失败: ${error}`);
        }
    }

  
    /**
     * 检测 Antigravity 可执行文件路径
     */
    static async detectExecutable(): Promise<AntigravityExecutableInfo> {
        try {
            const result = await invoke<AntigravityExecutableInfo>('detect_antigravity_executable');
            return result;
        } catch (error) {
            logger.error('检测 Antigravity 可执行文件失败', {
                module: 'AntigravityPathService',
                action: 'detect_executable_failed',
                error: error instanceof Error ? error.message : String(error)
              });
            throw new Error(`检测失败: ${error}`);
        }
    }

    /**
     * 验证可执行文件路径是否有效
     */
    static async validateExecutable(path: string): Promise<boolean> {
        try {
            const isValid = await invoke<boolean>('validate_antigravity_executable', { path });
            return isValid;
        } catch (error) {
            logger.error('验证可执行文件路径失败', {
                module: 'AntigravityPathService',
                action: 'validate_executable_failed',
                error: error instanceof Error ? error.message : String(error)
              });
            return false;
        }
    }

    /**
     * 保存用户选择的可执行文件路径
     */
    static async saveExecutable(path: string): Promise<string> {
        try {
            const result = await invoke<string>('save_antigravity_executable', { path });
            return result;
        } catch (error) {
            logger.error('保存可执行文件路径失败', {
                module: 'AntigravityPathService',
                action: 'save_executable_failed',
                error: error instanceof Error ? error.message : String(error)
              });
            throw new Error(`保存失败: ${error}`);
        }
    }

    /**
     * 获取当前配置的路径
     */
    static async getCurrentPaths(): Promise<CurrentPaths> {
        try {
            const result = await invoke<CurrentPaths>('get_current_paths');
            return result;
        } catch (error) {
            logger.error('获取当前路径失败', {
                module: 'AntigravityPathService',
                action: 'get_current_paths_failed',
                error: error instanceof Error ? error.message : String(error)
              });
            throw new Error(`获取失败: ${error}`);
        }
    }
}
