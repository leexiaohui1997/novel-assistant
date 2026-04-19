pub mod commands;
pub mod config;
pub mod database;

use commands::novel_commands::{create_novel, get_novels};
use database::pool::init_pool;
use tauri::Builder;

pub async fn run() {
    // 初始化数据库连接池
    let pool = match init_pool().await {
        Ok(p) => {
            tracing::info!("数据库连接池初始化成功");
            p
        }
        Err(e) => {
            tracing::error!("数据库初始化失败: {}", e);
            panic!("无法初始化数据库: {}", e);
        }
    };

    Builder::default()
        .manage(pool)
        .invoke_handler(tauri::generate_handler![create_novel, get_novels])
        .run(tauri::generate_context!())
        .expect("运行 Tauri 应用时出错");
}
