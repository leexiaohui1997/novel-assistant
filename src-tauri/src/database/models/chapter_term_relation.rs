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

/// 名词-章节关联与章节定位信息的联合查询结果（对外结构）
///
/// 仅用于"按名词 ID 反查章节关联列表"的场景，因此 `chapter_id` 必非空，
/// 同时附带章节标题、章节序号与所属分卷序号，方便前端直接渲染可跳转列表。
///
/// 排序约定：先按 `volume_sequence` 升序，再按 `chapter_sequence` 升序。
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChapterTermRelationWithChapter {
    /// 关联记录主键
    pub relation_id: Uuid,
    /// 所属小说 ID
    pub novel_id: Uuid,
    /// 关联的名词 ID
    pub term_id: Uuid,
    /// 关联的章节 ID（本场景必非空）
    pub chapter_id: Uuid,
    /// 关联自身的描述（可空）
    pub description: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    /// 章节标题
    pub chapter_title: String,
    /// 章节业务序号（草稿为 -1）
    pub chapter_sequence: i64,
    /// 所属分卷的业务序号；若未关联分卷则按"默认归属首卷"约定取 1
    pub volume_sequence: i64,
}

/// JOIN 查询的扁平行结构（仅 repo 内部使用）
///
/// 用于配合 `sqlx::FromRow` 反序列化三表 JOIN 结果，
/// 通过 `From` 转换为对外的 `ChapterTermRelationWithChapter`。
#[derive(Debug, Clone, sqlx::FromRow)]
pub(crate) struct ChapterTermRelationWithChapterRow {
    pub relation_id: Uuid,
    pub novel_id: Uuid,
    pub term_id: Uuid,
    pub chapter_id: Uuid,
    pub description: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub chapter_title: String,
    pub chapter_sequence: i64,
    pub volume_sequence: i64,
}

impl From<ChapterTermRelationWithChapterRow> for ChapterTermRelationWithChapter {
    fn from(row: ChapterTermRelationWithChapterRow) -> Self {
        Self {
            relation_id: row.relation_id,
            novel_id: row.novel_id,
            term_id: row.term_id,
            chapter_id: row.chapter_id,
            description: row.description,
            created_at: row.created_at,
            updated_at: row.updated_at,
            chapter_title: row.chapter_title,
            chapter_sequence: row.chapter_sequence,
            volume_sequence: row.volume_sequence,
        }
    }
}
