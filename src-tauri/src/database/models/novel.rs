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

/// 小说统计信息
///
/// 聚合单本小说非草稿章节的总章节数、总字数、最近更新时间与最近更新章节位置信息。
/// 用于作品管理列表卡片展示。
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NovelStats {
    /// 小说 ID
    pub novel_id: Uuid,
    /// 章节数（非草稿章节数量）
    pub chapter_count: i64,
    /// 总字数（所有非草稿章节 word_count 之和）
    pub total_word_count: i64,
    /// 最近更新时间（最近一个非草稿章节的 updated_at）
    pub last_updated_at: Option<DateTime<Utc>>,
    /// 最近更新章节所属分卷序号；无分卷关联时为 None
    pub last_updated_volume_sequence: Option<i64>,
    /// 最近更新章节自身的 sequence
    pub last_updated_chapter_sequence: Option<i64>,
    /// 最近更新章节的标题
    pub last_updated_chapter_title: Option<String>,
}

/// 小说详情模型（含标签）
/// 在 Novel 基础上关联了标签信息，用于需要展示标签的场景
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NovelWithTags {
    #[serde(flatten)]
    pub novel: Novel,
    #[serde(default)]
    pub tags: Vec<Tag>,
    /// 统计信息（仅在 with_stats 时有值）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stats: Option<NovelStats>,
}

impl NovelWithTags {
    /// 从 Novel 创建，tags 为空列表
    pub fn from_novel(novel: Novel) -> Self {
        Self {
            novel,
            tags: vec![],
            stats: None,
        }
    }

    /// 从 Novel 和 Tag 列表创建
    pub fn with_tags(novel: Novel, tags: Vec<Tag>) -> Self {
        Self {
            novel,
            tags,
            stats: None,
        }
    }

    /// 附加统计信息
    pub fn with_stats(mut self, stats: Option<NovelStats>) -> Self {
        self.stats = stats;
        self
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
