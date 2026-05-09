use async_trait::async_trait;
use chrono::{DateTime, Utc};

use crate::database::{
    error::DbError,
    models::ai_call_log::{AiCallLog, CreateAiCallLog, ModelUsageRow, TokensSummary},
};
use crate::utils::pagination::PaginatedResult;

/// AI 调用记录 Repository Trait
#[async_trait]
pub trait AiCallLogRepository: Send + Sync {
    /// 创建 AI 调用记录
    async fn create(&self, log: CreateAiCallLog) -> Result<AiCallLog, DbError>;

    /// 汇总指定时间范围内的请求数与 token 合计
    async fn summary(
        &self,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> Result<TokensSummary, DbError>;

    /// 按 (provider, model) 聚合指定时间范围内的用量
    async fn group_by_model(
        &self,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> Result<Vec<ModelUsageRow>, DbError>;

    /// 分页获取指定时间范围内的调用记录（按 call_time DESC）
    async fn list_by_range(
        &self,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
        page: i64,
        page_size: i64,
    ) -> Result<PaginatedResult<AiCallLog>, DbError>;

    /// 获取最近一条调用记录（按 call_time DESC 取首条）
    ///
    /// - 返回 `Ok(None)` 表示表内无任何记录。
    async fn find_latest(&self) -> Result<Option<AiCallLog>, DbError>;
}
