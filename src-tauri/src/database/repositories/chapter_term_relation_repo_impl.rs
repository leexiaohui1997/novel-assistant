use async_trait::async_trait;
use chrono::Utc;
use sqlx::{QueryBuilder, Sqlite, SqlitePool};
use uuid::Uuid;

use crate::database::error::DbError;
use crate::database::models::chapter_term_relation::{
    ChapterTermRelation, ChapterTermRelationQuery, ChapterTermRelationWithTerm,
    ChapterTermRelationWithTermRow, NewChapterTermRelation, UpdateChapterTermRelation,
};
use crate::utils::pagination::PaginatedResult;

/// SQLite 名词-章节关联仓储实现
pub struct SqliteChapterTermRelationRepository {
    pool: SqlitePool,
}

impl SqliteChapterTermRelationRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl super::chapter_term_relation_repo::ChapterTermRelationRepository
    for SqliteChapterTermRelationRepository
{
    async fn create(
        &self,
        relation: &NewChapterTermRelation,
    ) -> Result<ChapterTermRelation, DbError> {
        let now = Utc::now();
        let id = Uuid::new_v4();

        sqlx::query(
            "INSERT INTO chapter_term_relations (id, novel_id, chapter_id, term_id, description, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        )
        .bind(id)
        .bind(relation.novel_id)
        .bind(relation.chapter_id)
        .bind(relation.term_id)
        .bind(&relation.description)
        .bind(now)
        .bind(now)
        .execute(&self.pool)
        .await?;

        self.find_by_id(&id)
            .await?
            .ok_or_else(|| DbError::Business(format!("关联 {} 创建后未找到", id)))
    }

    async fn find_by_id(&self, id: &Uuid) -> Result<Option<ChapterTermRelation>, DbError> {
        let relation = sqlx::query_as::<_, ChapterTermRelation>(
            "SELECT * FROM chapter_term_relations WHERE id = ?1",
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;
        Ok(relation)
    }

    async fn update(
        &self,
        id: &Uuid,
        update: &UpdateChapterTermRelation,
    ) -> Result<ChapterTermRelation, DbError> {
        let now = Utc::now();

        // 动态构建 UPDATE SQL
        let mut query_parts = vec!["updated_at = ?1".to_string()];
        let mut param_index = 2;

        if update.chapter_id.is_some() {
            query_parts.push(format!("chapter_id = ?{}", param_index));
            param_index += 1;
        }

        if update.description.is_some() {
            query_parts.push(format!("description = ?{}", param_index));
            param_index += 1;
        }

        let set_clause = query_parts.join(", ");
        let sql = format!(
            "UPDATE chapter_term_relations SET {} WHERE id = ?{}",
            set_clause, param_index
        );

        let mut query = sqlx::query(&sql);
        query = query.bind(now);

        if let Some(chapter_id) = update.chapter_id {
            query = query.bind(chapter_id);
        }

        if let Some(description) = &update.description {
            query = query.bind(description);
        }

        query = query.bind(id);

        let result = query.execute(&self.pool).await?;

        if result.rows_affected() == 0 {
            return Err(DbError::Business(format!("关联 {} 不存在，无法更新", id)));
        }

        self.find_by_id(id)
            .await?
            .ok_or_else(|| DbError::Business(format!("关联 {} 更新后未找到", id)))
    }

    async fn delete(&self, id: &Uuid) -> Result<(), DbError> {
        let result = sqlx::query("DELETE FROM chapter_term_relations WHERE id = ?1")
            .bind(id)
            .execute(&self.pool)
            .await?;

        if result.rows_affected() == 0 {
            return Err(DbError::Business(format!("关联 {} 不存在，无法删除", id)));
        }
        Ok(())
    }

    async fn find_by_chapter_and_term(
        &self,
        chapter_id: &Uuid,
        term_id: &Uuid,
    ) -> Result<Option<ChapterTermRelation>, DbError> {
        let relation = sqlx::query_as::<_, ChapterTermRelation>(
            "SELECT * FROM chapter_term_relations WHERE chapter_id = ?1 AND term_id = ?2",
        )
        .bind(chapter_id)
        .bind(term_id)
        .fetch_optional(&self.pool)
        .await?;
        Ok(relation)
    }

    async fn find_with_query(
        &self,
        query: &ChapterTermRelationQuery,
    ) -> Result<PaginatedResult<ChapterTermRelation>, DbError> {
        let total = self.count_with_filter(query).await?;
        let data = self.list_with_filter(query).await?;

        let total = if query.page_size == 0 {
            data.len() as i64
        } else {
            total
        };
        Ok(PaginatedResult { data, total })
    }

    async fn delete_by_chapter_id(&self, chapter_id: &Uuid) -> Result<(), DbError> {
        sqlx::query("DELETE FROM chapter_term_relations WHERE chapter_id = ?1")
            .bind(chapter_id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    async fn delete_by_term_id(&self, term_id: &Uuid) -> Result<(), DbError> {
        sqlx::query("DELETE FROM chapter_term_relations WHERE term_id = ?1")
            .bind(term_id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    async fn sync_chapter_id(
        &self,
        novel_id: &Uuid,
        new_chapter_id: &Uuid,
    ) -> Result<u64, DbError> {
        let now = Utc::now();

        // 更新指定小说下所有 chapter_id 为空的记录
        let result = sqlx::query(
            "UPDATE chapter_term_relations 
             SET chapter_id = ?1, updated_at = ?2 
             WHERE novel_id = ?3 AND chapter_id IS NULL",
        )
        .bind(new_chapter_id)
        .bind(now)
        .bind(novel_id)
        .execute(&self.pool)
        .await?;

        Ok(result.rows_affected())
    }

    async fn find_terms_by_novel_and_chapter(
        &self,
        novel_id: &Uuid,
        chapter_id: Option<&Uuid>,
    ) -> Result<Vec<ChapterTermRelationWithTerm>, DbError> {
        let mut sql = String::from(
            "SELECT 
                ctr.id as relation_id,
                ctr.chapter_id,
                ctr.description,
                ctr.created_at,
                ctr.updated_at,
                nt.id as term_id,
                nt.novel_id as term_novel_id,
                nt.term_type,
                nt.name as term_name,
                nt.description as term_description,
                nt.created_at as term_created_at,
                nt.updated_at as term_updated_at
             FROM chapter_term_relations ctr
             INNER JOIN novel_terms nt ON ctr.term_id = nt.id
             WHERE ctr.novel_id = ?1",
        );

        if chapter_id.is_some() {
            sql.push_str(" AND ctr.chapter_id = ?2");
        } else {
            sql.push_str(" AND ctr.chapter_id IS NULL");
        }

        sql.push_str(" ORDER BY ctr.created_at ASC");

        let mut query = sqlx::query_as::<_, ChapterTermRelationWithTermRow>(&sql);
        query = query.bind(novel_id);

        if let Some(cid) = chapter_id {
            query = query.bind(cid);
        }

        let rows = query.fetch_all(&self.pool).await?;
        Ok(rows.into_iter().map(Into::into).collect())
    }

    async fn delete_by_novel_and_chapter(
        &self,
        novel_id: &Uuid,
        chapter_id: Option<&Uuid>,
    ) -> Result<u64, DbError> {
        let sql = if chapter_id.is_some() {
            "DELETE FROM chapter_term_relations WHERE novel_id = ?1 AND chapter_id = ?2"
        } else {
            "DELETE FROM chapter_term_relations WHERE novel_id = ?1 AND chapter_id IS NULL"
        };

        let mut query = sqlx::query(sql);
        query = query.bind(novel_id);

        if let Some(cid) = chapter_id {
            query = query.bind(cid);
        }

        let result = query.execute(&self.pool).await?;
        Ok(result.rows_affected())
    }
}

impl SqliteChapterTermRelationRepository {
    /// 计算符合条件的总数
    async fn count_with_filter(&self, query: &ChapterTermRelationQuery) -> Result<i64, DbError> {
        let mut builder: QueryBuilder<Sqlite> =
            QueryBuilder::new("SELECT COUNT(*) FROM chapter_term_relations");
        self.push_where_clause(&mut builder, query);

        let total: (i64,) = builder.build_query_as().fetch_one(&self.pool).await?;
        Ok(total.0)
    }

    /// 查询列表（支持筛选、排序、分页）
    async fn list_with_filter(
        &self,
        query: &ChapterTermRelationQuery,
    ) -> Result<Vec<ChapterTermRelation>, DbError> {
        let mut builder: QueryBuilder<Sqlite> =
            QueryBuilder::new("SELECT * FROM chapter_term_relations");
        self.push_where_clause(&mut builder, query);

        // 添加排序
        let sort_by = query.sort_by.as_deref().unwrap_or("created_at");
        let sort_order = query.sort_order.as_deref().unwrap_or("asc");

        // 验证排序字段，防止 SQL 注入
        let valid_sort_fields = ["created_at", "updated_at"];
        let sort_field = if valid_sort_fields.contains(&sort_by) {
            sort_by
        } else {
            "created_at"
        };

        let order = if sort_order.to_lowercase() == "asc" {
            "ASC"
        } else {
            "DESC"
        };

        builder.push(" ORDER BY ");
        builder.push(sort_field);
        builder.push(" ");
        builder.push(order);

        // 添加分页
        if query.page_size > 0 {
            let page = if query.page < 1 { 1 } else { query.page };
            let offset = (page - 1) * query.page_size;
            builder.push(" LIMIT ");
            builder.push_bind(query.page_size);
            builder.push(" OFFSET ");
            builder.push_bind(offset);
        }

        let data = builder
            .build_query_as::<ChapterTermRelation>()
            .fetch_all(&self.pool)
            .await?;
        Ok(data)
    }

    /// 构建 WHERE 子句
    fn push_where_clause<'a>(
        &self,
        builder: &mut QueryBuilder<'a, Sqlite>,
        query: &'a ChapterTermRelationQuery,
    ) {
        let mut needs_and = false;

        if let Some(chapter_id) = query.chapter_id {
            if needs_and {
                builder.push(" AND");
            }
            builder.push(" chapter_id = ");
            builder.push_bind(chapter_id);
            needs_and = true;
        }

        if let Some(term_id) = query.term_id {
            if needs_and {
                builder.push(" AND");
            }
            builder.push(" term_id = ");
            builder.push_bind(term_id);
        }
    }
}
