use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::SqlitePool;
use uuid::Uuid;

use crate::database::error::DbError;
use crate::database::models::chapter_version::ChapterVersion;

/// 单篇章节最多保存的历史版本数上限
pub const MAX_CHAPTER_VERSIONS: i64 = 50;

#[async_trait]
pub trait ChapterVersionRepository {
    /// 按 `chapter_id` 查询全部历史版本，按 `saved_at` 倒序返回。
    async fn list_by_chapter(&self, chapter_id: Uuid) -> Result<Vec<ChapterVersion>, DbError>;
}

pub struct SqliteChapterVersionRepository {
    pool: SqlitePool,
}

impl SqliteChapterVersionRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    /// 事务内：插入一条版本快照。
    ///
    /// - `saved_at`：应传入快照时刻章节的 `updated_at`
    /// - `created_at`：自动填充为当前 UTC 时间
    pub async fn insert_version_in_tx(
        tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
        chapter_id: Uuid,
        title: &str,
        content: &str,
        word_count: i64,
        saved_at: DateTime<Utc>,
    ) -> Result<(), DbError> {
        let id = Uuid::new_v4();
        let now = Utc::now();
        sqlx::query(
            "INSERT INTO chapter_versions
             (id, chapter_id, title, content, word_count, saved_at, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        )
        .bind(id)
        .bind(chapter_id)
        .bind(title)
        .bind(content)
        .bind(word_count)
        .bind(saved_at)
        .bind(now)
        .execute(tx.as_mut())
        .await?;
        Ok(())
    }

    /// 事务内：统计指定章节的版本总数。
    pub async fn count_by_chapter_in_tx(
        tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
        chapter_id: Uuid,
    ) -> Result<i64, DbError> {
        let count: i64 =
            sqlx::query_scalar("SELECT COUNT(*) FROM chapter_versions WHERE chapter_id = ?1")
                .bind(chapter_id)
                .fetch_one(tx.as_mut())
                .await?;
        Ok(count)
    }

    /// 事务内：删除指定章节中 `saved_at` 最早的一条版本。
    pub async fn delete_oldest_in_tx(
        tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
        chapter_id: Uuid,
    ) -> Result<(), DbError> {
        sqlx::query(
            "DELETE FROM chapter_versions
             WHERE id = (
                 SELECT id FROM chapter_versions
                 WHERE chapter_id = ?1
                 ORDER BY saved_at ASC LIMIT 1
             )",
        )
        .bind(chapter_id)
        .execute(tx.as_mut())
        .await?;
        Ok(())
    }

    /// 事务内：超限时裁剪最旧版本，保证不超过 `MAX_CHAPTER_VERSIONS`。
    pub async fn prune_if_exceed_in_tx(
        tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
        chapter_id: Uuid,
    ) -> Result<(), DbError> {
        let count = Self::count_by_chapter_in_tx(tx, chapter_id).await?;
        if count > MAX_CHAPTER_VERSIONS {
            Self::delete_oldest_in_tx(tx, chapter_id).await?;
        }
        Ok(())
    }

    /// 事务内：保存快照 + 超限裁剪（组合快捷方法）。
    pub async fn save_snapshot_in_tx(
        tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
        chapter_id: Uuid,
        title: &str,
        content: &str,
        word_count: i64,
        saved_at: DateTime<Utc>,
    ) -> Result<(), DbError> {
        Self::insert_version_in_tx(tx, chapter_id, title, content, word_count, saved_at).await?;
        Self::prune_if_exceed_in_tx(tx, chapter_id).await?;
        Ok(())
    }
}

#[async_trait]
impl ChapterVersionRepository for SqliteChapterVersionRepository {
    async fn list_by_chapter(&self, chapter_id: Uuid) -> Result<Vec<ChapterVersion>, DbError> {
        let versions = sqlx::query_as::<_, ChapterVersion>(
            "SELECT * FROM chapter_versions
             WHERE chapter_id = ?1
             ORDER BY saved_at DESC",
        )
        .bind(chapter_id)
        .fetch_all(&self.pool)
        .await?;
        Ok(versions)
    }
}
