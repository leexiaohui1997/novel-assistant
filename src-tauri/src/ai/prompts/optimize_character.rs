use serde::Serialize;

use super::types::CharacterDetail;

/// optimize_character 模板的上下文数据
#[derive(Debug, Serialize)]
pub struct OptimizeCharacterContext {
    /// 小说标题
    pub title: String,

    /// 频道名称（男频/女频）
    pub channel_name: String,

    /// 标签信息（可选）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tags: Option<String>,

    /// 作品简介（可选）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub introduction: Option<String>,

    /// 待优化的角色信息
    pub character: CharacterDetail,

    /// 需要优化的字段列表
    pub optimize_fields: Vec<String>,

    /// 用户优化意见（可选）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_feedback: Option<String>,
}
