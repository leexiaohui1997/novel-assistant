use std::sync::Arc;
use tokio::sync::RwLock;

use crate::database::repositories::AiConversationRepository;

pub mod conversation;

/// AI 服务（v2）
///
/// 提供基于会话 / 消息的新一代 AI 能力。
pub struct AiService {
    /// 会话仓储
    pub(crate) conversation_repo: Arc<RwLock<Box<dyn AiConversationRepository + Send + Sync>>>,
}

impl AiService {
    /// 构造 `AiService`
    ///
    /// # 参数
    /// - `conversation_repo`: 会话仓储的依赖注入
    pub fn new(
        conversation_repo: Arc<RwLock<Box<dyn AiConversationRepository + Send + Sync>>>,
    ) -> Self {
        Self { conversation_repo }
    }
}
