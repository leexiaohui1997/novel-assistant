use async_trait::async_trait;
use chrono::Utc;
use sqlx::SqlitePool;
use uuid::Uuid;

use crate::database::error::DbError;
use crate::database::models::provider::{NewProvider, Provider, UpdateProvider};
use crate::utils::pagination::{query_with_pagination, PaginatedResult, PaginationParams};

#[async_trait]
pub trait ProviderRepository {
    async fn create(&self, provider: &NewProvider) -> Result<Provider, DbError>;
    async fn find_with_pagination(
        &self,
        params: &PaginationParams,
    ) -> Result<PaginatedResult<Provider>, DbError>;
    async fn find_by_id(&self, id: Uuid) -> Result<Provider, DbError>;
    async fn update(&self, id: Uuid, provider: &UpdateProvider) -> Result<Provider, DbError>;
    async fn delete(&self, id: Uuid) -> Result<(), DbError>;
}

pub struct SqliteProviderRepository {
    pool: SqlitePool,
}

impl SqliteProviderRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl ProviderRepository for SqliteProviderRepository {
    /// 创建新供应商
    async fn create(&self, provider: &NewProvider) -> Result<Provider, DbError> {
        let id = Uuid::new_v4();
        let now = Utc::now();
        let is_enabled = provider.is_enabled.unwrap_or(true);

        let created = sqlx::query_as::<_, Provider>(
            "INSERT INTO ai_providers (id, name, base_url, api_key, is_enabled, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
             RETURNING *",
        )
        .bind(id)
        .bind(&provider.name)
        .bind(&provider.base_url)
        .bind(&provider.api_key)
        .bind(is_enabled)
        .bind(now)
        .bind(now)
        .fetch_one(&self.pool)
        .await?;

        tracing::info!("供应商创建成功: {} - {}", created.id, created.name);
        Ok(created)
    }

    /// 分页查询供应商列表
    async fn find_with_pagination(
        &self,
        params: &PaginationParams,
    ) -> Result<PaginatedResult<Provider>, DbError> {
        query_with_pagination::<Provider>(
            &self.pool,
            params,
            "SELECT COUNT(*) FROM ai_providers",
            "SELECT * FROM ai_providers ORDER BY created_at DESC LIMIT ? OFFSET ?",
        )
        .await
    }

    /// 根据 ID 查询供应商
    async fn find_by_id(&self, id: Uuid) -> Result<Provider, DbError> {
        sqlx::query_as::<_, Provider>("SELECT * FROM ai_providers WHERE id = ?1")
            .bind(id)
            .fetch_one(&self.pool)
            .await
            .map_err(Into::into)
    }

    /// 更新供应商信息
    async fn update(&self, id: Uuid, provider: &UpdateProvider) -> Result<Provider, DbError> {
        let now = Utc::now();
        let is_enabled = provider.is_enabled.unwrap_or(true);

        let updated = sqlx::query_as::<_, Provider>(
            "UPDATE ai_providers SET name = ?1, base_url = ?2, api_key = ?3, is_enabled = ?4, updated_at = ?5
             WHERE id = ?6
             RETURNING *",
        )
        .bind(&provider.name)
        .bind(&provider.base_url)
        .bind(&provider.api_key)
        .bind(is_enabled)
        .bind(now)
        .bind(id)
        .fetch_one(&self.pool)
        .await?;

        tracing::info!("供应商更新成功: {} - {}", updated.id, updated.name);
        Ok(updated)
    }

    /// 删除供应商（级联删除关联模型由数据库外键保证）
    async fn delete(&self, id: Uuid) -> Result<(), DbError> {
        let result = sqlx::query("DELETE FROM ai_providers WHERE id = ?1")
            .bind(id)
            .execute(&self.pool)
            .await?;

        if result.rows_affected() == 0 {
            return Err(DbError::Business(format!("供应商不存在: {}", id)));
        }

        tracing::info!("供应商删除成功: {}", id);
        Ok(())
    }
}
