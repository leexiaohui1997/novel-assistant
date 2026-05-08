use std::sync::Arc;
use tauri::AppHandle;
use tokio::sync::RwLock;

use crate::ai::service::AiService as V1AiService;
use crate::database::repositories::{
    AiConversationMessageRepository, AiConversationRepository, ModelRepository, ProviderRepository,
};

pub mod conversation;
pub mod driver;
pub mod message;

/// AI 服务（v2）
///
/// 提供基于会话 / 消息的新一代 AI 能力。
///
/// 通过依赖注入持有：
/// - `conversation_repo`：会话仓储（负责 `ai_conversations` 的写入与 `last_message_at` 维护）
/// - `message_repo`：会话消息仓储（负责 `ai_conversation_messages` 的插入与统计）
/// - `model_repo`：AI 模型仓储（用于写消息时查询模型元信息做快照）
/// - `provider_repo`：AI 供应商仓储（用于写消息时查询供应商名称做快照）
/// - `app_handle`：Tauri 应用句柄（用于 AI 驱动器向前端派发事件）
/// - `v1_ai_service`：v1 [`crate::ai::service::AiService`] 的共享句柄
///   （AI 驱动器复用其 `chat` 方法，避免重写 HTTP / 响应解析）
pub struct AiService {
    /// 会话仓储
    pub(crate) conversation_repo: Arc<RwLock<Box<dyn AiConversationRepository + Send + Sync>>>,

    /// 会话消息仓储
    pub(crate) message_repo: Arc<RwLock<Box<dyn AiConversationMessageRepository + Send + Sync>>>,

    /// AI 模型仓储
    pub(crate) model_repo: Arc<RwLock<Box<dyn ModelRepository + Send + Sync>>>,

    /// AI 供应商仓储
    pub(crate) provider_repo: Arc<RwLock<Box<dyn ProviderRepository + Send + Sync>>>,

    /// Tauri 应用句柄（事件派发使用）
    pub(crate) app_handle: AppHandle,

    /// v1 AI 服务共享句柄（复用 `chat` 方法）
    pub(crate) v1_ai_service: Arc<V1AiService>,
}

impl AiService {
    /// 构造 `AiService`
    ///
    /// # 参数
    /// - `conversation_repo`: 会话仓储的依赖注入
    /// - `message_repo`: 会话消息仓储的依赖注入
    /// - `model_repo`: 模型仓储的依赖注入（用于消息写时模型名称快照）
    /// - `provider_repo`: 供应商仓储的依赖注入（用于消息写时供应商名称快照）
    /// - `app_handle`: Tauri 应用句柄（用于 AI 驱动器向前端派发事件）
    /// - `v1_ai_service`: 共享的 v1 AI 服务（供 AI 驱动器复用）
    pub fn new(
        conversation_repo: Arc<RwLock<Box<dyn AiConversationRepository + Send + Sync>>>,
        message_repo: Arc<RwLock<Box<dyn AiConversationMessageRepository + Send + Sync>>>,
        model_repo: Arc<RwLock<Box<dyn ModelRepository + Send + Sync>>>,
        provider_repo: Arc<RwLock<Box<dyn ProviderRepository + Send + Sync>>>,
        app_handle: AppHandle,
        v1_ai_service: Arc<V1AiService>,
    ) -> Self {
        Self {
            conversation_repo,
            message_repo,
            model_repo,
            provider_repo,
            app_handle,
            v1_ai_service,
        }
    }
}
