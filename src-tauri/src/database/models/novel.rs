use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

use crate::database::models::tag::Tag;

/// 小说实体模型
/// 对应数据库中的 novels 表，用于查询和展示小说信息
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
#[serde(rename_all = "camelCase")]
pub struct Novel {
    pub id: Uuid,
    pub title: String,
    pub target_reader: String,
    pub description: String,
    pub cover_image: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// 小说详情模型（含标签）
/// 在 Novel 基础上关联了标签信息，用于需要展示标签的场景
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NovelWithTags {
    #[serde(flatten)]
    pub novel: Novel,
    #[serde(default)]
    pub tags: Vec<Tag>,
}

impl NovelWithTags {
    /// 从 Novel 创建，tags 为空列表
    pub fn from_novel(novel: Novel) -> Self {
        Self {
            novel,
            tags: vec![],
        }
    }

    /// 从 Novel 和 Tag 列表创建
    pub fn with_tags(novel: Novel, tags: Vec<Tag>) -> Self {
        Self { novel, tags }
    }
}

/// 创建小说请求参数
/// 用于接收前端创建新小说时提交的表单数据
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NewNovel {
    pub title: String,
    pub target_reader: String,
    pub tag_ids: Vec<i64>,
    pub description: String,
}

/// 更新小说请求参数
/// 用于接收前端编辑小说时提交的表单数据
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateNovel {
    pub title: String,
    pub target_reader: String,
    pub tag_ids: Vec<i64>,
    pub description: String,
}
