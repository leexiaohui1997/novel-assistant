use std::collections::HashSet;

use async_trait::async_trait;
use chrono::Utc;
use sqlx::SqlitePool;
use uuid::Uuid;

use crate::database::error::DbError;
use crate::database::models::chapter::{
    Chapter, ChapterQuery, NewChapter, NewVolume, UpdateChapter, UpdateVolume, Volume, VolumeUpsert,
};
use crate::utils::pagination::{PaginatedResult, PaginationParams};

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

    /// 查询指定小说下的全部分卷，按 `sequence` 升序返回。
    async fn get_volumes(&self, novel_id: Uuid) -> Result<Vec<Volume>, DbError>;

    /// 批量编辑分卷（事务写入）。
    ///
    /// 业务约束：
    /// - 含 `id` 视为更新，不含 `id` 视为新增；
    /// - 已有但不在请求列表中的分卷视为删除；
    /// - 删除前若存在关联章节则直接报错并回滚整个事务。
    async fn batch_update_volumes(
        &self,
        novel_id: Uuid,
        payload: &[VolumeUpsert],
    ) -> Result<Vec<Volume>, DbError>;

    /// 创建章节。
    ///
    /// 业务约束：
    /// - `payload.sequence` 为 `None` 时，序号默认写入 -1（草稿）；
    /// - `payload.sequence` 为 `Some(n)`（n >= 0）时，直接按正式章节写入；
    /// - 正文写入时同步统计字数。
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

    /// 分页查询章节。
    ///
    /// 业务约束详见 `ChapterQuery` 说明。
    async fn get_chapters_with_pagination(
        &self,
        novel_id: Uuid,
        query: &ChapterQuery,
        pagination: &PaginationParams,
    ) -> Result<PaginatedResult<Chapter>, DbError>;

    /// 查询指定分卷下非草稿章节的最大 `sequence`。
    ///
    /// 业务约束：
    /// - `volume_id = 1`（默认分卷）：含关联到分卷 1 及无分卷关联的非草稿章节；
    /// - `volume_id = N`（N != 1）：仅含关联到该分卷的非草稿章节；
    /// - 若分卷下无任何非草稿章节，返回 `None`。
    async fn get_max_sequence_in_volume(
        &self,
        novel_id: Uuid,
        volume_id: i64,
    ) -> Result<Option<i64>, DbError>;
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

    /// 将前端传入的 `sort_field` 映射为合法的数据库列名（白名单，防注入）。
    ///
    /// 未传或非法值时返回默认列：草稿模式默认 `created_at`，非草稿默认 `sequence`。
    fn resolve_sort_column(query: &ChapterQuery) -> String {
        let default_col = if query.is_draft {
            "created_at"
        } else {
            "sequence"
        };
        match query.sort_field.as_deref() {
            Some("sequence") => "sequence".to_string(),
            Some("title") => "title".to_string(),
            Some("createdAt") | Some("created_at") => "created_at".to_string(),
            Some("updatedAt") | Some("updated_at") => "updated_at".to_string(),
            _ => default_col.to_string(),
        }
    }

    /// 将前端传入的 `sort_order` 映射为 ASC / DESC。
    ///
    /// 未传时返回默认：`DESC`（章节序号与创建时间默认倒排）。
    fn resolve_sort_direction(query: &ChapterQuery) -> String {
        match query.sort_order.as_deref() {
            Some("asc") | Some("ASC") => "ASC".to_string(),
            _ => "DESC".to_string(),
        }
    }

    /// 查询草稿章节（sequence = -1）。
    async fn query_draft_chapters(
        &self,
        novel_id: Uuid,
        sort_col: &str,
        sort_dir: &str,
        pagination: &PaginationParams,
    ) -> Result<PaginatedResult<Chapter>, DbError> {
        let total: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM chapters WHERE novel_id = ?1 AND sequence = -1",
        )
        .bind(novel_id)
        .fetch_one(&self.pool)
        .await?;

        let data_sql = format!(
            "SELECT * FROM chapters
             WHERE novel_id = ?1 AND sequence = -1
             ORDER BY {} {} LIMIT ?2 OFFSET ?3",
            sort_col, sort_dir
        );
        let offset = (pagination.page - 1) * pagination.page_size;
        let data = sqlx::query_as::<_, Chapter>(&data_sql)
            .bind(novel_id)
            .bind(pagination.page_size)
            .bind(offset)
            .fetch_all(&self.pool)
            .await?;

        Ok(PaginatedResult { data, total })
    }

    /// 查询默认分卷（序号=1）下的非草稿章节，同时包含无分卷关联的章节。
    async fn query_default_volume_chapters(
        &self,
        novel_id: Uuid,
        sort_col: &str,
        sort_dir: &str,
        pagination: &PaginationParams,
    ) -> Result<PaginatedResult<Chapter>, DbError> {
        // 过滤条件：非草稿 且（关联到 volume 1 或 无任何 volume 关联）
        let where_sql = "c.novel_id = ?1 AND c.sequence >= 0
             AND (vc.volume_id = 1 OR vc.volume_id IS NULL)";

        let count_sql = format!(
            "SELECT COUNT(*) FROM chapters c
             LEFT JOIN volume_chapters vc ON vc.chapter_id = c.id
             WHERE {}",
            where_sql
        );
        let total: i64 = sqlx::query_scalar(&count_sql)
            .bind(novel_id)
            .fetch_one(&self.pool)
            .await?;

        let data_sql = format!(
            "SELECT c.* FROM chapters c
             LEFT JOIN volume_chapters vc ON vc.chapter_id = c.id
             WHERE {}
             ORDER BY c.{} {} LIMIT ?2 OFFSET ?3",
            where_sql, sort_col, sort_dir
        );
        let offset = (pagination.page - 1) * pagination.page_size;
        let data = sqlx::query_as::<_, Chapter>(&data_sql)
            .bind(novel_id)
            .bind(pagination.page_size)
            .bind(offset)
            .fetch_all(&self.pool)
            .await?;

        Ok(PaginatedResult { data, total })
    }

    /// 查询非草稿章节；有 `volume_id` 时按分卷过滤，无则查全部非草稿章节。
    async fn query_non_draft_chapters(
        &self,
        novel_id: Uuid,
        volume_id: Option<i64>,
        sort_col: &str,
        sort_dir: &str,
        pagination: &PaginationParams,
    ) -> Result<PaginatedResult<Chapter>, DbError> {
        match volume_id {
            Some(vid) => {
                self.query_by_volume(novel_id, vid, sort_col, sort_dir, pagination)
                    .await
            }
            None => {
                self.query_all_non_draft(novel_id, sort_col, sort_dir, pagination)
                    .await
            }
        }
    }

    /// 按指定分卷查询非草稿章节。
    async fn query_by_volume(
        &self,
        novel_id: Uuid,
        volume_id: i64,
        sort_col: &str,
        sort_dir: &str,
        pagination: &PaginationParams,
    ) -> Result<PaginatedResult<Chapter>, DbError> {
        let total: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM chapters c
             INNER JOIN volume_chapters vc ON vc.chapter_id = c.id
             WHERE c.novel_id = ?1 AND c.sequence >= 0 AND vc.volume_id = ?2",
        )
        .bind(novel_id)
        .bind(volume_id)
        .fetch_one(&self.pool)
        .await?;

        let data_sql = format!(
            "SELECT c.* FROM chapters c
             INNER JOIN volume_chapters vc ON vc.chapter_id = c.id
             WHERE c.novel_id = ?1 AND c.sequence >= 0 AND vc.volume_id = ?2
             ORDER BY c.{} {} LIMIT ?3 OFFSET ?4",
            sort_col, sort_dir
        );
        let offset = (pagination.page - 1) * pagination.page_size;
        let data = sqlx::query_as::<_, Chapter>(&data_sql)
            .bind(novel_id)
            .bind(volume_id)
            .bind(pagination.page_size)
            .bind(offset)
            .fetch_all(&self.pool)
            .await?;

        Ok(PaginatedResult { data, total })
    }

    /// 查询小说下全部非草稿章节（无分卷过滤）。
    async fn query_all_non_draft(
        &self,
        novel_id: Uuid,
        sort_col: &str,
        sort_dir: &str,
        pagination: &PaginationParams,
    ) -> Result<PaginatedResult<Chapter>, DbError> {
        let total: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM chapters WHERE novel_id = ?1 AND sequence >= 0",
        )
        .bind(novel_id)
        .fetch_one(&self.pool)
        .await?;

        let data_sql = format!(
            "SELECT * FROM chapters
             WHERE novel_id = ?1 AND sequence >= 0
             ORDER BY {} {} LIMIT ?2 OFFSET ?3",
            sort_col, sort_dir
        );
        let offset = (pagination.page - 1) * pagination.page_size;
        let data = sqlx::query_as::<_, Chapter>(&data_sql)
            .bind(novel_id)
            .bind(pagination.page_size)
            .bind(offset)
            .fetch_all(&self.pool)
            .await?;

        Ok(PaginatedResult { data, total })
    }

    /// 查询当前小说下已有分卷的 ID 集合（事务内）。
    async fn collect_existing_volume_ids(
        tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
        novel_id: Uuid,
    ) -> Result<HashSet<i64>, DbError> {
        let rows: Vec<(i64,)> = sqlx::query_as("SELECT id FROM volumes WHERE novel_id = ?1")
            .bind(novel_id)
            .fetch_all(tx.as_mut())
            .await?;

        Ok(rows.into_iter().map(|(id,)| id).collect())
    }

    /// 事务内删除分卷；若该分卷下仍存在关联章节则直接报错中止事务。
    async fn delete_volume_in_tx(
        tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
        novel_id: Uuid,
        volume_id: i64,
    ) -> Result<(), DbError> {
        let linked_count: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM volume_chapters WHERE novel_id = ?1 AND volume_id = ?2",
        )
        .bind(novel_id)
        .bind(volume_id)
        .fetch_one(tx.as_mut())
        .await?;

        if linked_count > 0 {
            return Err(DbError::Business(format!(
                "分卷（id={}）下存在章节，无法删除",
                volume_id
            )));
        }

        sqlx::query("DELETE FROM volumes WHERE id = ?1 AND novel_id = ?2")
            .bind(volume_id)
            .bind(novel_id)
            .execute(tx.as_mut())
            .await?;

        Ok(())
    }

    /// 事务内查询小说下 sequence 最小的分卷 id（用于首卷孤儿归属）。
    async fn find_first_volume_id_in_tx(
        tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
        novel_id: Uuid,
    ) -> Result<Option<i64>, DbError> {
        let row: Option<(i64,)> = sqlx::query_as(
            "SELECT id FROM volumes WHERE novel_id = ?1 ORDER BY sequence ASC LIMIT 1",
        )
        .bind(novel_id)
        .fetch_optional(tx.as_mut())
        .await?;
        Ok(row.map(|(id,)| id))
    }

    /// 事务内将孤儿章节（非草稿且无 volume_chapters 关联）批量归属到指定分卷。
    ///
    /// 用于"小说首次创建真实分卷"场景，确保历史数据一致性。
    async fn link_orphan_chapters_in_tx(
        tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
        novel_id: Uuid,
        volume_id: i64,
    ) -> Result<(), DbError> {
        sqlx::query(
            "INSERT INTO volume_chapters (novel_id, volume_id, chapter_id)
             SELECT ?1, ?2, c.id
             FROM chapters c
             LEFT JOIN volume_chapters vc ON vc.chapter_id = c.id
             WHERE c.novel_id = ?1 AND c.sequence >= 0 AND vc.chapter_id IS NULL",
        )
        .bind(novel_id)
        .bind(volume_id)
        .execute(tx.as_mut())
        .await?;
        Ok(())
    }

    /// 事务内执行单条分卷的新增或更新。
    async fn upsert_volume_in_tx(
        tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
        novel_id: Uuid,
        item: &VolumeUpsert,
    ) -> Result<(), DbError> {
        let now = Utc::now();

        match item.id {
            Some(id) => {
                sqlx::query(
                    "UPDATE volumes
                     SET name = ?1, sequence = ?2, updated_at = ?3
                     WHERE id = ?4 AND novel_id = ?5",
                )
                .bind(&item.name)
                .bind(item.sequence)
                .bind(now)
                .bind(id)
                .bind(novel_id)
                .execute(tx.as_mut())
                .await?;
            }
            None => {
                sqlx::query(
                    "INSERT INTO volumes (novel_id, name, sequence, created_at, updated_at)
                     VALUES (?1, ?2, ?3, ?4, ?5)",
                )
                .bind(novel_id)
                .bind(&item.name)
                .bind(item.sequence)
                .bind(now)
                .bind(now)
                .execute(tx.as_mut())
                .await?;
            }
        }

        Ok(())
    }

    /// 查询默认分卷（volume_id=1）下的最大 sequence。
    ///
    /// 包含：关联到分卷 1 的非草稿章节 + 无 `volume_chapters` 关联的非草稿章节。
    async fn max_sequence_default_volume(&self, novel_id: Uuid) -> Result<Option<i64>, DbError> {
        let row: Option<(Option<i64>,)> = sqlx::query_as(
            "SELECT MAX(c.sequence) FROM chapters c
             LEFT JOIN volume_chapters vc ON vc.chapter_id = c.id
             WHERE c.novel_id = ?1 AND c.sequence >= 0
               AND (vc.volume_id = 1 OR vc.volume_id IS NULL)",
        )
        .bind(novel_id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.and_then(|(v,)| v))
    }

    /// 查询指定（非默认）分卷下的最大 sequence。
    async fn max_sequence_by_volume(
        &self,
        novel_id: Uuid,
        volume_id: i64,
    ) -> Result<Option<i64>, DbError> {
        let row: Option<(Option<i64>,)> = sqlx::query_as(
            "SELECT MAX(c.sequence) FROM chapters c
             INNER JOIN volume_chapters vc ON vc.chapter_id = c.id
             WHERE c.novel_id = ?1 AND c.sequence >= 0 AND vc.volume_id = ?2",
        )
        .bind(novel_id)
        .bind(volume_id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.and_then(|(v,)| v))
    }

    /// 事务内查询目标章节的 sequence 及所属分卷 ID。
    ///
    /// 返回 `(sequence, volume_id)`；`volume_id` 为 `None` 表示无分卷关联（归属默认分卷）。
    async fn fetch_chapter_volume_in_tx(
        tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
        novel_id: Uuid,
        chapter_id: Uuid,
    ) -> Result<(i64, Option<i64>), DbError> {
        let row: Option<(i64, Option<i64>)> = sqlx::query_as(
            "SELECT c.sequence, vc.volume_id
             FROM chapters c
             LEFT JOIN volume_chapters vc ON vc.chapter_id = c.id
             WHERE c.id = ?1 AND c.novel_id = ?2",
        )
        .bind(chapter_id)
        .bind(novel_id)
        .fetch_optional(tx.as_mut())
        .await?;

        row.ok_or_else(|| DbError::Business("章节不存在".to_string()))
    }

    /// 事务内查询指定分卷的最大 sequence（默认分卷含无关联章节）。
    async fn max_sequence_in_volume_in_tx(
        tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
        novel_id: Uuid,
        volume_id: i64,
    ) -> Result<Option<i64>, DbError> {
        if volume_id == 1 {
            let row: Option<(Option<i64>,)> = sqlx::query_as(
                "SELECT MAX(c.sequence) FROM chapters c
                 LEFT JOIN volume_chapters vc ON vc.chapter_id = c.id
                 WHERE c.novel_id = ?1 AND c.sequence >= 0
                   AND (vc.volume_id = 1 OR vc.volume_id IS NULL)",
            )
            .bind(novel_id)
            .fetch_optional(tx.as_mut())
            .await?;
            return Ok(row.and_then(|(v,)| v));
        }

        let row: Option<(Option<i64>,)> = sqlx::query_as(
            "SELECT MAX(c.sequence) FROM chapters c
             INNER JOIN volume_chapters vc ON vc.chapter_id = c.id
             WHERE c.novel_id = ?1 AND c.sequence >= 0 AND vc.volume_id = ?2",
        )
        .bind(novel_id)
        .bind(volume_id)
        .fetch_optional(tx.as_mut())
        .await?;
        Ok(row.and_then(|(v,)| v))
    }

    /// 事务内校验目标章节是否为所属分卷的末尾章节；非末尾则返回业务错误。
    async fn ensure_tail_chapter_in_tx(
        tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
        novel_id: Uuid,
        chapter_id: Uuid,
    ) -> Result<(), DbError> {
        let (sequence, volume_id) =
            Self::fetch_chapter_volume_in_tx(tx, novel_id, chapter_id).await?;

        if sequence < 0 {
            return Err(DbError::Business("只能删除最后一个章节".to_string()));
        }

        // 无分卷关联的章节归属默认分卷（id=1）
        let effective_volume_id = volume_id.unwrap_or(1);
        let max_seq = Self::max_sequence_in_volume_in_tx(tx, novel_id, effective_volume_id).await?;

        if max_seq != Some(sequence) {
            return Err(DbError::Business("只能删除最后一个章节".to_string()));
        }
        Ok(())
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

    async fn get_volumes(&self, novel_id: Uuid) -> Result<Vec<Volume>, DbError> {
        let volumes = sqlx::query_as::<_, Volume>(
            "SELECT * FROM volumes WHERE novel_id = ?1 ORDER BY sequence ASC",
        )
        .bind(novel_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(volumes)
    }

    async fn batch_update_volumes(
        &self,
        novel_id: Uuid,
        payload: &[VolumeUpsert],
    ) -> Result<Vec<Volume>, DbError> {
        let mut tx = self.pool.begin().await?;

        // 1) 收集当前小说下已有分卷 ID 集合，与请求列表中已标注 id 的做差集得到待删除集合
        let existing_ids = Self::collect_existing_volume_ids(&mut tx, novel_id).await?;
        let keep_ids: HashSet<i64> = payload.iter().filter_map(|v| v.id).collect();
        // 记录"是否为首卷创建"场景：提交前无任何真实分卷
        let is_first_volume_creation = existing_ids.is_empty() && !payload.is_empty();

        // 2) 删除不再保留的分卷（删除前校验无关联章节）
        for id in existing_ids.difference(&keep_ids) {
            Self::delete_volume_in_tx(&mut tx, novel_id, *id).await?;
        }

        // 3) 批量 upsert：含 id 走更新，不含 id 走新增
        for item in payload {
            Self::upsert_volume_in_tx(&mut tx, novel_id, item).await?;
        }

        // 4) 首卷创建时，将孤儿章节（非草稿且无分卷关联）归属到 sequence 最小的新分卷
        if is_first_volume_creation {
            if let Some(first_id) = Self::find_first_volume_id_in_tx(&mut tx, novel_id).await? {
                Self::link_orphan_chapters_in_tx(&mut tx, novel_id, first_id).await?;
            }
        }

        tx.commit().await?;

        // 5) 返回更新后的完整分卷列表
        self.get_volumes(novel_id).await
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

        // 未显式传入 sequence 时，按草稿（-1）落库；传入则直接作为正式章节序号。
        let sequence = payload.sequence.unwrap_or(-1);
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
        .bind(sequence)
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

        // 业务约束：仅允许删除分卷下的最后一个非草稿章节
        Self::ensure_tail_chapter_in_tx(&mut tx, novel_id, chapter_id).await?;

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

    async fn get_chapters_with_pagination(
        &self,
        novel_id: Uuid,
        query: &ChapterQuery,
        pagination: &PaginationParams,
    ) -> Result<PaginatedResult<Chapter>, DbError> {
        let sort_col = Self::resolve_sort_column(query);
        let sort_dir = Self::resolve_sort_direction(query);

        if query.is_draft {
            return self
                .query_draft_chapters(novel_id, &sort_col, &sort_dir, pagination)
                .await;
        }

        // volume_id = 1 时，需同时包含无分卷关联的非草稿章节
        if query.volume_id == Some(1) {
            return self
                .query_default_volume_chapters(novel_id, &sort_col, &sort_dir, pagination)
                .await;
        }

        self.query_non_draft_chapters(novel_id, query.volume_id, &sort_col, &sort_dir, pagination)
            .await
    }

    async fn get_max_sequence_in_volume(
        &self,
        novel_id: Uuid,
        volume_id: i64,
    ) -> Result<Option<i64>, DbError> {
        // volume_id = 1 是默认分卷：需包含无分卷关联的章节
        if volume_id == 1 {
            return self.max_sequence_default_volume(novel_id).await;
        }
        self.max_sequence_by_volume(novel_id, volume_id).await
    }
}
