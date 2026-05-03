use async_trait::async_trait;
use chrono::Utc;
use sqlx::SqlitePool;
use uuid::Uuid;

use crate::database::error::DbError;
use crate::database::models::novel::{NewNovel, Novel, NovelWithTags, UpdateNovel};
use crate::database::models::tag::Tag;
use crate::utils::pagination::{query_with_pagination, PaginatedResult, PaginationParams};

/// 查询选项，控制是否解析关联的标签
#[derive(Debug, Clone, Default)]
pub struct QueryOptions {
    /// 是否加载关联的标签信息
    pub with_tags: bool,
}

/// novel_tags 关联行，用于接收 novel_id + tag_id 查询结果
#[derive(Debug, sqlx::FromRow)]
struct NovelTagRow {
    novel_id: Uuid,
    tag_id: i64,
}

#[async_trait]
pub trait NovelRepository {
    async fn create(&self, novel: &NewNovel) -> Result<Novel, DbError>;
    async fn find_all(&self, options: &QueryOptions) -> Result<Vec<NovelWithTags>, DbError>;
    async fn find_with_pagination(
        &self,
        params: &PaginationParams,
        options: &QueryOptions,
    ) -> Result<PaginatedResult<NovelWithTags>, DbError>;
    async fn find_by_id(&self, id: Uuid, options: &QueryOptions) -> Result<NovelWithTags, DbError>;
    async fn update(&self, id: Uuid, novel: &UpdateNovel) -> Result<Novel, DbError>;
    async fn delete(&self, id: Uuid) -> Result<(), DbError>;
}

pub struct SqliteNovelRepository {
    pool: SqlitePool,
}

impl SqliteNovelRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    /// 根据 novel_id 列表查询关联标签（两步查询：关联表 + 标签表）
    async fn find_tags_by_novel_ids(
        &self,
        novel_ids: &[Uuid],
    ) -> Result<std::collections::HashMap<Uuid, Vec<Tag>>, DbError> {
        if novel_ids.is_empty() {
            return Ok(std::collections::HashMap::new());
        }

        // 第一步：从 novel_tags 查询关联关系
        let placeholders = novel_ids.iter().map(|_| "?").collect::<Vec<_>>().join(", ");
        let assoc_query = format!(
            "SELECT novel_id, tag_id FROM novel_tags WHERE novel_id IN ({})",
            placeholders
        );

        let mut db_query = sqlx::query_as::<_, NovelTagRow>(&assoc_query);
        for id in novel_ids {
            db_query = db_query.bind(id);
        }
        let assoc_rows = db_query.fetch_all(&self.pool).await?;

        if assoc_rows.is_empty() {
            return Ok(std::collections::HashMap::new());
        }

        // 第二步：根据 tag_id 列表查询标签详情
        let tag_ids: Vec<i64> = assoc_rows.iter().map(|r| r.tag_id).collect();
        let tag_placeholders = tag_ids.iter().map(|_| "?").collect::<Vec<_>>().join(", ");
        let tag_query = format!("SELECT * FROM tags WHERE id IN ({})", tag_placeholders);

        let mut tag_db_query = sqlx::query_as::<_, Tag>(&tag_query);
        for id in &tag_ids {
            tag_db_query = tag_db_query.bind(id);
        }
        let tags = tag_db_query.fetch_all(&self.pool).await?;

        // 构建 tag_id -> Tag 映射
        let tag_map: std::collections::HashMap<i64, Tag> =
            tags.into_iter().map(|t| (t.id, t)).collect();

        // 按 novel_id 分组标签
        let mut result: std::collections::HashMap<Uuid, Vec<Tag>> =
            std::collections::HashMap::new();
        for row in assoc_rows {
            if let Some(tag) = tag_map.get(&row.tag_id).cloned() {
                result.entry(row.novel_id).or_default().push(tag);
            }
        }

        Ok(result)
    }

    /// 为小说列表附加标签信息
    async fn attach_tags(&self, novels: Vec<Novel>) -> Result<Vec<NovelWithTags>, DbError> {
        let ids: Vec<Uuid> = novels.iter().map(|n| n.id).collect();
        let tag_map = self.find_tags_by_novel_ids(&ids).await?;

        let result = novels
            .into_iter()
            .map(|novel| {
                let tags = tag_map.get(&novel.id).cloned().unwrap_or_default();
                NovelWithTags::with_tags(novel, tags)
            })
            .collect();

        Ok(result)
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
    /// 当 with_tags 为 true 时，额外查询并附加标签信息。
    async fn find_all(&self, options: &QueryOptions) -> Result<Vec<NovelWithTags>, DbError> {
        let novels = sqlx::query_as::<_, Novel>("SELECT * FROM novels ORDER BY created_at DESC")
            .fetch_all(&self.pool)
            .await?;

        if options.with_tags {
            return self.attach_tags(novels).await;
        }

        Ok(novels.into_iter().map(NovelWithTags::from_novel).collect())
    }

    /// 分页查询小说列表
    ///
    /// 使用通用分页工具函数，按创建时间倒序排列。
    /// 当 with_tags 为 true 时，额外查询并附加标签信息。
    async fn find_with_pagination(
        &self,
        params: &PaginationParams,
        options: &QueryOptions,
    ) -> Result<PaginatedResult<NovelWithTags>, DbError> {
        let result = query_with_pagination::<Novel>(
            &self.pool,
            params,
            "SELECT COUNT(*) FROM novels",
            "SELECT * FROM novels ORDER BY created_at DESC LIMIT ? OFFSET ?",
        )
        .await?;

        if options.with_tags {
            let with_tags = self.attach_tags(result.data).await?;
            return Ok(PaginatedResult {
                data: with_tags,
                total: result.total,
            });
        }

        Ok(PaginatedResult {
            data: result
                .data
                .into_iter()
                .map(NovelWithTags::from_novel)
                .collect(),
            total: result.total,
        })
    }

    /// 根据 ID 获取小说信息
    ///
    /// 当 with_tags 为 true 时，额外查询并附加标签信息。
    async fn find_by_id(&self, id: Uuid, options: &QueryOptions) -> Result<NovelWithTags, DbError> {
        let novel = sqlx::query_as::<_, Novel>("SELECT * FROM novels WHERE id = ?1")
            .bind(id)
            .fetch_one(&self.pool)
            .await?;

        if options.with_tags {
            let tag_map = self.find_tags_by_novel_ids(&[id]).await?;
            let tags = tag_map.get(&id).cloned().unwrap_or_default();
            return Ok(NovelWithTags::with_tags(novel, tags));
        }

        Ok(NovelWithTags::from_novel(novel))
    }

    /// 更新小说信息及标签关联
    ///
    /// 使用数据库事务确保更新操作原子性：
    /// 1. 更新小说主表字段
    /// 2. 删除旧标签关联
    /// 3. 插入新标签关联
    async fn update(&self, id: Uuid, novel: &UpdateNovel) -> Result<Novel, DbError> {
        let mut tx = self.pool.begin().await?;
        let now = Utc::now();

        let updated_novel = sqlx::query_as::<_, Novel>(
            "UPDATE novels SET title = ?1, target_reader = ?2, description = ?3, updated_at = ?4
             WHERE id = ?5
             RETURNING *",
        )
        .bind(&novel.title)
        .bind(&novel.target_reader)
        .bind(&novel.description)
        .bind(now)
        .bind(id)
        .fetch_one(tx.as_mut())
        .await?;

        sqlx::query("DELETE FROM novel_tags WHERE novel_id = ?1")
            .bind(id)
            .execute(tx.as_mut())
            .await?;

        for tag_id in &novel.tag_ids {
            sqlx::query("INSERT OR IGNORE INTO novel_tags (novel_id, tag_id) VALUES (?1, ?2)")
                .bind(id)
                .bind(tag_id)
                .execute(tx.as_mut())
                .await?;
        }

        tx.commit().await?;
        tracing::info!(
            "小说更新成功: {} - {}",
            updated_novel.id,
            updated_novel.title
        );
        Ok(updated_novel)
    }

    /// 删除小说及其所有关联数据
    ///
    /// 使用事务确保级联删除的原子性，按以下顺序删除：
    /// 1. 章节历史版本 (chapter_versions)
    /// 2. 章节与分卷关联 (volume_chapters)
    /// 3. 章节 (chapters)
    /// 4. 分卷 (volumes)
    /// 5. 标签关联 (novel_tags)
    /// 6. 角色 (characters)
    /// 7. 小说本身 (novels)
    async fn delete(&self, id: Uuid) -> Result<(), DbError> {
        let mut tx = self.pool.begin().await?;

        // 1. 查询该小说下的所有章节ID
        let chapter_ids: Vec<Uuid> =
            sqlx::query_scalar("SELECT id FROM chapters WHERE novel_id = ?1")
                .bind(id)
                .fetch_all(tx.as_mut())
                .await?
                .into_iter()
                .collect();

        // 2. 删除所有章节的历史版本 (chapter_versions)
        if !chapter_ids.is_empty() {
            let placeholders = chapter_ids
                .iter()
                .map(|_| "?")
                .collect::<Vec<_>>()
                .join(", ");
            let query = format!(
                "DELETE FROM chapter_versions WHERE chapter_id IN ({})",
                placeholders
            );
            let mut db_query = sqlx::query(&query);
            for chapter_id in &chapter_ids {
                db_query = db_query.bind(chapter_id);
            }
            db_query.execute(tx.as_mut()).await?;
        }

        // 3. 删除所有章节与分卷的关联 (volume_chapters)
        if !chapter_ids.is_empty() {
            let placeholders = chapter_ids
                .iter()
                .map(|_| "?")
                .collect::<Vec<_>>()
                .join(", ");
            let query = format!(
                "DELETE FROM volume_chapters WHERE chapter_id IN ({})",
                placeholders
            );
            let mut db_query = sqlx::query(&query);
            for chapter_id in &chapter_ids {
                db_query = db_query.bind(chapter_id);
            }
            db_query.execute(tx.as_mut()).await?;
        }

        // 4. 删除所有章节 (chapters)
        sqlx::query("DELETE FROM chapters WHERE novel_id = ?1")
            .bind(id)
            .execute(tx.as_mut())
            .await?;

        // 5. 删除该小说下的所有分卷 (volumes)
        sqlx::query("DELETE FROM volumes WHERE novel_id = ?1")
            .bind(id)
            .execute(tx.as_mut())
            .await?;

        // 6. 删除所有标签关联 (novel_tags)
        sqlx::query("DELETE FROM novel_tags WHERE novel_id = ?1")
            .bind(id)
            .execute(tx.as_mut())
            .await?;

        // 7. 删除该小说下的所有角色 (characters)
        sqlx::query("DELETE FROM characters WHERE novel_id = ?1")
            .bind(id)
            .execute(tx.as_mut())
            .await?;

        // 8. 最后删除小说本身 (novels)
        sqlx::query("DELETE FROM novels WHERE id = ?1")
            .bind(id)
            .execute(tx.as_mut())
            .await?;

        tx.commit().await?;
        tracing::info!("小说及其关联数据删除成功: {}", id);
        Ok(())
    }
}
