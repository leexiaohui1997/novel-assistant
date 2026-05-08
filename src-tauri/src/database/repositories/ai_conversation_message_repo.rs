use async_trait::async_trait;
use uuid::Uuid;

use crate::database::{
    error::DbError,
    models::ai_conversation_message::{AiConversationMessage, CreateAiConversationMessage},
};

/// AI 会话消息 Repository Trait
///
/// 定义 `ai_conversation_messages` 表的持久化操作，供业务层（如 `AiService`）依赖注入使用。
///
/// 职责边界：
/// - 仅负责 **本表** 的 CRUD 与统计类查询；
/// - 不得直接更新 `ai_conversations` 表（如 `last_message_at`），该职责由
///   [`crate::database::repositories::AiConversationRepository`] 承担。
#[async_trait]
pub trait AiConversationMessageRepository: Send + Sync {
    /// 插入一条会话消息
    ///
    /// # 参数
    /// - `payload`: 消息创建 DTO（`id` / `sequence` 由上层生成后传入）
    ///
    /// # 返回
    /// - `Ok(AiConversationMessage)`: 新建的完整消息实体（含数据库生成的时间字段）
    /// - `Err(DbError)`: 数据库错误（如外键约束、唯一约束冲突等均透传）
    async fn create(
        &self,
        payload: CreateAiConversationMessage,
    ) -> Result<AiConversationMessage, DbError>;

    /// 统计指定会话下的消息总数
    ///
    /// 典型用途：Service 层据此计算下一条消息的 `sequence`（即 `count + 1`）。
    ///
    /// # 参数
    /// - `conversation_id`: 会话 ID
    ///
    /// # 返回
    /// - `Ok(i64)`: 该会话当前的消息总数（可能为 0）
    /// - `Err(DbError)`: 数据库错误
    async fn count_by_conversation(&self, conversation_id: Uuid) -> Result<i64, DbError>;
}
