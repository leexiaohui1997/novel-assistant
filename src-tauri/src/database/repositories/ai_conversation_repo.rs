use async_trait::async_trait;

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
}
