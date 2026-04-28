use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

/// 章节实体模型
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
#[serde(rename_all = "camelCase")]
pub struct Chapter {
    pub id: Uuid,
    pub novel_id: Uuid,
    pub title: String,
    pub content: String,
    pub word_count: i64,
    pub sequence: i64,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// 章节接口响应 DTO
///
/// 在数据库 `Chapter` 基础上扩展接口层字段（如 `deletable`），
/// 仅用于 API 返回，不参与数据库映射。
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChapterResponse {
    pub id: Uuid,
    pub novel_id: Uuid,
    pub title: String,
    pub content: String,
    pub word_count: i64,
    pub sequence: i64,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    /// 是否可删除（仅分卷下最后一个非草稿章节为 true）
    pub deletable: bool,
}

impl ChapterResponse {
    /// 基于数据库 `Chapter` 和 `deletable` 标识构造响应 DTO。
    pub fn from_chapter(chapter: Chapter, deletable: bool) -> Self {
        Self {
            id: chapter.id,
            novel_id: chapter.novel_id,
            title: chapter.title,
            content: chapter.content,
            word_count: chapter.word_count,
            sequence: chapter.sequence,
            created_at: chapter.created_at,
            updated_at: chapter.updated_at,
            deletable,
        }
    }
}

/// 分卷实体模型
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
#[serde(rename_all = "camelCase")]
pub struct Volume {
    pub id: i64,
    pub novel_id: Uuid,
    pub name: String,
    pub sequence: i64,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// 创建分卷请求
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NewVolume {
    pub novel_id: Uuid,
    pub name: String,
    pub sequence: i64,
}

/// 更新分卷请求
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateVolume {
    pub name: String,
    pub sequence: i64,
}

/// 创建章节请求
///
/// `sequence` 省略（或 `None`）时，后端写入 -1 视为草稿；
/// 传入 `>=0` 则直接作为正式章节序号落库。
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NewChapter {
    pub title: String,
    pub content: String,
    pub volume_id: Option<i64>,
    pub sequence: Option<i64>,
}

/// 更新章节请求
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateChapter {
    pub title: String,
    pub content: String,
    pub sequence: Option<i64>,
    pub volume_id: Option<i64>,
}

/// 分页查询章节请求
///
/// 业务说明（`volume_sequence` 为分卷的业务序号，非主键 id）：
/// - `is_draft = true`：仅查询草稿章节（sequence = -1），忽略 `volume_sequence`
/// - `is_draft = false` + `volume_sequence = Some(1)`：查非草稿且（属于首卷或无分卷关联）
/// - `is_draft = false` + `volume_sequence = Some(N)`（N != 1）：查非草稿且属于 sequence=N 的分卷
/// - `is_draft = false` + `volume_sequence = None`：查所有非草稿章节
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct ChapterQuery {
    /// 分卷业务序号（非草稿模式下有效）
    pub volume_sequence: Option<i64>,
    /// 是否草稿模式
    #[serde(default)]
    pub is_draft: bool,
    /// 排序字段：sequence / title / created_at / updated_at
    pub sort_field: Option<String>,
    /// 排序顺序：asc / desc
    pub sort_order: Option<String>,
}

/// 批量编辑分卷请求项
///
/// 用于 `batch_update_volumes` 接口：
/// - `id = Some(_)`：修改已有分卷；
/// - `id = None`：新增分卷。
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VolumeUpsert {
    /// 分卷 ID；为 None 时表示新增
    pub id: Option<i64>,
    pub name: String,
    pub sequence: i64,
}
