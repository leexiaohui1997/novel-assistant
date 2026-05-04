use async_trait::async_trait;
use chrono::Utc;
use sqlx::SqlitePool;
use uuid::Uuid;

use crate::database::error::DbError;
use crate::database::models::chapter_outline::ChapterOutline;
use crate::database::repositories::chapter_outline_repo::ChapterOutlineRepository;

/// SQLite 章节大纲仓储实现
pub struct SqliteChapterOutlineRepository {
    pool: SqlitePool,
}

impl SqliteChapterOutlineRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl ChapterOutlineRepository for SqliteChapterOutlineRepository {
    async fn find_by_novel_and_chapter(
        &self,
        novel_id: &Uuid,
        chapter_id: Option<&Uuid>,
    ) -> Result<Option<ChapterOutline>, DbError> {
        // 使用 IS NOT DISTINCT FROM 统一处理 NULL 和非 NULL 情况
        let outline = sqlx::query_as::<_, ChapterOutline>(
            "SELECT * FROM chapter_outlines WHERE novel_id = ?1 AND chapter_id IS NOT DISTINCT FROM ?2",
        )
        .bind(novel_id)
        .bind(chapter_id)
        .fetch_optional(&self.pool)
        .await?;
        Ok(outline)
    }

    async fn upsert(&self, outline: &ChapterOutline) -> Result<ChapterOutline, DbError> {
        let now = Utc::now();

        // 检查是否已存在
        let existing = self
            .find_by_novel_and_chapter(&outline.novel_id, outline.chapter_id.as_ref())
            .await?;

        if let Some(_) = existing {
            // 更新现有记录
            sqlx::query(
                "UPDATE chapter_outlines SET positioning = ?1, updated_at = ?2 WHERE novel_id = ?3 AND chapter_id IS NOT DISTINCT FROM ?4",
            )
            .bind(&outline.positioning)
            .bind(now)
            .bind(&outline.novel_id)
            .bind(&outline.chapter_id)
            .execute(&self.pool)
            .await?;

            // 返回更新后的记录
            self.find_by_novel_and_chapter(&outline.novel_id, outline.chapter_id.as_ref())
                .await
                .map(|opt| opt.expect("Record should exist after update"))
        } else {
            // 创建新记录
            let id = sqlx::query_scalar::<_, i64>(
                "INSERT INTO chapter_outlines (novel_id, chapter_id, positioning, created_at, updated_at) VALUES (?1, ?2, ?3, ?4, ?5) RETURNING id",
            )
            .bind(&outline.novel_id)
            .bind(&outline.chapter_id)
            .bind(&outline.positioning)
            .bind(now)
            .bind(now)
            .fetch_one(&self.pool)
            .await?;

            Ok(ChapterOutline {
                id,
                novel_id: outline.novel_id,
                chapter_id: outline.chapter_id,
                positioning: outline.positioning.clone(),
                created_at: now,
                updated_at: now,
            })
        }
    }
}
