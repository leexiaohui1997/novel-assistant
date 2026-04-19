// 防止在 Windows 发布版本中显示额外的控制台窗口，请勿删除！！
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use novel_assistant_lib::{config::env::AppConfig, logging};

fn main() {
    // 初始化应用配置
    AppConfig::load();

    // 初始化日志系统
    let _guard = logging::init_logging_for_env();

    // 运行异步主函数
    tauri::async_runtime::block_on(novel_assistant_lib::run());
}
