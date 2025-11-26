//! Tracing 配置模块
//! 提供统一的结构化日志配置和初始化

/// 记录系统启动信息
pub fn log_system_info() {
    tracing::info!(
        target: "app::startup",
        version = env!("CARGO_PKG_VERSION"),
        "🚀 启动 Antigravity Agent"
    );
    tracing::info!(
        target: "app::startup",
        os = std::env::consts::OS,
        arch = std::env::consts::ARCH,
        "🖥️ 系统信息"
    );
    tracing::info!(
        target: "app::startup",
        "📁 配置目录已初始化"
    );
    tracing::info!(
        target: "app::startup",
        "📁 Tracing 日志系统已启用"
    );
}

/// 记录数据库操作
pub fn log_database_operation(operation: &str, table: Option<&str>, success: bool) {
    match (table, success) {
        (Some(table), true) => {
            tracing::info!(
                target: "database::operation",
                operation = operation,
                table = table,
                success = true,
                "🗄️ 数据库操作成功"
            );
        }
        (Some(table), false) => {
            tracing::error!(
                target: "database::operation",
                operation = operation,
                table = table,
                success = false,
                "❌ 数据库操作失败"
            );
        }
        (None, true) => {
            tracing::info!(
                target: "database::operation",
                operation = operation,
                success = true,
                "🗄️ 数据库操作成功"
            );
        }
        (None, false) => {
            tracing::error!(
                target: "database::operation",
                operation = operation,
                success = false,
                "❌ 数据库操作失败"
            );
        }
    }
}
