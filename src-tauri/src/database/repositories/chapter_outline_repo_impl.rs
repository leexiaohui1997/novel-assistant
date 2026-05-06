use async_trait::async_trait;
use chrono::Utc;
use sqlx::SqlitePool;
use uuid::Uuid;

use crate::database::error::DbError;
use crate::database::models::chapter_outline::ChapterOutline;
use crate::database::models::chapter_outline::ChapterOutlineWithCharacters;
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
                "UPDATE chapter_outlines SET positioning = ?1, plot = ?2, updated_at = ?3 WHERE novel_id = ?4 AND chapter_id IS NOT DISTINCT FROM ?5",
            )
            .bind(&outline.positioning)
            .bind(&outline.plot)
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
                "INSERT INTO chapter_outlines (novel_id, chapter_id, positioning, plot, created_at, updated_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6) RETURNING id",
            )
            .bind(&outline.novel_id)
            .bind(&outline.chapter_id)
            .bind(&outline.positioning)
            .bind(&outline.plot)
            .bind(now)
            .bind(now)
            .fetch_one(&self.pool)
            .await?;

            Ok(ChapterOutline {
                id,
                novel_id: outline.novel_id,
                chapter_id: outline.chapter_id,
                positioning: outline.positioning.clone(),
                plot: outline.plot.clone(),
                created_at: now,
                updated_at: now,
            })
        }
    }

    async fn find_character_ids(&self, outline_id: i64) -> Result<Vec<Uuid>, DbError> {
        let ids: Vec<Uuid> = sqlx::query_scalar(
            "SELECT character_id FROM chapter_outline_characters WHERE chapter_outline_id = ?1",
        )
        .bind(outline_id)
        .fetch_all(&self.pool)
        .await?;
        Ok(ids)
    }

    async fn sync_characters(
        &self,
        outline_id: i64,
        character_ids: &[Uuid],
    ) -> Result<(), DbError> {
        // 先删后插，事务保证原子性
        let mut tx = self.pool.begin().await?;

        sqlx::query("DELETE FROM chapter_outline_characters WHERE chapter_outline_id = ?1")
            .bind(outline_id)
            .execute(&mut *tx)
            .await?;

        for cid in character_ids {
            sqlx::query(
                "INSERT INTO chapter_outline_characters (chapter_outline_id, character_id) VALUES (?1, ?2)",
            )
            .bind(outline_id)
            .bind(cid)
            .execute(&mut *tx)
            .await?;
        }

        tx.commit().await?;
        Ok(())
    }

    async fn find_with_characters(
        &self,
        novel_id: &Uuid,
        chapter_id: Option<&Uuid>,
    ) -> Result<Option<ChapterOutlineWithCharacters>, DbError> {
        let outline = self.find_by_novel_and_chapter(novel_id, chapter_id).await?;

        match outline {
            Some(o) => {
                let character_ids = self.find_character_ids(o.id).await?;
                Ok(Some(ChapterOutlineWithCharacters {
                    outline: o,
                    character_ids,
                }))
            }
            None => Ok(None),
        }
    }
}
