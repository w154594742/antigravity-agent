import React, {useCallback, useEffect, useState} from 'react';
import {useDevToolsShortcut} from './hooks/useDevToolsShortcut';
import {useUserManagement} from './modules/user-management/store';
import {DATABASE_EVENTS, useDbMonitoringStore} from './modules/db-monitoring-store';
import useConfigManager from './modules/config-management/useConfigStore';
import {useAntigravityProcess} from './hooks/use-antigravity-process';
import {useAntigravityIsRunning} from './hooks/useAntigravityIsRunning';
import StatusNotification from './components/StatusNotification';
import Toolbar from './components/Toolbar';
import BusinessSettingsDialog from './components/business/SettingsDialog';
import PasswordDialog from './components/PasswordDialog';
import {TooltipProvider} from './components/ui/tooltip';
import {AntigravityPathService} from './services/antigravity-path-service';
import {useLanguageServerState} from "@/hooks/use-language-server-state.ts";
import {logger} from './utils/logger';
import {AppUserPanel} from "@/AppUserPanel.tsx";

interface Status {
  message: string;
  isError: boolean;
}

function AppContent() {
  // ========== 应用状态 ==========
  const [status, setStatus] = useState<Status>({message: '', isError: false});
  const [isSettingsOpen, setIsSettingsOpen] = useState(false);

  // 状态提示
  const showStatus = useCallback((message: string, isError: boolean = false): void => {
    setStatus({message, isError});
    setTimeout(() => setStatus({message: '', isError: false}), 5000);
  }, []);

  const [passwordDialog, setPasswordDialog] = useState({
    isOpen: false,
    title: '',
    description: '',
    requireConfirmation: false,
    onSubmit: () => {},
    validatePassword: null as (password: string) => { isValid: boolean; message?: string },
  });

  // 打开密码对话框
  const showPasswordDialog = useCallback((config) => {
    setPasswordDialog({
      isOpen: true,
      title: config.title,
      description: config.description || '',
      requireConfirmation: config.requireConfirmation || false,
      onSubmit: config.onSubmit,
      validatePassword: config.validatePassword
    });
  }, []);

  // 关闭密码对话框
  const closePasswordDialog = useCallback(() => {
    setPasswordDialog(prev => ({ ...prev, isOpen: false }));
  }, []);

  // 处理密码对话框取消
  const handlePasswordDialogCancel = useCallback(() => {
    closePasswordDialog();
    setStatus({message: '操作已取消', isError: true});
  }, [closePasswordDialog]);


  // 配置管理
  const {isImporting, isExporting, isCheckingData, importConfig, exportConfig} = useConfigManager(
    showStatus,
    showPasswordDialog,
    closePasswordDialog
  );

  // 进程管理
  const { isProcessLoading, backupAndRestartAntigravity} = useAntigravityProcess(showStatus);

  // 合并 loading 状态
  const loadingState = {
    isProcessLoading: isProcessLoading,
    isImporting,
    isExporting
  };

  return (
    <>
      <Toolbar
        onImport={importConfig}
        onExport={exportConfig}
        isCheckingData={isCheckingData}
        onBackupAndRestart={backupAndRestartAntigravity}
        loadingState={loadingState}
        showStatus={showStatus}
        onSettingsClick={() => setIsSettingsOpen(true)}
      />

      <div className="container">
        <AppUserPanel/>
      </div>

      <StatusNotification status={status}/>

      <PasswordDialog
        isOpen={passwordDialog.isOpen}
        title={passwordDialog.title}
        description={passwordDialog.description}
        requireConfirmation={passwordDialog.requireConfirmation}
        onSubmit={passwordDialog.onSubmit}
        onCancel={handlePasswordDialogCancel}
        onOpenChange={(isOpen) => {
          if (!isOpen) {
            closePasswordDialog();
          }
        }}
        validatePassword={passwordDialog.validatePassword}
      />

      <BusinessSettingsDialog
        isOpen={isSettingsOpen}
        onOpenChange={setIsSettingsOpen}
      />
    </>
  );
}

function App() {
  // ========== 应用状态 ==========
  const [status, setStatus] = useState<Status>({message: '', isError: false});
  const [isDetecting, setIsDetecting] = useState(true);
  const languageServerState = useLanguageServerState();

  // ========== Hook 集成 ==========
  useDevToolsShortcut();

  // 状态提示
  const showStatus = useCallback((message: string, isError: boolean = false): void => {
    setStatus({message, isError});
    setTimeout(() => setStatus({message: '', isError: false}), 5000);
  }, []);

  // 用户管理
  const {addCurrentUser} = useUserManagement();

  // 监听数据库变化事件
  const {initializeMonitoring, addListener} = useDbMonitoringStore();

  useEffect(() => {
    // 初始化监控（自动启动）
    initializeMonitoring();

    // 添加事件监听器
    return addListener(DATABASE_EVENTS.DATA_CHANGED, addCurrentUser);
  }, []);

  // 启动 Antigravity 进程状态自动检查
  const antigravityIsRunning = useAntigravityIsRunning();

  useEffect(() => {
    antigravityIsRunning.startAutoCheck();
    return () => antigravityIsRunning.stopAutoCheck();
  }, []);

  // 处理语言服务的状态
  useEffect(() => {
    if (antigravityIsRunning.isRunning) {
      languageServerState.initializeLanguageServerState()
    } else {
      languageServerState.clearLanguageServerState()
    }
  }, [antigravityIsRunning.isRunning]);

  // ========== 初始化启动流程 ==========
  const initializeApp = useCallback(async () => {
    try {
      logger.info('开始检测 Antigravity 安装', {
        module: 'AppState',
        action: 'detect_antigravity'
      });

      // 只检测数据库路径
      const pathInfo = await AntigravityPathService.detectAntigravityPath();

      if (pathInfo.found) {
        logger.info('Antigravity 数据库检测成功', {
          module: 'AppState',
          action: 'detect_success',
          pathFound: pathInfo.found
        });
        setIsDetecting(false);
      } else {
        logger.error('Antigravity 数据库未找到，应用无法使用', {
          module: 'AppState',
          action: 'detect_failed',
          pathFound: pathInfo.found
        });
        setIsDetecting(false);
      }
    } catch (error) {
      logger.error('启动检测失败', {
        module: 'AppState',
        action: 'detect_error',
        error: error instanceof Error ? error.message : String(error)
      });
      setIsDetecting(false);
    }
  }, []);

  // 组件启动时执行初始化
  useEffect(() => {
    initializeApp();
  }, [initializeApp]);

  // ========== 渲染逻辑 ==========
  if (isDetecting) {
    return (
      <div
        className="flex items-center justify-center min-h-screen bg-gradient-to-br from-slate-50 to-slate-100 dark:from-slate-900 dark:to-slate-800">
        <div className="text-center">
          <div className="animate-spin rounded-full h-16 w-16 mx-auto mb-6 text-blue-500"></div>
          <h2 className="text-2xl font-semibold mb-2 text-gray-800 dark:text-gray-100">
            正在检测 Antigravity...
          </h2>
          <p className="text-gray-500 dark:text-gray-400">
            请稍候，正在查找 Antigravity 安装路径
          </p>
        </div>
      </div>
    );
  }

  // 如果未运行
  if (!antigravityIsRunning.isRunning) {
    // TODO 补充 UI
    return <div>请先运行 Antigravity</div>
  }

  return <TooltipProvider>
    <AppContent/>
  </TooltipProvider>;
}

export default App;
