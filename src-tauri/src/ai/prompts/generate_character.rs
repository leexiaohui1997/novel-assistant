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

    /// 相关文章内容（可选）
    ///
    /// 用户提供的一段参考文本（如已有作品的相关原文片段、设定文档节选等），
    /// 在模板中以 `## 相关文章内容` 段落渲染；空白或未提供时不渲染。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reference_content: Option<String>,

    /// 角色类型枚举定义（用于动态注入模板）
    pub character_type_options: Vec<CharacterTypeOption>,
}

/// 角色类型选项
#[derive(Debug, Serialize)]
pub struct CharacterTypeOption {
    /// 枚举值（snake_case）
    pub value: String,
    /// 中文标签
    pub label: String,
}
