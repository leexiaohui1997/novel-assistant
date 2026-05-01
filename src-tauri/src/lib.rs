pub mod ai;
pub mod commands;
pub mod config;
pub mod database;
pub mod logging;
pub mod utils;

use std::sync::Arc;
use tokio::sync::RwLock;

use ai::model_fetchers::FetcherRegistry;
use commands::ai_commands::test_model;
use commands::chapter_commands::{
    batch_update_volumes, create_chapter, create_volume, delete_chapter, delete_volume,
    get_chapter_versions, get_chapters_with_pagination, get_volumes, update_chapter, update_volume,
};
use commands::creation_state_commands::{get_creation_state, upsert_creation_state};
use commands::model_commands::{
    add_models, delete_model, fetch_provider_models, get_models_with_pagination,
    get_provider_types, toggle_model_enabled, update_model_alias,
};
use commands::novel_commands::{
    create_novel, get_novel_by_id, get_novels, get_novels_with_pagination, update_novel,
};
use commands::provider_commands::{
    create_provider, delete_provider, get_providers_with_pagination, update_provider,
};
use commands::tag_commands::{get_tags_by_audience, get_tags_by_ids};
use database::pool::init_pool;
use database::repositories::{
    AiCallLogRepository, ChapterRepository, ChapterVersionRepository, CreationStateRepository,
    ModelRepository, NovelRepository, ProviderRepository, SqliteAiCallLogRepository,
    SqliteChapterRepository, SqliteChapterVersionRepository, SqliteCreationStateRepository,
    SqliteModelRepository, SqliteNovelRepository, SqliteProviderRepository, SqliteTagRepository,
    TagRepository,
};
use tauri::Builder;

pub struct AppState {
    pub novel_repo: Arc<RwLock<Box<dyn NovelRepository + Send + Sync>>>,
    pub chapter_repo: Arc<RwLock<Box<dyn ChapterRepository + Send + Sync>>>,
    pub chapter_version_repo: Arc<RwLock<Box<dyn ChapterVersionRepository + Send + Sync>>>,
    pub tag_repo: Arc<RwLock<Box<dyn TagRepository + Send + Sync>>>,
    pub creation_state_repo: Arc<RwLock<Box<dyn CreationStateRepository + Send + Sync>>>,
    pub provider_repo: Arc<RwLock<Box<dyn ProviderRepository + Send + Sync>>>,
    pub model_repo: Arc<RwLock<Box<dyn ModelRepository + Send + Sync>>>,
    pub call_log_repo: Arc<RwLock<Box<dyn AiCallLogRepository + Send + Sync>>>,
    pub fetcher_registry: Arc<RwLock<FetcherRegistry>>,
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
    let chapter_repo = SqliteChapterRepository::new(pool.clone());
    let chapter_version_repo = SqliteChapterVersionRepository::new(pool.clone());
    let tag_repo = SqliteTagRepository::new(pool.clone());
    let creation_state_repo = SqliteCreationStateRepository::new(pool.clone());
    let provider_repo = SqliteProviderRepository::new(pool.clone());
    let model_repo = SqliteModelRepository::new(pool.clone());
    let call_log_repo = SqliteAiCallLogRepository::new(pool.clone());

    // 创建模型拉取策略注册表
    let fetcher_registry = FetcherRegistry::new();

    // 创建应用状态
    let state = AppState {
        novel_repo: Arc::new(RwLock::new(Box::new(novel_repo))),
        chapter_repo: Arc::new(RwLock::new(Box::new(chapter_repo))),
        chapter_version_repo: Arc::new(RwLock::new(Box::new(chapter_version_repo))),
        tag_repo: Arc::new(RwLock::new(Box::new(tag_repo))),
        creation_state_repo: Arc::new(RwLock::new(Box::new(creation_state_repo))),
        provider_repo: Arc::new(RwLock::new(Box::new(provider_repo))),
        model_repo: Arc::new(RwLock::new(Box::new(model_repo))),
        call_log_repo: Arc::new(RwLock::new(Box::new(call_log_repo))),
        fetcher_registry: Arc::new(RwLock::new(fetcher_registry)),
    };
    Builder::default()
        .manage(state)
        .invoke_handler(tauri::generate_handler![
            create_novel,
            get_novel_by_id,
            get_novels,
            get_novels_with_pagination,
            update_novel,
            create_volume,
            update_volume,
            delete_volume,
            get_volumes,
            batch_update_volumes,
            create_chapter,
            update_chapter,
            delete_chapter,
            get_chapters_with_pagination,
            get_chapter_versions,
            get_tags_by_audience,
            get_tags_by_ids,
            get_creation_state,
            upsert_creation_state,
            create_provider,
            get_providers_with_pagination,
            update_provider,
            delete_provider,
            fetch_provider_models,
            get_provider_types,
            add_models,
            get_models_with_pagination,
            delete_model,
            toggle_model_enabled,
            update_model_alias,
            test_model
        ])
        .run(tauri::generate_context!())
        .expect("运行 Tauri 应用时出错");
}
