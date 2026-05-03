pub mod ai;
pub mod commands;
pub mod config;
pub mod database;
pub mod logging;
pub mod utils;

use std::sync::Arc;
use tokio::sync::RwLock;

use ai::actions::builtin::{GenerateIntroductionAction, GenerateTitleAction, RecommendTagsAction};
use ai::actions::{ActionExecutor, ActionRouter};
use ai::model_fetchers::FetcherRegistry;
use ai::service::AiService;
use commands::action_commands::{execute_action, list_actions};
use commands::ai_commands::test_model;
use commands::chapter_commands::{
    batch_update_volumes, create_chapter, create_volume, delete_chapter, delete_volume,
    get_chapter_versions, get_chapters_with_pagination, get_volumes, update_chapter, update_volume,
};
use commands::creation_state_commands::{get_creation_state, upsert_creation_state};
use commands::model_commands::{
    add_models, delete_model, fetch_provider_models, get_all_models, get_models_with_pagination,
    get_provider_types, toggle_model_enabled, update_model_alias,
};
use commands::novel_commands::{
    create_novel, delete_novel, get_novel_by_id, get_novels, get_novels_with_pagination,
    update_novel,
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
    // AI Actions 系统
    pub action_router: Arc<RwLock<ActionRouter>>,
    pub action_executor: Arc<ActionExecutor>,
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

    // 初始化 AI Actions 系统
    let mut action_router = ActionRouter::new();
    action_router.register(Arc::new(RecommendTagsAction));
    action_router.register(Arc::new(GenerateIntroductionAction));
    action_router.register(Arc::new(GenerateTitleAction));
    let action_router = Arc::new(RwLock::new(action_router));

    // 创建应用状态（先不包含 action_executor）
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
        action_router: action_router.clone(),
        action_executor: Arc::new(ActionExecutor::new(
            action_router.clone(),
            Arc::new(AiService::new(
                Arc::new(RwLock::new(Box::new(SqliteModelRepository::new(
                    pool.clone(),
                )))),
                Arc::new(RwLock::new(Box::new(SqliteProviderRepository::new(
                    pool.clone(),
                )))),
                Arc::new(RwLock::new(Box::new(SqliteAiCallLogRepository::new(
                    pool.clone(),
                )))),
            )),
            Arc::new(RwLock::new(Box::new(SqliteTagRepository::new(
                pool.clone(),
            )))),
        )),
    };
    Builder::default()
        .manage(state)
        .invoke_handler(tauri::generate_handler![
            create_novel,
            get_novel_by_id,
            get_novels,
            get_novels_with_pagination,
            update_novel,
            delete_novel,
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
            get_all_models,
            get_models_with_pagination,
            delete_model,
            toggle_model_enabled,
            update_model_alias,
            test_model,
            execute_action,
            list_actions
        ])
        .run(tauri::generate_context!())
        .expect("运行 Tauri 应用时出错");
}
