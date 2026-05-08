use std::sync::Arc;
use tokio::sync::RwLock;

use crate::database::repositories::{
    AiConversationMessageRepository, AiConversationRepository, ModelRepository, ProviderRepository,
};

pub mod conversation;
pub mod message;

/// AI 服务（v2）
///
/// 提供基于会话 / 消息的新一代 AI 能力。
///
/// 通过依赖注入持有四个仓储：
/// - `conversation_repo`：会话仓储（负责 `ai_conversations` 的写入与 `last_message_at` 维护）
/// - `message_repo`：会话消息仓储（负责 `ai_conversation_messages` 的插入与统计）
/// - `model_repo`：AI 模型仓储（用于写消息时查询模型元信息做快照）
/// - `provider_repo`：AI 供应商仓储（用于写消息时查询供应商名称做快照）
pub struct AiService {
    /// 会话仓储
    pub(crate) conversation_repo: Arc<RwLock<Box<dyn AiConversationRepository + Send + Sync>>>,

    /// 会话消息仓储
    pub(crate) message_repo: Arc<RwLock<Box<dyn AiConversationMessageRepository + Send + Sync>>>,

    /// AI 模型仓储
    pub(crate) model_repo: Arc<RwLock<Box<dyn ModelRepository + Send + Sync>>>,

    /// AI 供应商仓储
    pub(crate) provider_repo: Arc<RwLock<Box<dyn ProviderRepository + Send + Sync>>>,
}

impl AiService {
    /// 构造 `AiService`
    ///
    /// # 参数
    /// - `conversation_repo`: 会话仓储的依赖注入
    /// - `message_repo`: 会话消息仓储的依赖注入
    /// - `model_repo`: 模型仓储的依赖注入（用于消息写时模型名称快照）
    /// - `provider_repo`: 供应商仓储的依赖注入（用于消息写时供应商名称快照）
    pub fn new(
        conversation_repo: Arc<RwLock<Box<dyn AiConversationRepository + Send + Sync>>>,
        message_repo: Arc<RwLock<Box<dyn AiConversationMessageRepository + Send + Sync>>>,
        model_repo: Arc<RwLock<Box<dyn ModelRepository + Send + Sync>>>,
        provider_repo: Arc<RwLock<Box<dyn ProviderRepository + Send + Sync>>>,
    ) -> Self {
        Self {
            conversation_repo,
            message_repo,
            model_repo,
            provider_repo,
        }
    }
}
