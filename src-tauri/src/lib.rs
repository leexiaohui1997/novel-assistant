pub mod commands;
pub mod config;
pub mod database;
pub mod logging;
pub mod utils;

use std::sync::Arc;
use tokio::sync::RwLock;

use commands::novel_commands::{create_novel, get_novels, get_novels_with_pagination};
use commands::tag_commands::{get_tags_by_audience, get_tags_by_ids};
use database::pool::init_pool;
use database::repositories::{
    NovelRepository, SqliteNovelRepository, SqliteTagRepository, TagRepository,
};
use tauri::Builder;

pub struct AppState {
    pub novel_repo: Arc<RwLock<Box<dyn NovelRepository + Send + Sync>>>,
    pub tag_repo: Arc<RwLock<Box<dyn TagRepository + Send + Sync>>>,
}

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

    // 创建仓储实例
    let novel_repo = SqliteNovelRepository::new(pool.clone());
    let tag_repo = SqliteTagRepository::new(pool.clone());

    // 创建应用状态
    let state = AppState {
        novel_repo: Arc::new(RwLock::new(Box::new(novel_repo))),
        tag_repo: Arc::new(RwLock::new(Box::new(tag_repo))),
    };

    Builder::default()
        .manage(state)
        .invoke_handler(tauri::generate_handler![
            create_novel,
            get_novels,
            get_novels_with_pagination,
            get_tags_by_audience,
            get_tags_by_ids
        ])
        .run(tauri::generate_context!())
        .expect("运行 Tauri 应用时出错");
}
