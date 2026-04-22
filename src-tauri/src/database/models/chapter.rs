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
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NewChapter {
    pub title: String,
    pub content: String,
    pub volume_id: Option<i64>,
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
