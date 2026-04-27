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
/// 业务说明：
/// - `is_draft = true`：仅查询草稿章节（sequence = -1），忽略 `volume_id`
/// - `is_draft = false` + `volume_id = Some(1)`：查非草稿且（属于分卷 1 或无分卷关联）
/// - `is_draft = false` + `volume_id = Some(N)`（N != 1）：查非草稿且属于该分卷
/// - `is_draft = false` + `volume_id = None`：查所有非草稿章节
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct ChapterQuery {
    /// 分卷序号（非草稿模式下有效）
    pub volume_id: Option<i64>,
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
