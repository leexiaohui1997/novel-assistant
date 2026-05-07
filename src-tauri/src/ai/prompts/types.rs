use serde::Serialize;
use uuid::Uuid;

/// 角色信息（用于提示词上下文）
#[derive(Debug, Clone, Serialize)]
pub struct CharacterInfo {
    /// 角色 ID（仅用于内部筛选，不输出到模板）
    #[serde(skip)]
    pub id: Uuid,

    /// 角色名称
    pub name: String,

    /// 性别
    pub gender: String,

    /// 角色类型（可选，如：主角、配角等，已转为中文标签）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub character_type: Option<String>,

    /// 背景（可选）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub background: Option<String>,

    /// 外貌（可选）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub appearance: Option<String>,

    /// 性格（可选）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub personality: Option<String>,

    /// 其它描述（可选）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub additional_info: Option<String>,
}

/// 角色详细信息（用于优化）
#[derive(Debug, Serialize)]
pub struct CharacterDetail {
    /// 角色名称
    pub name: String,

    /// 性别
    pub gender: String,

    /// 背景（可选）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub background: Option<String>,

    /// 外貌（可选）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub appearance: Option<String>,

    /// 性格（可选）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub personality: Option<String>,

    /// 其它描述（可选）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub additional_info: Option<String>,
}

/// 带角色 ID 的角色信息（用于出场角色识别模板，ID 需输出到模板供 AI 引用）
#[derive(Debug, Clone, Serialize)]
pub struct CharacterWithIdInfo {
    /// 角色 ID（输出到模板，供 AI 识别已有角色时引用）
    pub id: String,

    /// 角色名称
    pub name: String,

    /// 性别
    pub gender: String,

    /// 角色类型（可选，如：主角、配角等，已转为中文标签）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub character_type: Option<String>,

    /// 背景（可选）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub background: Option<String>,

    /// 外貌（可选）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub appearance: Option<String>,

    /// 性格（可选）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub personality: Option<String>,

    /// 其它描述（可选）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub additional_info: Option<String>,
}
