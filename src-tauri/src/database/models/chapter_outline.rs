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
    pub positioning: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
