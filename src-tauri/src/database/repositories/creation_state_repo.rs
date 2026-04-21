use async_trait::async_trait;
use chrono::Utc;
use sqlx::SqlitePool;
use uuid::Uuid;

use crate::database::error::DbError;
use crate::database::models::creation_state::CreationState;

/// 创作状态仓储 trait
#[async_trait]
pub trait CreationStateRepository {
    /// 获取指定小说的状态记录
    async fn get_state(&self, novel_id: &Uuid) -> Result<Option<CreationState>, DbError>;

    /// 创建或更新指定小说的状态记录（upsert）
    async fn upsert_state(&self, novel_id: &Uuid) -> Result<CreationState, DbError>;
}

/// SQLite 创作状态仓储实现
pub struct SqliteCreationStateRepository {
    pool: SqlitePool,
}

impl SqliteCreationStateRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl CreationStateRepository for SqliteCreationStateRepository {
    /// 获取指定小说的创作状态记录
    ///
    /// 如果不存在则返回 None
    async fn get_state(&self, novel_id: &Uuid) -> Result<Option<CreationState>, DbError> {
        let result =
            sqlx::query_as::<_, CreationState>("SELECT * FROM creation_states WHERE novel_id = ?1")
                .bind(novel_id)
                .fetch_optional(&self.pool)
                .await?;

        Ok(result)
    }

    /// 创建或更新指定小说的创作状态记录
    ///
    /// 使用 INSERT OR UPDATE 策略，如果记录已存在则更新 updated_at，否则插入新记录
    async fn upsert_state(&self, novel_id: &Uuid) -> Result<CreationState, DbError> {
        let now = Utc::now();

        let state = sqlx::query_as::<_, CreationState>(
            "INSERT INTO creation_states (novel_id, updated_at) 
             VALUES (?1, ?2)
             ON CONFLICT(novel_id) DO UPDATE SET updated_at = ?2
             RETURNING *",
        )
        .bind(novel_id)
        .bind(now)
        .fetch_one(&self.pool)
        .await?;

        tracing::info!("创作状态记录已更新: novel_id={}", novel_id);
        Ok(state)
    }
}
