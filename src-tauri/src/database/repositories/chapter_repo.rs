use async_trait::async_trait;
use chrono::Utc;
use sqlx::SqlitePool;
use uuid::Uuid;

use crate::database::error::DbError;
use crate::database::models::chapter::{
    Chapter, NewChapter, NewVolume, UpdateChapter, UpdateVolume, Volume,
};

#[async_trait]
pub trait ChapterRepository {
    /// 创建分卷。
    async fn create_volume(&self, payload: &NewVolume) -> Result<Volume, DbError>;

    /// 编辑分卷基础信息（名称、序号）。
    async fn update_volume(
        &self,
        volume_id: i64,
        payload: &UpdateVolume,
    ) -> Result<Volume, DbError>;

    /// 删除分卷。
    ///
    /// 业务约束：若分卷下已关联章节，则禁止删除。
    async fn delete_volume(&self, novel_id: Uuid, volume_id: i64) -> Result<(), DbError>;

    /// 创建章节。
    ///
    /// 业务约束：章节初始序号固定为 -1（草稿）；正文写入时同步统计字数。
    async fn create_chapter(
        &self,
        novel_id: Uuid,
        payload: &NewChapter,
    ) -> Result<Chapter, DbError>;

    /// 编辑章节。
    ///
    /// 业务约束：正文更新时重算字数；分卷关联采用“先删后建”的方式更新。
    async fn update_chapter(
        &self,
        chapter_id: Uuid,
        payload: &UpdateChapter,
    ) -> Result<Chapter, DbError>;

    /// 删除章节，同时清理章节与分卷的关联关系。
    async fn delete_chapter(&self, novel_id: Uuid, chapter_id: Uuid) -> Result<(), DbError>;
}

pub struct SqliteChapterRepository {
    pool: SqlitePool,
}

impl SqliteChapterRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    /// 按字符数统计字数。
    ///
    /// 说明：当前采用字符计数，适配中英文混排场景；如需按词统计可在此统一替换策略。
    fn count_words(content: &str) -> i64 {
        content.chars().count() as i64
    }
}

#[async_trait]
impl ChapterRepository for SqliteChapterRepository {
    async fn create_volume(&self, payload: &NewVolume) -> Result<Volume, DbError> {
        let now = Utc::now();

        let volume = sqlx::query_as::<_, Volume>(
            "INSERT INTO volumes (novel_id, name, sequence, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5)
             RETURNING *",
        )
        .bind(payload.novel_id)
        .bind(&payload.name)
        .bind(payload.sequence)
        .bind(now)
        .bind(now)
        .fetch_one(&self.pool)
        .await?;

        Ok(volume)
    }

    async fn update_volume(
        &self,
        volume_id: i64,
        payload: &UpdateVolume,
    ) -> Result<Volume, DbError> {
        let now = Utc::now();

        let volume = sqlx::query_as::<_, Volume>(
            "UPDATE volumes
             SET name = ?1, sequence = ?2, updated_at = ?3
             WHERE id = ?4
             RETURNING *",
        )
        .bind(&payload.name)
        .bind(payload.sequence)
        .bind(now)
        .bind(volume_id)
        .fetch_one(&self.pool)
        .await?;

        Ok(volume)
    }

    async fn delete_volume(&self, novel_id: Uuid, volume_id: i64) -> Result<(), DbError> {
        // 删除前先判断是否存在章节关联：有则直接阻断。
        let linked_count: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM volume_chapters WHERE novel_id = ?1 AND volume_id = ?2",
        )
        .bind(novel_id)
        .bind(volume_id)
        .fetch_one(&self.pool)
        .await?;

        if linked_count > 0 {
            return Err(DbError::Business(
                "当前分卷下存在章节，无法删除".to_string(),
            ));
        }

        sqlx::query("DELETE FROM volumes WHERE id = ?1 AND novel_id = ?2")
            .bind(volume_id)
            .bind(novel_id)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    async fn create_chapter(
        &self,
        novel_id: Uuid,
        payload: &NewChapter,
    ) -> Result<Chapter, DbError> {
        let mut tx = self.pool.begin().await?;
        let id = Uuid::new_v4();
        let now = Utc::now();
        let word_count = Self::count_words(&payload.content);

        // 新建章节时，序号固定为 -1（草稿）。
        let chapter = sqlx::query_as::<_, Chapter>(
            "INSERT INTO chapters (id, novel_id, title, content, word_count, sequence, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
             RETURNING *",
        )
        .bind(id)
        .bind(novel_id)
        .bind(&payload.title)
        .bind(&payload.content)
        .bind(word_count)
        .bind(-1_i64)
        .bind(now)
        .bind(now)
        .fetch_one(tx.as_mut())
        .await?;

        if let Some(volume_id) = payload.volume_id {
            // 仅当前端传入分卷时才建立关联。
            sqlx::query(
                "INSERT INTO volume_chapters (novel_id, volume_id, chapter_id)
                 VALUES (?1, ?2, ?3)",
            )
            .bind(novel_id)
            .bind(volume_id)
            .bind(id)
            .execute(tx.as_mut())
            .await?;
        }

        tx.commit().await?;
        Ok(chapter)
    }

    async fn update_chapter(
        &self,
        chapter_id: Uuid,
        payload: &UpdateChapter,
    ) -> Result<Chapter, DbError> {
        let mut tx = self.pool.begin().await?;
        let now = Utc::now();
        let word_count = Self::count_words(&payload.content);

        // sequence 为空时保持原值不变。
        let chapter = sqlx::query_as::<_, Chapter>(
            "UPDATE chapters
             SET title = ?1, content = ?2, word_count = ?3, sequence = COALESCE(?4, sequence), updated_at = ?5
             WHERE id = ?6
             RETURNING *",
        )
        .bind(&payload.title)
        .bind(&payload.content)
        .bind(word_count)
        .bind(payload.sequence)
        .bind(now)
        .bind(chapter_id)
        .fetch_one(tx.as_mut())
        .await?;

        // 更新分卷关联采用“先清理再重建”，保证最终只有一条有效关联。
        sqlx::query("DELETE FROM volume_chapters WHERE chapter_id = ?1")
            .bind(chapter_id)
            .execute(tx.as_mut())
            .await?;

        if let Some(volume_id) = payload.volume_id {
            sqlx::query(
                "INSERT INTO volume_chapters (novel_id, volume_id, chapter_id)
                 VALUES (?1, ?2, ?3)",
            )
            .bind(chapter.novel_id)
            .bind(volume_id)
            .bind(chapter.id)
            .execute(tx.as_mut())
            .await?;
        }

        tx.commit().await?;
        Ok(chapter)
    }

    async fn delete_chapter(&self, novel_id: Uuid, chapter_id: Uuid) -> Result<(), DbError> {
        let mut tx = self.pool.begin().await?;

        // 先删关联，再删主记录，避免留下脏数据。
        sqlx::query("DELETE FROM volume_chapters WHERE chapter_id = ?1")
            .bind(chapter_id)
            .execute(tx.as_mut())
            .await?;

        sqlx::query("DELETE FROM chapters WHERE id = ?1 AND novel_id = ?2")
            .bind(chapter_id)
            .bind(novel_id)
            .execute(tx.as_mut())
            .await?;

        tx.commit().await?;
        Ok(())
    }
}
