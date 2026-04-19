use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

/// 小说实体模型
/// 对应数据库中的 novels 表，用于查询和展示小说信息
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Novel {
    pub id: Uuid,
    pub title: String,
    pub target_reader: String,
    pub description: String,
    pub cover_image: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// 标签实体模型
/// 对应数据库中的 tags 表，存储系统内所有的作品标签
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Tag {
    pub id: i64,
    pub name: String,
}

/// 创建小说请求参数
/// 用于接收前端创建新小说时提交的表单数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewNovel {
    pub title: String,
    pub target_reader: String,
    pub tag_ids: Vec<i64>,
    pub description: String,
}
