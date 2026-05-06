use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// 章节大纲实体模型
/// 对应数据库中的 chapter_outlines 表，存储章节的大纲信息
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct ChapterOutline {
    pub id: i64,
    pub novel_id: Uuid,
    pub chapter_id: Option<Uuid>,
    pub positioning: Option<String>,
    pub plot: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// 章节大纲（含关联角色ID列表）
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChapterOutlineWithCharacters {
    #[serde(flatten)]
    pub outline: ChapterOutline,
    pub character_ids: Vec<Uuid>,
}
