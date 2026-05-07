use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::{Pool, Sqlite};
use uuid::Uuid;

use crate::database::{
    error::DbError,
    models::ai_call_log::{AiCallLog, CreateAiCallLog, ModelUsageRow, TokensSummary},
    repositories::AiCallLogRepository,
};
use crate::utils::pagination::PaginatedResult;

/// SQLite AI 调用记录 Repository 实现
pub struct SqliteAiCallLogRepository {
    pool: Pool<Sqlite>,
}

impl SqliteAiCallLogRepository {
    pub fn new(pool: Pool<Sqlite>) -> Self {
        Self { pool }
    }
}

/// 空汇总常量，供非法区间直接返回
fn empty_summary() -> TokensSummary {
    TokensSummary {
        total_requests: 0,
        input_tokens: 0,
        output_tokens: 0,
    }
}

#[async_trait]
impl AiCallLogRepository for SqliteAiCallLogRepository {
    async fn create(&self, log: CreateAiCallLog) -> Result<AiCallLog, DbError> {
        let id = Uuid::new_v4();

        sqlx::query_as::<_, AiCallLog>(
            r#"
            INSERT INTO ai_call_logs (
                id, provider_id, model_id, model_name, provider_name,
                input_tokens, output_tokens, total_tokens, duration_ms,
                message, response, thinking_content, status, error_message, call_time
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15)
            RETURNING *
            "#,
        )
        .bind(id)
        .bind(log.provider_id)
        .bind(log.model_id)
        .bind(&log.model_name)
        .bind(&log.provider_name)
        .bind(log.input_tokens)
        .bind(log.output_tokens)
        .bind(log.total_tokens)
        .bind(log.duration_ms)
        .bind(&log.message)
        .bind(log.response.as_deref())
        .bind(log.thinking_content.as_deref())
        .bind(&log.status)
        .bind(log.error_message.as_deref())
        .bind(log.call_time)
        .fetch_one(&self.pool)
        .await
        .map_err(Into::into)
    }

    async fn summary(
        &self,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> Result<TokensSummary, DbError> {
        if start > end {
            return Ok(empty_summary());
        }

        sqlx::query_as::<_, TokensSummary>(
            r#"
            SELECT
                COUNT(*) AS total_requests,
                COALESCE(SUM(input_tokens), 0) AS input_tokens,
                COALESCE(SUM(output_tokens), 0) AS output_tokens
            FROM ai_call_logs
            WHERE call_time BETWEEN ?1 AND ?2
            "#,
        )
        .bind(start)
        .bind(end)
        .fetch_one(&self.pool)
        .await
        .map_err(Into::into)
    }

    async fn group_by_model(
        &self,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> Result<Vec<ModelUsageRow>, DbError> {
        if start > end {
            return Ok(Vec::new());
        }

        sqlx::query_as::<_, ModelUsageRow>(
            r#"
            SELECT
                provider_id,
                model_id,
                MAX(provider_name) AS provider_name,
                MAX(model_name) AS model_name,
                COUNT(*) AS request_count,
                COALESCE(SUM(input_tokens), 0) AS input_tokens,
                COALESCE(SUM(output_tokens), 0) AS output_tokens,
                COALESCE(SUM(total_tokens), 0) AS total_tokens
            FROM ai_call_logs
            WHERE call_time BETWEEN ?1 AND ?2
            GROUP BY provider_id, model_id
            ORDER BY total_tokens DESC
            "#,
        )
        .bind(start)
        .bind(end)
        .fetch_all(&self.pool)
        .await
        .map_err(Into::into)
    }

    async fn list_by_range(
        &self,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
        page: i64,
        page_size: i64,
    ) -> Result<PaginatedResult<AiCallLog>, DbError> {
        if start > end {
            return Ok(PaginatedResult {
                data: Vec::new(),
                total: 0,
            });
        }

        let offset = (page - 1).max(0) * page_size;

        let total: (i64,) = sqlx::query_as(
            r#"SELECT COUNT(*) FROM ai_call_logs WHERE call_time BETWEEN ?1 AND ?2"#,
        )
        .bind(start)
        .bind(end)
        .fetch_one(&self.pool)
        .await?;

        let data = sqlx::query_as::<_, AiCallLog>(
            r#"
            SELECT * FROM ai_call_logs
            WHERE call_time BETWEEN ?1 AND ?2
            ORDER BY call_time DESC
            LIMIT ?3 OFFSET ?4
            "#,
        )
        .bind(start)
        .bind(end)
        .bind(page_size)
        .bind(offset)
        .fetch_all(&self.pool)
        .await?;

        Ok(PaginatedResult {
            data,
            total: total.0,
        })
    }
}
