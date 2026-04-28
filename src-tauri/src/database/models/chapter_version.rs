use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

/// 章节历史版本实体
///
/// 每次更新章节的标题或正文前，会将"更新前"的快照写入该表，
/// 用于前端展示历史版本列表以及"应用历史版本"功能。
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
#[serde(rename_all = "camelCase")]
pub struct ChapterVersion {
    pub id: Uuid,
    pub chapter_id: Uuid,
    pub title: String,
    pub content: String,
    pub word_count: i64,
    /// 该版本文章的更新时间（即快照时刻章节的 `updated_at`）
    pub saved_at: DateTime<Utc>,
    /// 快照记录的插入时间
    pub created_at: DateTime<Utc>,
}
