use serde::Serialize;

use super::types::CharacterInfo;

/// generate_character 模板的上下文数据
#[derive(Debug, Serialize)]
pub struct GenerateCharacterContext {
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

    /// 已有角色列表
    #[serde(skip_serializing_if = "Option::is_none")]
    pub existing_characters: Option<Vec<CharacterInfo>>,

    /// 用户意见（可选）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_feedback: Option<String>,
}
