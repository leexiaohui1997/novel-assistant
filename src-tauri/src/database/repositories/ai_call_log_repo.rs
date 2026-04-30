use async_trait::async_trait;

use crate::database::{
    error::DbError,
    models::ai_call_log::{AiCallLog, CreateAiCallLog},
};

/// AI 调用记录 Repository Trait
#[async_trait]
pub trait AiCallLogRepository: Send + Sync {
    /// 创建 AI 调用记录
    async fn create(&self, log: CreateAiCallLog) -> Result<AiCallLog, DbError>;
}
