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

    /// 根据目标读者和标签类型筛选标签
    async fn find_by_audience_and_type(
        &self,
        target_audience: &str,
        tag_type: &str,
    ) -> Result<Vec<Tag>, DbError>;

    /// 根据标签名称列表获取标签
    async fn find_by_names(&self, names: &[&str]) -> Result<Vec<Tag>, DbError>;
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

    async fn find_by_audience_and_type(
        &self,
        target_audience: &str,
        tag_type: &str,
    ) -> Result<Vec<Tag>, DbError> {
        let tags = sqlx::query_as::<_, Tag>(
            "SELECT * FROM tags WHERE target_audience IN (?1, 'both') AND tag_type = ?2 ORDER BY id",
        )
        .bind(target_audience)
        .bind(tag_type)
        .fetch_all(&self.pool)
        .await?;
        Ok(tags)
    }

    async fn find_by_names(&self, names: &[&str]) -> Result<Vec<Tag>, DbError> {
        if names.is_empty() {
            return Ok(vec![]);
        }

        let placeholders = names.iter().map(|_| "?").collect::<Vec<_>>().join(", ");
        let query = format!("SELECT * FROM tags WHERE name IN ({})", placeholders);

        let mut db_query = sqlx::query_as::<_, Tag>(&query);
        for name in names {
            db_query = db_query.bind(name);
        }

        let tags = db_query.fetch_all(&self.pool).await?;

        // 按照输入名称的顺序排序
        let name_to_tag: std::collections::HashMap<&str, &Tag> =
            tags.iter().map(|tag| (tag.name.as_str(), tag)).collect();

        let mut ordered_tags = Vec::new();
        for name in names {
            if let Some(tag) = name_to_tag.get(name) {
                ordered_tags.push((*tag).clone());
            }
        }

        Ok(ordered_tags)
    }
}
