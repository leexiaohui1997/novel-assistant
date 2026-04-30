use async_trait::async_trait;
use chrono::Utc;
use sqlx::SqlitePool;
use uuid::Uuid;

use crate::database::error::DbError;
use crate::database::models::model::{Model, ModelWithProvider, NewModelItem};
use crate::utils::pagination::{query_with_pagination, PaginatedResult, PaginationParams};

/// AI 模型仓储 trait
#[async_trait]
pub trait ModelRepository {
    /// 批量添加模型（同一供应商下）
    async fn batch_create(
        &self,
        provider_id: Uuid,
        models: &[NewModelItem],
    ) -> Result<Vec<Model>, DbError>;

    /// 分页查询模型列表（带供应商名称）
    async fn find_with_pagination(
        &self,
        params: &PaginationParams,
    ) -> Result<PaginatedResult<ModelWithProvider>, DbError>;

    /// 删除模型
    async fn delete(&self, id: Uuid) -> Result<(), DbError>;

    /// 切换启用状态
    async fn toggle_enabled(&self, id: Uuid, is_enabled: bool) -> Result<Model, DbError>;
}

/// SQLite 模型仓储实现
pub struct SqliteModelRepository {
    pool: SqlitePool,
}

impl SqliteModelRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl ModelRepository for SqliteModelRepository {
    /// 批量添加模型
    /// 使用事务保证原子性：全部成功或全部失败
    async fn batch_create(
        &self,
        provider_id: Uuid,
        models: &[NewModelItem],
    ) -> Result<Vec<Model>, DbError> {
        if models.is_empty() {
            return Ok(vec![]);
        }

        let mut tx = self.pool.begin().await?;
        let now = Utc::now();
        let mut created = Vec::with_capacity(models.len());

        for item in models {
            let id = Uuid::new_v4();
            let model = sqlx::query_as::<_, Model>(
                "INSERT INTO ai_models (id, provider_id, model_id, alias, is_default, is_enabled, created_at, updated_at)
                 VALUES (?1, ?2, ?3, ?4, 0, 1, ?5, ?5)
                 RETURNING *",
            )
            .bind(id)
            .bind(provider_id)
            .bind(&item.model_id)
            .bind(&item.alias)
            .bind(now)
            .fetch_one(tx.as_mut())
            .await?;

            created.push(model);
        }

        tx.commit().await?;
        tracing::info!(
            "批量添加模型成功: provider_id={}, count={}",
            provider_id,
            created.len()
        );
        Ok(created)
    }

    /// 分页查询模型列表（带供应商名称）
    async fn find_with_pagination(
        &self,
        params: &PaginationParams,
    ) -> Result<PaginatedResult<ModelWithProvider>, DbError> {
        query_with_pagination::<ModelWithProvider>(
            &self.pool,
            params,
            "SELECT COUNT(*) FROM ai_models",
            "SELECT m.id, m.provider_id, p.name AS provider_name, m.model_id, m.alias, \
             m.is_default, m.is_enabled, m.created_at, m.updated_at \
             FROM ai_models m \
             INNER JOIN ai_providers p ON p.id = m.provider_id \
             ORDER BY m.created_at DESC \
             LIMIT ? OFFSET ?",
        )
        .await
    }

    /// 删除模型
    async fn delete(&self, id: Uuid) -> Result<(), DbError> {
        let result = sqlx::query("DELETE FROM ai_models WHERE id = ?1")
            .bind(id)
            .execute(&self.pool)
            .await?;

        if result.rows_affected() == 0 {
            return Err(DbError::Business(format!("模型不存在: {}", id)));
        }

        tracing::info!("模型删除成功: {}", id);
        Ok(())
    }

    /// 切换启用状态
    async fn toggle_enabled(&self, id: Uuid, is_enabled: bool) -> Result<Model, DbError> {
        let now = Utc::now();
        let updated = sqlx::query_as::<_, Model>(
            "UPDATE ai_models SET is_enabled = ?1, updated_at = ?2 WHERE id = ?3 RETURNING *",
        )
        .bind(is_enabled)
        .bind(now)
        .bind(id)
        .fetch_one(&self.pool)
        .await?;

        tracing::info!("模型启用状态更新: {} -> {}", updated.id, is_enabled);
        Ok(updated)
    }
}
