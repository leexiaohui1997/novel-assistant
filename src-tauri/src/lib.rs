pub mod ai;
#[path = "ai-v2/mod.rs"]
pub mod ai_v2;
pub mod commands;
pub mod config;
pub mod database;
pub mod logging;
pub mod utils;

use std::sync::Arc;
use tokio::sync::RwLock;

use ai::actions::builtin::{
    EditChapterCharactersAction, EditChapterPlotAction, EditChapterPositioningAction,
    EditChapterTitleAction, ExtractTermsAction, GenerateChapterContentAction,
    GenerateCharacterAction, GenerateIntroductionAction, GenerateTitleAction,
    OptimizeCharacterAction, RecommendTagsAction,
};
use ai::actions::{ActionExecutor, ActionRouter};
use ai::model_fetchers::FetcherRegistry;
use ai::service::AiService;
use ai_v2::tools::builtin::{SearchNovelTool, SearchTagsTool};
use ai_v2::{AiService as AiServiceV2, SkillRegistry, TemplateManager, ToolRegistry};
use commands::action_commands::{execute_action, list_actions};
use commands::ai_commands::test_model;
use commands::ai_execute_commands::execute_ai;
use commands::chapter_commands::{
    batch_update_volumes, create_chapter, create_volume, delete_chapter, delete_volume,
    get_chapter_by_id, get_chapter_versions, get_chapters_with_pagination, get_volumes,
    update_chapter, update_volume,
};
use commands::chapter_outline_commands::{edit_chapter_outline, get_chapter_outline};
use commands::character_commands::{
    create_character, delete_character, get_character_by_id, get_characters_by_novel,
    get_characters_with_pagination, update_character,
};
use commands::creation_state_commands::{get_creation_state, upsert_creation_state};
use commands::log_commands::{get_log_file_content, get_log_files};
use commands::model_commands::{
    add_models, delete_model, fetch_provider_models, get_all_models, get_models_with_pagination,
    get_provider_types, set_model_as_default, toggle_model_enabled, toggle_model_thinking,
    update_model_alias,
};
use commands::novel_commands::{
    create_novel, delete_novel, get_novel_by_id, get_novel_stats, get_novels,
    get_novels_with_pagination, update_novel,
};
use commands::novel_term_commands::{
    create_novel_term, delete_novel_term, get_chapter_relations_by_term, get_chapter_terms,
    get_novel_term_by_id, get_novel_terms, update_chapter_terms, update_novel_term,
};
use commands::provider_commands::{
    create_provider, delete_provider, get_providers_with_pagination, update_provider,
};
use commands::tag_commands::{get_tags_by_audience, get_tags_by_ids};
use commands::tokens_dashboard_commands::{
    get_tokens_model_usage, get_tokens_summary, list_ai_call_logs,
};
use config::paths::{get_template_root, get_templates_base};
use database::pool::init_pool;
use database::repositories::{
    AiCallLogRepository, AiConversationMessageRepository, AiConversationRepository,
    ChapterOutlineRepository, ChapterRepository, ChapterTermRelationRepository,
    ChapterVersionRepository, CharacterRepository, CreationStateRepository, ModelRepository,
    NovelRepository, NovelTermRepository, ProviderRepository, SqliteAiCallLogRepository,
    SqliteAiConversationMessageRepository, SqliteAiConversationRepository,
    SqliteChapterOutlineRepository, SqliteChapterRepository, SqliteChapterTermRelationRepository,
    SqliteChapterVersionRepository, SqliteCharacterRepository, SqliteCreationStateRepository,
    SqliteModelRepository, SqliteNovelRepository, SqliteNovelTermRepository,
    SqliteProviderRepository, SqliteTagRepository, TagRepository,
};
use tauri::{Builder, Manager};

pub struct AppState {
    pub pool: sqlx::SqlitePool,
    pub novel_repo: Arc<RwLock<Box<dyn NovelRepository + Send + Sync>>>,
    pub chapter_repo: Arc<RwLock<Box<dyn ChapterRepository + Send + Sync>>>,
    pub chapter_version_repo: Arc<RwLock<Box<dyn ChapterVersionRepository + Send + Sync>>>,
    pub character_repo: Arc<RwLock<Box<dyn CharacterRepository + Send + Sync>>>,
    pub novel_term_repo: Arc<RwLock<Box<dyn NovelTermRepository + Send + Sync>>>,
    pub chapter_term_relation_repo:
        Arc<RwLock<Box<dyn ChapterTermRelationRepository + Send + Sync>>>,
    pub tag_repo: Arc<RwLock<Box<dyn TagRepository + Send + Sync>>>,
    pub creation_state_repo: Arc<RwLock<Box<dyn CreationStateRepository + Send + Sync>>>,
    pub provider_repo: Arc<RwLock<Box<dyn ProviderRepository + Send + Sync>>>,
    pub model_repo: Arc<RwLock<Box<dyn ModelRepository + Send + Sync>>>,
    pub call_log_repo: Arc<RwLock<Box<dyn AiCallLogRepository + Send + Sync>>>,
    pub chapter_outline_repo: Arc<RwLock<Box<dyn ChapterOutlineRepository + Send + Sync>>>,
    pub ai_conversation_repo: Arc<RwLock<Box<dyn AiConversationRepository + Send + Sync>>>,
    pub ai_conversation_message_repo:
        Arc<RwLock<Box<dyn AiConversationMessageRepository + Send + Sync>>>,
    pub fetcher_registry: Arc<RwLock<FetcherRegistry>>,
    // AI Actions 系统
    pub action_router: Arc<RwLock<ActionRouter>>,
    pub action_executor: Arc<ActionExecutor>,
    // Tera 模板管理（AI v2）
    pub template_manager: Arc<TemplateManager>,
    // AI 工具注册中心（AI v2）
    pub tool_registry: Arc<RwLock<ToolRegistry>>,
    // AI 服务（v2）
    pub ai_service: Arc<AiServiceV2>,
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

    // 初始化 AI Actions 路由（无异步依赖，在此处一次构建）
    let mut action_router = ActionRouter::new();
    action_router.register(Arc::new(RecommendTagsAction));
    action_router.register(Arc::new(GenerateIntroductionAction));
    action_router.register(Arc::new(GenerateTitleAction));
    action_router.register(Arc::new(GenerateCharacterAction));
    action_router.register(Arc::new(OptimizeCharacterAction));
    action_router.register(Arc::new(EditChapterPositioningAction));
    action_router.register(Arc::new(EditChapterPlotAction));
    action_router.register(Arc::new(EditChapterCharactersAction));
    action_router.register(Arc::new(EditChapterTitleAction));
    action_router.register(Arc::new(GenerateChapterContentAction));
    action_router.register(Arc::new(ExtractTermsAction));
    let action_router = Arc::new(RwLock::new(action_router));

    // 供 setup 闭包捕获使用
    let pool_for_setup = pool.clone();
    let action_router_for_setup = action_router.clone();

    Builder::default()
        .setup(move |app| {
            // 通过 AppHandle 解析模板目录（prod 态指向 resource_dir）
            let handle = app.handle().clone();
            let templates_base = match get_templates_base(&handle) {
                Ok(p) => p,
                Err(e) => {
                    tracing::error!("解析模板基础目录失败: {}", e);
                    return Err(Box::new(e));
                }
            };
            let template_root = match get_template_root(&handle) {
                Ok(p) => p,
                Err(e) => {
                    tracing::error!("解析 v2 模板目录失败: {}", e);
                    return Err(Box::new(e));
                }
            };

            // 初始化 Tera 模板管理器（AI v2）
            let template_manager = match TemplateManager::new(&template_root) {
                Ok(m) => {
                    tracing::info!("Tera 模板管理器初始化成功，根目录：{:?}", template_root);
                    Arc::new(m)
                }
                Err(e) => {
                    tracing::error!("Tera 模板管理器初始化失败: {}", e);
                    return Err(Box::new(e));
                }
            };

            // 初始化 AI 工具注册中心（AI v2）并注册内置工具
            // 注意：此处不能使用 tool_registry.blocking_write()，
            // Tauri v2 的 setup 回调运行在 tokio runtime 线程上，
            // 在 runtime 内调用 blocking_write 会触发 panic。
            let mut tool_registry_inner = ToolRegistry::new();
            tool_registry_inner.register(Arc::new(SearchNovelTool::new(Arc::new(RwLock::new(
                Box::new(SqliteNovelRepository::new(pool.clone())),
            )))));
            tool_registry_inner.register(Arc::new(SearchTagsTool::new(Arc::new(RwLock::new(
                Box::new(SqliteTagRepository::new(pool.clone())),
            )))));
            let tool_registry = Arc::new(RwLock::new(tool_registry_inner));

            // 初始化 AI 技能注册中心（AI v2）
            let mut skill_registry_inner = SkillRegistry::new();
            skill_registry_inner.register(Arc::new(ai_v2::skills::RefineNovelBasicSkill));
            let skill_registry = Arc::new(RwLock::new(skill_registry_inner));

            // AI v1 模板根（供 PromptTemplates 使用）
            let templates_root_v1 = Arc::new(templates_base);

            let pool = pool_for_setup.clone();
            let action_router = action_router_for_setup.clone();

            // 共享的 v2 会话 / 会话消息仓储（v2 AiService 与未来 commands 复用）
            let ai_conversation_repo: Arc<RwLock<Box<dyn AiConversationRepository + Send + Sync>>> =
                Arc::new(RwLock::new(Box::new(SqliteAiConversationRepository::new(
                    pool.clone(),
                ))));
            let ai_conversation_message_repo: Arc<
                RwLock<Box<dyn AiConversationMessageRepository + Send + Sync>>,
            > = Arc::new(RwLock::new(Box::new(
                SqliteAiConversationMessageRepository::new(pool.clone()),
            )));

            // 共享的 v1 AI 服务（ActionExecutor 与 v2 AiService 复用，避免多份实例）
            let v1_ai_service = Arc::new(AiService::new(
                Arc::new(RwLock::new(Box::new(SqliteModelRepository::new(
                    pool.clone(),
                )))),
                Arc::new(RwLock::new(Box::new(SqliteProviderRepository::new(
                    pool.clone(),
                )))),
                Arc::new(RwLock::new(Box::new(SqliteAiCallLogRepository::new(
                    pool.clone(),
                )))),
            ));

            // v2 AI 服务（新增）
            let ai_service_v2 = Arc::new(AiServiceV2::new(
                ai_conversation_repo.clone(),
                ai_conversation_message_repo.clone(),
                Arc::new(RwLock::new(Box::new(SqliteModelRepository::new(
                    pool.clone(),
                )))),
                Arc::new(RwLock::new(Box::new(SqliteProviderRepository::new(
                    pool.clone(),
                )))),
                v1_ai_service.clone(),
                tool_registry.clone(),
                skill_registry.clone(),
                template_manager.clone(),
            ));

            let state = AppState {
                pool: pool.clone(),
                novel_repo: Arc::new(RwLock::new(Box::new(SqliteNovelRepository::new(
                    pool.clone(),
                )))),
                chapter_repo: Arc::new(RwLock::new(Box::new(SqliteChapterRepository::new(
                    pool.clone(),
                )))),
                chapter_version_repo: Arc::new(RwLock::new(Box::new(
                    SqliteChapterVersionRepository::new(pool.clone()),
                ))),
                character_repo: Arc::new(RwLock::new(Box::new(SqliteCharacterRepository::new(
                    pool.clone(),
                )))),
                novel_term_repo: Arc::new(RwLock::new(Box::new(SqliteNovelTermRepository::new(
                    pool.clone(),
                )))),
                chapter_term_relation_repo: Arc::new(RwLock::new(Box::new(
                    SqliteChapterTermRelationRepository::new(pool.clone()),
                ))),
                tag_repo: Arc::new(RwLock::new(Box::new(SqliteTagRepository::new(
                    pool.clone(),
                )))),
                creation_state_repo: Arc::new(RwLock::new(Box::new(
                    SqliteCreationStateRepository::new(pool.clone()),
                ))),
                provider_repo: Arc::new(RwLock::new(Box::new(SqliteProviderRepository::new(
                    pool.clone(),
                )))),
                model_repo: Arc::new(RwLock::new(Box::new(SqliteModelRepository::new(
                    pool.clone(),
                )))),
                call_log_repo: Arc::new(RwLock::new(Box::new(SqliteAiCallLogRepository::new(
                    pool.clone(),
                )))),
                chapter_outline_repo: Arc::new(RwLock::new(Box::new(
                    SqliteChapterOutlineRepository::new(pool.clone()),
                ))),
                ai_conversation_repo: ai_conversation_repo.clone(),
                ai_conversation_message_repo: ai_conversation_message_repo.clone(),
                fetcher_registry: Arc::new(RwLock::new(FetcherRegistry::new())),
                action_router: action_router.clone(),
                action_executor: Arc::new(ActionExecutor::new(
                    action_router.clone(),
                    v1_ai_service.clone(),
                    Arc::new(RwLock::new(Box::new(SqliteTagRepository::new(
                        pool.clone(),
                    )))),
                    Arc::new(RwLock::new(Box::new(SqliteNovelRepository::new(
                        pool.clone(),
                    )))),
                    Arc::new(RwLock::new(Box::new(SqliteCharacterRepository::new(
                        pool.clone(),
                    )))),
                    Arc::new(RwLock::new(Box::new(SqliteChapterRepository::new(
                        pool.clone(),
                    )))),
                    Arc::new(RwLock::new(Box::new(SqliteChapterOutlineRepository::new(
                        pool.clone(),
                    )))),
                    Arc::new(RwLock::new(Box::new(SqliteNovelTermRepository::new(
                        pool.clone(),
                    )))),
                    Arc::new(RwLock::new(Box::new(
                        SqliteChapterTermRelationRepository::new(pool.clone()),
                    ))),
                    templates_root_v1,
                )),
                template_manager,
                tool_registry,
                ai_service: ai_service_v2,
            };

            app.manage(state);
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            create_novel,
            get_novel_by_id,
            get_novels,
            get_novels_with_pagination,
            get_novel_stats,
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
            get_chapter_by_id,
            edit_chapter_outline,
            get_chapter_outline,
            create_character,
            get_characters_by_novel,
            get_characters_with_pagination,
            get_character_by_id,
            update_character,
            delete_character,
            create_novel_term,
            update_novel_term,
            delete_novel_term,
            get_novel_term_by_id,
            get_novel_terms,
            get_chapter_terms,
            get_chapter_relations_by_term,
            update_chapter_terms,
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
            toggle_model_thinking,
            set_model_as_default,
            update_model_alias,
            test_model,
            execute_action,
            list_actions,
            get_tokens_summary,
            get_tokens_model_usage,
            list_ai_call_logs,
            execute_ai,
            get_log_files,
            get_log_file_content
        ])
        .run(tauri::generate_context!())
        .expect("运行 Tauri 应用时出错");
}
