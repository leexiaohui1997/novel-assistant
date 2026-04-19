use async_trait::async_trait;
use chrono::Utc;
use sqlx::SqlitePool;
use uuid::Uuid;

use crate::database::error::DbError;
use crate::database::models::novel::{NewNovel, Novel};

#[async_trait]
pub trait NovelRepository {
    async fn create(&self, novel: &NewNovel) -> Result<Novel, DbError>;
    async fn find_all(&self) -> Result<Vec<Novel>, DbError>;
}

pub struct SqliteNovelRepository {
    pool: SqlitePool,
}

impl SqliteNovelRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl NovelRepository for SqliteNovelRepository {
    /// 创建新小说并关联标签
    ///
    /// 使用数据库事务确保小说主表插入与标签关联的原子性。
    /// 如果标签 ID 不存在，关联操作将被忽略（INSERT OR IGNORE）。
    async fn create(&self, novel: &NewNovel) -> Result<Novel, DbError> {
        let mut tx = self.pool.begin().await?;
        let id = Uuid::new_v4();
        let now = Utc::now();

        // 1. 插入小说主表
        let created_novel = sqlx::query_as::<_, Novel>(
            "INSERT INTO novels (id, title, target_reader, description, created_at, updated_at) 
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)
             RETURNING *",
        )
        .bind(id)
        .bind(&novel.title)
        .bind(&novel.target_reader)
        .bind(&novel.description)
        .bind(now)
        .bind(now)
        .fetch_one(tx.as_mut())
        .await?;

        // 2. 处理标签关联
        for tag_id in &novel.tag_ids {
            sqlx::query("INSERT OR IGNORE INTO novel_tags (novel_id, tag_id) VALUES (?1, ?2)")
                .bind(id)
                .bind(tag_id)
                .execute(tx.as_mut())
                .await?;
        }

        tx.commit().await?;
        tracing::info!(
            "小说及其标签创建成功: {} - {}",
            created_novel.id,
            created_novel.title
        );
        Ok(created_novel)
    }

    /// 获取所有小说列表
    ///
    /// 按创建时间倒序排列，用于前端作品管理页面的初始化展示。
    async fn find_all(&self) -> Result<Vec<Novel>, DbError> {
        let novels = sqlx::query_as::<_, Novel>("SELECT * FROM novels ORDER BY created_at DESC")
            .fetch_all(&self.pool)
            .await?;

        Ok(novels)
    }
}
