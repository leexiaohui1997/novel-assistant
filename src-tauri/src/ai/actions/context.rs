// Action Context 模块
//
// 定义 ActionContext

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::ai::service::AiService;
use crate::database::repositories::TagRepository;

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

    /// 扩展元数据（未来可添加用户信息、会话 ID 等）
    pub metadata: HashMap<String, serde_json::Value>,
}

impl ActionContext {
    /// 创建新的 ActionContext
    pub fn new(
        input: serde_json::Value,
        ai_service: Arc<AiService>,
        tag_repo: Arc<RwLock<Box<dyn TagRepository + Send + Sync>>>,
    ) -> Self {
        Self {
            input,
            ai_service,
            tag_repo,
            metadata: HashMap::new(),
        }
    }
}
