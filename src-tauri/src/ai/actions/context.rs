// Action Context 模块
//
// 定义 ActionContext

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::ai::service::AiService;
use crate::database::repositories::{CharacterRepository, NovelRepository, TagRepository};

/// Action 执行上下文
///
/// 封装执行 Action 所需的所有信息和依赖
pub struct ActionContext {
    /// 输入参数（JSON 格式，灵活支持各种 Action）
    pub input: serde_json::Value,

    /// AI 服务引用（用于调用 LLM）
    pub ai_service: Arc<AiService>,

    /// 标签仓储引用（用于查询标签）
    pub tag_repo: Arc<RwLock<Box<dyn TagRepository + Send + Sync>>>,

    /// 小说仓储引用（用于查询小说信息）
    pub novel_repo: Arc<RwLock<Box<dyn NovelRepository + Send + Sync>>>,

    /// 角色仓储引用（用于查询角色列表）
    pub character_repo: Arc<RwLock<Box<dyn CharacterRepository + Send + Sync>>>,

    /// 指定的模型 ID（可选）
    pub model_id: Option<String>,

    /// 扩展元数据（未来可添加用户信息、会话 ID 等）
    pub metadata: HashMap<String, serde_json::Value>,
}

impl ActionContext {
    /// 创建新的 ActionContext
    pub fn new(
        input: serde_json::Value,
        ai_service: Arc<AiService>,
        tag_repo: Arc<RwLock<Box<dyn TagRepository + Send + Sync>>>,
        novel_repo: Arc<RwLock<Box<dyn NovelRepository + Send + Sync>>>,
        character_repo: Arc<RwLock<Box<dyn CharacterRepository + Send + Sync>>>,
    ) -> Self {
        Self {
            input,
            ai_service,
            tag_repo,
            novel_repo,
            character_repo,
            model_id: None,
            metadata: HashMap::new(),
        }
    }

    /// 设置模型 ID
    pub fn set_model_id(&mut self, model_id: String) {
        self.model_id = Some(model_id);
    }
}
