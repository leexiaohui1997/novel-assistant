use async_trait::async_trait;
use sqlx::SqlitePool;

use crate::database::error::DbError;
use crate::database::models::tag::Tag;

/// 标签仓储 trait
#[async_trait]
pub trait TagRepository {
    /// 获取所有标签
    async fn find_all(&self) -> Result<Vec<Tag>, DbError>;

    /// 根据目标读者筛选标签
    async fn find_by_target_audience(&self, target_audience: &str) -> Result<Vec<Tag>, DbError>;

    /// 根据 ID 列表获取标签
    async fn find_by_ids(&self, ids: &[i64]) -> Result<Vec<Tag>, DbError>;
}

/// SQLite 标签仓储实现
pub struct SqliteTagRepository {
    pool: SqlitePool,
}

impl SqliteTagRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl TagRepository for SqliteTagRepository {
    async fn find_all(&self) -> Result<Vec<Tag>, DbError> {
        let tags = sqlx::query_as::<_, Tag>("SELECT * FROM tags ORDER BY tag_type, id")
            .fetch_all(&self.pool)
            .await?;
        Ok(tags)
    }

    async fn find_by_target_audience(&self, target_audience: &str) -> Result<Vec<Tag>, DbError> {
        let tags = sqlx::query_as::<_, Tag>(
            "SELECT * FROM tags WHERE target_audience IN (?1, 'both') ORDER BY tag_type, id",
        )
        .bind(target_audience)
        .fetch_all(&self.pool)
        .await?;
        Ok(tags)
    }

    async fn find_by_ids(&self, ids: &[i64]) -> Result<Vec<Tag>, DbError> {
        if ids.is_empty() {
            return Ok(vec![]);
        }

        let placeholders = ids.iter().map(|_| "?").collect::<Vec<_>>().join(", ");
        let query = format!(
            "SELECT * FROM tags WHERE id IN ({}) ORDER BY id",
            placeholders
        );

        let mut db_query = sqlx::query_as::<_, Tag>(&query);
        for id in ids {
            db_query = db_query.bind(id);
        }

        let tags = db_query.fetch_all(&self.pool).await?;
        Ok(tags)
    }
}
