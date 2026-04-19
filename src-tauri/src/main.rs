// 防止在 Windows 发布版本中显示额外的控制台窗口，请勿删除！！
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use tracing_subscriber::{self, EnvFilter};

fn main() {
    // 初始化应用配置
    novel_assistant_lib::config::env::AppConfig::load();

    // 初始化日志系统
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    // 运行异步主函数
    tauri::async_runtime::block_on(novel_assistant_lib::run());
}
