use async_trait::async_trait;
use chrono::Utc;
use sqlx::SqlitePool;
use uuid::Uuid;

use crate::database::error::DbError;
use crate::database::models::novel::{NewNovel, Novel, NovelStats, NovelWithTags, UpdateNovel};
use crate::database::models::tag::Tag;
use crate::utils::pagination::{query_with_pagination, PaginatedResult, PaginationParams};

/// 查询选项，控制是否解析关联的标签
#[derive(Debug, Clone, Default)]
pub struct QueryOptions {
    /// 是否加载关联的标签信息
    pub with_tags: bool,
    /// 是否加载统计信息（章节数、字数、最近更新等）
    pub with_stats: bool,
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
    async fn search_with_pagination(
        &self,
        keyword: Option<&str>,
        params: &PaginationParams,
        options: &QueryOptions,
    ) -> Result<PaginatedResult<NovelWithTags>, DbError>;
    async fn find_by_id(&self, id: Uuid, options: &QueryOptions) -> Result<NovelWithTags, DbError>;
    async fn update(&self, id: Uuid, novel: &UpdateNovel) -> Result<Novel, DbError>;
    async fn delete(&self, id: Uuid) -> Result<(), DbError>;
    /// 批量获取小说统计信息；返回 novel_id -> NovelStats 的映射
    async fn get_novel_stats(
        &self,
        novel_ids: &[Uuid],
    ) -> Result<std::collections::HashMap<Uuid, NovelStats>, DbError>;
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

    /// 为 NovelWithTags 列表附加统计信息
    async fn attach_stats(
        &self,
        novels: Vec<NovelWithTags>,
    ) -> Result<Vec<NovelWithTags>, DbError> {
        let ids: Vec<Uuid> = novels.iter().map(|n| n.novel.id).collect();
        let stats_map = self.get_novel_stats(&ids).await?;
        let result = novels
            .into_iter()
            .map(|n| {
                let stats = stats_map.get(&n.novel.id).cloned();
                n.with_stats(stats)
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

        let with_tags = if options.with_tags {
            self.attach_tags(novels).await?
        } else {
            novels.into_iter().map(NovelWithTags::from_novel).collect()
        };

        if options.with_stats {
            return self.attach_stats(with_tags).await;
        }
        Ok(with_tags)
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

        let total = result.total;
        let with_tags = if options.with_tags {
            self.attach_tags(result.data).await?
        } else {
            result
                .data
                .into_iter()
                .map(NovelWithTags::from_novel)
                .collect()
        };

        let data = if options.with_stats {
            self.attach_stats(with_tags).await?
        } else {
            with_tags
        };

        Ok(PaginatedResult { data, total })
    }

    /// 按书名模糊搜索并分页查询
    async fn search_with_pagination(
        &self,
        keyword: Option<&str>,
        params: &PaginationParams,
        options: &QueryOptions,
    ) -> Result<PaginatedResult<NovelWithTags>, DbError> {
        let (count_sql, data_sql) = match keyword {
            Some(_) => (
                "SELECT COUNT(*) FROM novels WHERE title LIKE ?",
                "SELECT * FROM novels WHERE title LIKE ? ORDER BY created_at DESC LIMIT ? OFFSET ?",
            ),
            None => (
                "SELECT COUNT(*) FROM novels",
                "SELECT * FROM novels ORDER BY created_at DESC LIMIT ? OFFSET ?",
            ),
        };

        // 处理全量查询 (limit=0)
        let effective_limit = if params.page_size == 0 {
            i64::MAX
        } else {
            params.page_size
        };
        let offset = (params.page - 1) * params.page_size;

        // 1. 查询总数
        let total: (i64,) = if let Some(kw) = keyword {
            sqlx::query_as(count_sql)
                .bind(format!("%{}%", kw))
                .fetch_one(&self.pool)
                .await?
        } else {
            sqlx::query_as(count_sql).fetch_one(&self.pool).await?
        };

        // 2. 查询分页数据
        let novels = if let Some(kw) = keyword {
            sqlx::query_as::<_, Novel>(data_sql)
                .bind(format!("%{}%", kw))
                .bind(effective_limit)
                .bind(offset)
                .fetch_all(&self.pool)
                .await?
        } else {
            sqlx::query_as::<_, Novel>(data_sql)
                .bind(effective_limit)
                .bind(offset)
                .fetch_all(&self.pool)
                .await?
        };

        let with_tags = if options.with_tags {
            self.attach_tags(novels).await?
        } else {
            novels.into_iter().map(NovelWithTags::from_novel).collect()
        };

        let data = if options.with_stats {
            self.attach_stats(with_tags).await?
        } else {
            with_tags
        };

        Ok(PaginatedResult {
            data,
            total: total.0,
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

        let with_tags = if options.with_tags {
            let tag_map = self.find_tags_by_novel_ids(&[id]).await?;
            let tags = tag_map.get(&id).cloned().unwrap_or_default();
            NovelWithTags::with_tags(novel, tags)
        } else {
            NovelWithTags::from_novel(novel)
        };

        if options.with_stats {
            let stats_map = self.get_novel_stats(&[id]).await?;
            return Ok(with_tags.with_stats(stats_map.get(&id).cloned()));
        }
        Ok(with_tags)
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
    /// 7. 章节大纲 (chapter_outlines)
    /// 8. 小说本身 (novels)
    async fn delete(&self, id: Uuid) -> Result<(), DbError> {
        let mut tx = self.pool.begin().await?;

        // 0. 查询该小说下的所有章节ID
        let chapter_ids: Vec<Uuid> =
            sqlx::query_scalar("SELECT id FROM chapters WHERE novel_id = ?1")
                .bind(id)
                .fetch_all(tx.as_mut())
                .await?
                .into_iter()
                .collect();

        // 1. 删除所有章节的历史版本 (chapter_versions)
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

        // 2. 删除所有章节与分卷的关联 (volume_chapters)
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

        // 3. 删除所有章节 (chapters)
        sqlx::query("DELETE FROM chapters WHERE novel_id = ?1")
            .bind(id)
            .execute(tx.as_mut())
            .await?;

        // 4. 删除该小说下的所有分卷 (volumes)
        sqlx::query("DELETE FROM volumes WHERE novel_id = ?1")
            .bind(id)
            .execute(tx.as_mut())
            .await?;

        // 5. 删除所有标签关联 (novel_tags)
        sqlx::query("DELETE FROM novel_tags WHERE novel_id = ?1")
            .bind(id)
            .execute(tx.as_mut())
            .await?;

        // 6. 删除该小说下的所有角色 (characters)
        sqlx::query("DELETE FROM characters WHERE novel_id = ?1")
            .bind(id)
            .execute(tx.as_mut())
            .await?;

        // 7. 删除该小说下的所有章节大纲 (chapter_outlines)
        sqlx::query("DELETE FROM chapter_outlines WHERE novel_id = ?1")
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

    /// 批量查询小说统计信息
    ///
    /// 只统计非草稿章节（sequence >= 0）。分三步：
    /// 1. 聚合查询：总章节数、总字数、最近更新时间；
    /// 2. 查询每本小说最近更新章节的标题、序号、所属分卷序号；
    /// 3. 合并为 NovelStats Map。
    async fn get_novel_stats(
        &self,
        novel_ids: &[Uuid],
    ) -> Result<std::collections::HashMap<Uuid, NovelStats>, DbError> {
        if novel_ids.is_empty() {
            return Ok(std::collections::HashMap::new());
        }

        let aggregates = fetch_stats_aggregate(&self.pool, novel_ids).await?;
        let latest_map = fetch_latest_chapter_info(&self.pool, novel_ids).await?;

        Ok(build_stats_map(novel_ids, aggregates, latest_map))
    }
}

/// 聚合统计行（每本小说一条）
#[derive(Debug, sqlx::FromRow)]
struct StatsAggregateRow {
    novel_id: Uuid,
    chapter_count: i64,
    total_word_count: i64,
    last_updated_at: Option<chrono::DateTime<Utc>>,
}

/// 最近更新章节信息行
#[derive(Debug, sqlx::FromRow)]
struct LatestChapterRow {
    novel_id: Uuid,
    chapter_title: String,
    chapter_sequence: i64,
    volume_sequence: Option<i64>,
}

/// 查询每本小说非草稿章节的聚合信息（章节数、总字数、最近更新时间）
async fn fetch_stats_aggregate(
    pool: &SqlitePool,
    novel_ids: &[Uuid],
) -> Result<std::collections::HashMap<Uuid, StatsAggregateRow>, DbError> {
    let placeholders = novel_ids.iter().map(|_| "?").collect::<Vec<_>>().join(", ");
    let sql = format!(
        "SELECT novel_id,
                COUNT(*) AS chapter_count,
                COALESCE(SUM(word_count), 0) AS total_word_count,
                MAX(updated_at) AS last_updated_at
         FROM chapters
         WHERE sequence >= 0 AND novel_id IN ({})
         GROUP BY novel_id",
        placeholders
    );

    let mut query = sqlx::query_as::<_, StatsAggregateRow>(&sql);
    for id in novel_ids {
        query = query.bind(id);
    }
    let rows = query.fetch_all(pool).await?;

    Ok(rows.into_iter().map(|r| (r.novel_id, r)).collect())
}

/// 查询每本小说"最近更新章节"的标题、序号及所属分卷序号
///
/// 通过窗口函数选出 updated_at 最大的非草稿章节；
/// 若章节未关联分卷，则回退取该小说的首卷 sequence。
async fn fetch_latest_chapter_info(
    pool: &SqlitePool,
    novel_ids: &[Uuid],
) -> Result<std::collections::HashMap<Uuid, LatestChapterRow>, DbError> {
    let placeholders = novel_ids.iter().map(|_| "?").collect::<Vec<_>>().join(", ");
    // 使用子查询按 novel_id + updated_at 取最新章节，再关联分卷获取 sequence；
    // 章节无分卷关联时，尝试取该小说 sequence 最小的分卷（即首卷）的 sequence。
    let sql = format!(
        "SELECT c.novel_id AS novel_id,
                c.title AS chapter_title,
                c.sequence AS chapter_sequence,
                COALESCE(v.sequence, (
                    SELECT MIN(v2.sequence) FROM volumes v2 WHERE v2.novel_id = c.novel_id
                )) AS volume_sequence
         FROM chapters c
         LEFT JOIN volume_chapters vc ON vc.chapter_id = c.id
         LEFT JOIN volumes v ON v.id = vc.volume_id
         WHERE c.sequence >= 0 AND c.novel_id IN ({})
           AND c.updated_at = (
               SELECT MAX(c2.updated_at) FROM chapters c2
               WHERE c2.novel_id = c.novel_id AND c2.sequence >= 0
           )",
        placeholders
    );

    let mut query = sqlx::query_as::<_, LatestChapterRow>(&sql);
    for id in novel_ids {
        query = query.bind(id);
    }
    let rows = query.fetch_all(pool).await?;

    // 同一 updated_at 时可能多行，按 novel_id 去重（保留第一条）
    let mut map: std::collections::HashMap<Uuid, LatestChapterRow> =
        std::collections::HashMap::new();
    for row in rows {
        map.entry(row.novel_id).or_insert(row);
    }
    Ok(map)
}

/// 根据聚合数据与最新章节数据合并生成 NovelStats Map
fn build_stats_map(
    novel_ids: &[Uuid],
    aggregates: std::collections::HashMap<Uuid, StatsAggregateRow>,
    latest_map: std::collections::HashMap<Uuid, LatestChapterRow>,
) -> std::collections::HashMap<Uuid, NovelStats> {
    novel_ids
        .iter()
        .map(|id| {
            let stats = match aggregates.get(id) {
                Some(agg) => build_stats_with_aggregate(*id, agg, latest_map.get(id)),
                None => empty_stats(*id),
            };
            (*id, stats)
        })
        .collect()
}

/// 根据聚合行与最新章节行构造 NovelStats
fn build_stats_with_aggregate(
    novel_id: Uuid,
    agg: &StatsAggregateRow,
    latest: Option<&LatestChapterRow>,
) -> NovelStats {
    NovelStats {
        novel_id,
        chapter_count: agg.chapter_count,
        total_word_count: agg.total_word_count,
        last_updated_at: agg.last_updated_at,
        last_updated_volume_sequence: latest.and_then(|r| r.volume_sequence),
        last_updated_chapter_sequence: latest.map(|r| r.chapter_sequence),
        last_updated_chapter_title: latest.map(|r| r.chapter_title.clone()),
    }
}

/// 无章节场景的空统计
fn empty_stats(novel_id: Uuid) -> NovelStats {
    NovelStats {
        novel_id,
        chapter_count: 0,
        total_word_count: 0,
        last_updated_at: None,
        last_updated_volume_sequence: None,
        last_updated_chapter_sequence: None,
        last_updated_chapter_title: None,
    }
}
