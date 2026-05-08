use async_trait::async_trait;
use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::database::{
    error::DbError,
    models::ai_conversation::{AiConversation, CreateAiConversation},
};

/// AI 会话 Repository Trait
///
/// 定义 `ai_conversations` 表的持久化操作，供业务层（如 `AiService`）依赖注入使用。
#[async_trait]
pub trait AiConversationRepository: Send + Sync {
    /// 创建一条会话记录
    ///
    /// # 参数
    /// - `data`: 会话创建 DTO
    ///
    /// # 返回
    /// - `Ok(AiConversation)`: 新建的完整会话实体（含数据库生成的时间字段）
    /// - `Err(DbError)`: 数据库错误
    async fn create(&self, data: CreateAiConversation) -> Result<AiConversation, DbError>;

    /// 更新会话的 `last_message_at` 时间戳
    ///
    /// 通常由 Service 层在插入一条新消息后立即调用，用于让"按最后一条消息时间排序"的
    /// 会话列表实时更新。
    ///
    /// # 参数
    /// - `conversation_id`: 会话 ID
    /// - `at`: 要写入的时间戳（UTC）
    ///
    /// # 返回
    /// - `Ok(())`: 更新成功（即便目标会话不存在也视作成功，影响行数为 0）
    /// - `Err(DbError)`: 数据库错误
    async fn touch_last_message_at(
        &self,
        conversation_id: Uuid,
        at: DateTime<Utc>,
    ) -> Result<(), DbError>;

    /// 更新会话的 `status` 字段
    ///
    /// # 参数
    /// - `id`: 会话 ID
    /// - `status`: 目标状态字符串（由枚举 `.to_string()` 得到，例如 `"Ready"` / `"Error"`）
    ///
    /// # 返回
    /// - `Ok(())`: 更新成功（即便目标会话不存在也视作成功，影响行数为 0）
    /// - `Err(DbError)`: 数据库错误
    async fn update_status(&self, id: Uuid, status: &str) -> Result<(), DbError>;
}
