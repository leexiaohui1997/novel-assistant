use async_trait::async_trait;
use sqlx::{Pool, Sqlite};
use uuid::Uuid;

use crate::database::{
    error::DbError,
    models::ai_call_log::{AiCallLog, CreateAiCallLog},
    repositories::AiCallLogRepository,
};

/// SQLite AI 调用记录 Repository 实现
pub struct SqliteAiCallLogRepository {
    pool: Pool<Sqlite>,
}

impl SqliteAiCallLogRepository {
    pub fn new(pool: Pool<Sqlite>) -> Self {
        Self { pool }
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
                message, response, status, error_message, call_time
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14)
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
        .bind(&log.status)
        .bind(log.error_message.as_deref())
        .bind(log.call_time)
        .fetch_one(&self.pool)
        .await
        .map_err(Into::into)
    }
}
