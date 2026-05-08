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

    /// 列出指定会话下的全部消息（按 `sequence` 升序）
    ///
    /// 典型用途：AI 驱动器在触发调用前加载完整历史以构造 prompt 消息列表。
    ///
    /// # 参数
    /// - `conversation_id`: 会话 ID
    ///
    /// # 返回
    /// - `Ok(Vec<AiConversationMessage>)`: 该会话的全部消息（可能为空数组）
    /// - `Err(DbError)`: 数据库错误
    async fn list_by_conversation(
        &self,
        conversation_id: Uuid,
    ) -> Result<Vec<AiConversationMessage>, DbError>;

    /// 事务版"assistant 消息收尾"
    ///
    /// 在**同一个数据库事务**内完成以下三件事，任一步失败则整体回滚：
    /// 1. INSERT 一条新的会话消息（通常为 `MessageType::Assistant`）
    /// 2. UPDATE 所属会话 `ai_conversations` 的 `last_message_at = <新消息 created_at>`
    /// 3. UPDATE 所属会话 `ai_conversations` 的 `status = <conversation_status>`
    ///
    /// 典型用途：AI 驱动器在收到 AI 回复后，以原子方式写入回复并把会话置为 `Completed`。
    ///
    /// # 参数
    /// - `payload`: 新消息的创建 DTO（`id` / `sequence` 由上层预先生成）
    /// - `conversation_status`: 新的会话状态字符串（如 `"Completed"`）
    ///
    /// # 返回
    /// - `Ok(AiConversationMessage)`: 新插入的完整消息实体
    /// - `Err(DbError)`: 任一步数据库错误，事务已回滚
    async fn finalize_assistant_turn(
        &self,
        payload: CreateAiConversationMessage,
        conversation_status: &str,
    ) -> Result<AiConversationMessage, DbError>;
}
