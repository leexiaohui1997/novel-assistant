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

/// 章节大纲条目（用于提示词上下文中渲染"全书章节标题"列表）
///
/// 每项代表小说中一个非草稿章节，按 (volumeSequence, chapterSequence) 升序输出。
/// 字段使用 camelCase 序列化以与既有提示词上下文风格保持一致。
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ChapterOutlineItem {
    /// 所属分卷的业务序号（孤儿章节归首卷=1）
    pub volume_sequence: i64,

    /// 章节业务序号（同卷内）
    pub chapter_sequence: i64,

    /// 章节标题
    pub title: String,
}
