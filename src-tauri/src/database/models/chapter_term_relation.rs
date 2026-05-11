use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// 小说名词-章节关联实体
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct ChapterTermRelation {
    pub id: Uuid,
    pub novel_id: Uuid,
    pub chapter_id: Option<Uuid>,
    pub term_id: Uuid,
    pub description: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// 新建关联入参
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewChapterTermRelation {
    pub novel_id: Uuid,
    pub chapter_id: Option<Uuid>,
    pub term_id: Uuid,
    pub description: Option<String>,
}

/// 更新关联入参（支持更新 chapter_id）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateChapterTermRelation {
    pub chapter_id: Option<Uuid>,
    pub description: Option<String>,
}

/// 关联查询参数
#[derive(Debug, Clone, Default)]
pub struct ChapterTermRelationQuery {
    /// 可选：按章节ID筛选
    pub chapter_id: Option<Uuid>,
    /// 可选：按名词ID筛选
    pub term_id: Option<Uuid>,
    /// 页码（从1开始）
    pub page: i64,
    /// 每页数量（0表示查询全部）
    pub page_size: i64,
    /// 排序字段：created_at 或 updated_at
    pub sort_by: Option<String>,
    /// 排序方向：asc 或 desc（默认desc）
    pub sort_order: Option<String>,
}

/// 名词-章节关联与名词详情的联合查询结果
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct ChapterTermRelationWithTerm {
    pub relation_id: Uuid,
    pub chapter_id: Option<Uuid>,
    pub term_id: Uuid,
    pub description: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    // 名词详情
    pub term_name: String,
    pub term_type: String,
    pub term_description: Option<String>,
}
