/// 分页查询工具函数
///
/// 提供通用的分页查询能力，支持任意类型的数据库查询。
///
/// # 使用示例
///
/// ```rust
/// use crate::utils::pagination::{PaginationParams, PaginatedResult, query_with_pagination};
///
/// async fn find_users_with_pagination(pool: &SqlitePool, page: i64, page_size: i64) -> Result<PaginatedResult<User>, DbError> {
///     let params = PaginationParams { page, page_size };
///     
///     query_with_pagination(
///         pool,
///         &params,
///         "SELECT COUNT(*) FROM users",
///         "SELECT * FROM users ORDER BY created_at DESC LIMIT ? OFFSET ?",
///     ).await
/// }
/// ```
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, SqlitePool};

use crate::database::error::DbError;

/// 分页查询参数
#[derive(Debug, Clone, Deserialize)]
pub struct PaginationParams {
    /// 页码（从 1 开始）
    pub page: i64,
    /// 每页数量
    pub page_size: i64,
}

/// 分页响应结果
#[derive(Debug, Clone, Serialize)]
pub struct PaginatedResult<T> {
    /// 数据列表
    pub data: Vec<T>,
    /// 总数量
    pub total: i64,
}

/// 通用分页查询函数
///
/// # 参数
///
/// * `pool` - 数据库连接池
/// * `params` - 分页参数（页码和每页数量）
/// * `count_sql` - 查询总数的 SQL 语句
/// * `data_sql` - 查询数据的 SQL 语句（必须包含 LIMIT 和 OFFSET 占位符）
///
/// # 返回值
///
/// 返回包含数据列表和总数的分页结果
///
/// # 示例
///
/// ```rust
/// let result = query_with_pagination::<User>(
///     &pool,
///     &PaginationParams { page: 1, page_size: 10 },
///     "SELECT COUNT(*) FROM novels",
///     "SELECT * FROM novels ORDER BY created_at DESC LIMIT ? OFFSET ?",
/// ).await?;
/// ```
pub async fn query_with_pagination<T>(
    pool: &SqlitePool,
    params: &PaginationParams,
    count_sql: &str,
    data_sql: &str,
) -> Result<PaginatedResult<T>, DbError>
where
    T: for<'r> FromRow<'r, sqlx::sqlite::SqliteRow> + Send + Unpin,
{
    // 计算偏移量
    let offset = (params.page - 1) * params.page_size;

    // 1. 查询总数
    let total: (i64,) = sqlx::query_as(count_sql).fetch_one(pool).await?;

    // 2. 查询分页数据
    let data = sqlx::query_as::<_, T>(data_sql)
        .bind(params.page_size)
        .bind(offset)
        .fetch_all(pool)
        .await?;

    Ok(PaginatedResult {
        data,
        total: total.0,
    })
}
