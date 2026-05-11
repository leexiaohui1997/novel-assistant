use async_trait::async_trait;
use chrono::Utc;
use sqlx::{QueryBuilder, Sqlite, SqlitePool};
use uuid::Uuid;

use crate::database::error::DbError;
use crate::database::models::novel_term::{NovelTerm, TermType};
use crate::utils::pagination::PaginatedResult;

/// 小说名词组合查询入参
#[derive(Debug, Clone, Default)]
pub struct NovelTermQuery {
    /// 必填：所属小说 ID
    pub novel_id: Uuid,
    /// 可选：精确匹配名词类型
    pub term_type: Option<TermType>,
    /// 可选：精确匹配名词名称（trim 后非空才生效）
    pub name: Option<String>,
    /// 可选：模糊匹配名词描述（trim 后非空才生效）
    pub description_keyword: Option<String>,
    /// 页码（从 1 开始；< 1 时兜底为 1）
    pub page: i64,
    /// 每页数量；为 0 时表示查询全部
    pub page_size: i64,
}

/// 小说名词仓储 trait
#[async_trait]
pub trait NovelTermRepository {
    /// 创建名词
    async fn create(&self, term: &NovelTerm) -> Result<NovelTerm, DbError>;

    /// 根据 ID 查询名词
    async fn find_by_id(&self, id: &Uuid) -> Result<Option<NovelTerm>, DbError>;

    /// 更新名词（仅更新 term_type / name / description / updated_at）
    async fn update(&self, term: &NovelTerm) -> Result<NovelTerm, DbError>;

    /// 删除名词
    async fn delete(&self, id: &Uuid) -> Result<(), DbError>;

    /// 组合查询（按类型/名称精确 + 描述模糊 + 分页 + 排序）
    async fn find_with_query(
        &self,
        query: &NovelTermQuery,
    ) -> Result<PaginatedResult<NovelTerm>, DbError>;
}

/// SQLite 名词仓储实现
pub struct SqliteNovelTermRepository {
    pool: SqlitePool,
}

impl SqliteNovelTermRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }
}

/// 经过归一化（trim + 转义）的过滤条件，用于驱动 SQL 拼装与参数绑定
struct NormalizedFilter {
    novel_id: Uuid,
    term_type: Option<TermType>,
    name: Option<String>,
    description_like: Option<String>,
}

/// 对 SQL LIKE 元字符（`%`、`_`、`\`）做转义，配合 `ESCAPE '\\'` 子句使用
fn escape_like_keyword(raw: &str) -> String {
    let mut buf = String::with_capacity(raw.len() + raw.len() / 4);
    for ch in raw.chars() {
        if ch == '\\' || ch == '%' || ch == '_' {
            buf.push('\\');
        }
        buf.push(ch);
    }
    buf
}

/// 将 trim 后非空的可选字符串包装为 `Some`，否则返回 `None`
fn non_empty_trimmed(value: &Option<String>) -> Option<String> {
    value
        .as_ref()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
}

/// 由原始查询入参构造归一化过滤条件
fn normalize_filter(query: &NovelTermQuery) -> NormalizedFilter {
    let description_like = non_empty_trimmed(&query.description_keyword)
        .map(|kw| format!("%{}%", escape_like_keyword(&kw)));
    NormalizedFilter {
        novel_id: query.novel_id,
        term_type: query.term_type,
        name: non_empty_trimmed(&query.name),
        description_like,
    }
}

/// 在已有 `QueryBuilder` 上追加 WHERE 与所有过滤条件（必含 novel_id 等值匹配）
fn push_where_clause<'a>(builder: &mut QueryBuilder<'a, Sqlite>, filter: &'a NormalizedFilter) {
    builder.push(" WHERE novel_id = ");
    builder.push_bind(filter.novel_id);

    if let Some(term_type) = filter.term_type {
        builder.push(" AND term_type = ");
        builder.push_bind(term_type);
    }
    if let Some(name) = &filter.name {
        builder.push(" AND name = ");
        builder.push_bind(name.clone());
    }
    if let Some(like) = &filter.description_like {
        builder.push(" AND description LIKE ");
        builder.push_bind(like.clone());
        builder.push(" ESCAPE '\\'");
    }
}

/// 计算实际生效的页码（最小为 1）
fn effective_page(page: i64) -> i64 {
    if page < 1 {
        1
    } else {
        page
    }
}

#[async_trait]
impl NovelTermRepository for SqliteNovelTermRepository {
    async fn create(&self, term: &NovelTerm) -> Result<NovelTerm, DbError> {
        let now = Utc::now();
        let id = Uuid::new_v4();

        sqlx::query(
            "INSERT INTO novel_terms (id, novel_id, term_type, name, description, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        )
        .bind(id)
        .bind(term.novel_id)
        .bind(term.term_type)
        .bind(&term.name)
        .bind(&term.description)
        .bind(now)
        .bind(now)
        .execute(&self.pool)
        .await?;

        self.find_by_id(&id)
            .await?
            .ok_or_else(|| DbError::Business(format!("名词 {} 创建后未找到", id)))
    }

    async fn find_by_id(&self, id: &Uuid) -> Result<Option<NovelTerm>, DbError> {
        let term = sqlx::query_as::<_, NovelTerm>("SELECT * FROM novel_terms WHERE id = ?1")
            .bind(id)
            .fetch_optional(&self.pool)
            .await?;
        Ok(term)
    }

    async fn update(&self, term: &NovelTerm) -> Result<NovelTerm, DbError> {
        let now = Utc::now();

        let result = sqlx::query(
            "UPDATE novel_terms
             SET term_type = ?2, name = ?3, description = ?4, updated_at = ?5
             WHERE id = ?1",
        )
        .bind(term.id)
        .bind(term.term_type)
        .bind(&term.name)
        .bind(&term.description)
        .bind(now)
        .execute(&self.pool)
        .await?;

        if result.rows_affected() == 0 {
            return Err(DbError::Business(format!(
                "名词 {} 不存在，无法更新",
                term.id
            )));
        }

        self.find_by_id(&term.id)
            .await?
            .ok_or_else(|| DbError::Business(format!("名词 {} 更新后未找到", term.id)))
    }

    async fn delete(&self, id: &Uuid) -> Result<(), DbError> {
        let result = sqlx::query("DELETE FROM novel_terms WHERE id = ?1")
            .bind(id)
            .execute(&self.pool)
            .await?;

        if result.rows_affected() == 0 {
            return Err(DbError::Business(format!("名词 {} 不存在，无法删除", id)));
        }
        Ok(())
    }

    async fn find_with_query(
        &self,
        query: &NovelTermQuery,
    ) -> Result<PaginatedResult<NovelTerm>, DbError> {
        if query.novel_id.is_nil() {
            return Err(DbError::Business("novel_id 不能为空".to_string()));
        }

        let filter = normalize_filter(query);
        let total = self.count_with_filter(&filter).await?;
        let data = self
            .list_with_filter(&filter, query.page, query.page_size)
            .await?;

        let total = if query.page_size == 0 {
            data.len() as i64
        } else {
            total
        };
        Ok(PaginatedResult { data, total })
    }
}

impl SqliteNovelTermRepository {
    /// 仅查询命中总数，与列表共享同一 WHERE 子句
    async fn count_with_filter(&self, filter: &NormalizedFilter) -> Result<i64, DbError> {
        let mut builder: QueryBuilder<Sqlite> =
            QueryBuilder::new("SELECT COUNT(*) FROM novel_terms");
        push_where_clause(&mut builder, filter);

        let total: (i64,) = builder.build_query_as().fetch_one(&self.pool).await?;
        Ok(total.0)
    }

    /// 查询列表（按 created_at ASC, id ASC 排序，按需分页）
    async fn list_with_filter(
        &self,
        filter: &NormalizedFilter,
        page: i64,
        page_size: i64,
    ) -> Result<Vec<NovelTerm>, DbError> {
        let mut builder: QueryBuilder<Sqlite> = QueryBuilder::new("SELECT * FROM novel_terms");
        push_where_clause(&mut builder, filter);
        builder.push(" ORDER BY created_at ASC, id ASC");

        if page_size > 0 {
            let offset = (effective_page(page) - 1) * page_size;
            builder.push(" LIMIT ");
            builder.push_bind(page_size);
            builder.push(" OFFSET ");
            builder.push_bind(offset);
        }

        let data = builder
            .build_query_as::<NovelTerm>()
            .fetch_all(&self.pool)
            .await?;
        Ok(data)
    }
}
