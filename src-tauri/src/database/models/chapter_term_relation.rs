use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::novel_term::{NovelTerm, TermType};

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

/// 名词-章节关联与名词详情的联合查询结果（对外结构）
///
/// 名词信息以嵌套的 `NovelTerm` 形式表达，避免与关联自身字段命名冲突。
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChapterTermRelationWithTerm {
    pub relation_id: Uuid,
    pub chapter_id: Option<Uuid>,
    pub description: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    /// 关联的名词主表完整信息
    pub term: NovelTerm,
}

/// JOIN 查询的扁平行结构（仅 repo 内部使用）
///
/// 用于配合 `sqlx::FromRow` 反序列化 JOIN 结果，
/// 通过 `From` 转换为对外的 `ChapterTermRelationWithTerm`。
#[derive(Debug, Clone, sqlx::FromRow)]
pub(crate) struct ChapterTermRelationWithTermRow {
    pub relation_id: Uuid,
    pub chapter_id: Option<Uuid>,
    pub description: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub term_id: Uuid,
    pub term_novel_id: Uuid,
    pub term_type: TermType,
    pub term_name: String,
    pub term_description: Option<String>,
    pub term_created_at: DateTime<Utc>,
    pub term_updated_at: DateTime<Utc>,
}

impl From<ChapterTermRelationWithTermRow> for ChapterTermRelationWithTerm {
    fn from(row: ChapterTermRelationWithTermRow) -> Self {
        Self {
            relation_id: row.relation_id,
            chapter_id: row.chapter_id,
            description: row.description,
            created_at: row.created_at,
            updated_at: row.updated_at,
            term: NovelTerm {
                id: row.term_id,
                novel_id: row.term_novel_id,
                term_type: row.term_type,
                name: row.term_name,
                description: row.term_description,
                created_at: row.term_created_at,
                updated_at: row.term_updated_at,
            },
        }
    }
}
