use serde::Serialize;

use super::types::CharacterInfo;

/// edit_chapter_plot 模板的上下文数据
///
/// 与 EditChapterPositioningContext 字段一致，
/// 但模板中"任务要求"和"写作要求"不同，因此独立定义
#[derive(Debug, Serialize)]
pub struct EditChapterPlotContext {
    /// 小说标题
    pub title: String,

    /// 频道名称（男频/女频）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub channel_name: Option<String>,

    /// 标签（可选）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tags: Option<String>,

    /// 小说简介（可选）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// 章节序号（1-based，用于显示"第N章"）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub chapter_sequence: Option<i64>,

    /// 卷序号（1-based，用于显示"第N卷"）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub volume_sequence: Option<i64>,

    /// 卷名（可选）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub volume_name: Option<String>,

    /// 大纲定位（可选）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub outline_positioning: Option<String>,

    /// 本章剧情（可选）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub outline_plot: Option<String>,

    /// 出场角色（可选，大纲关联的角色子集）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub outline_characters: Option<Vec<CharacterInfo>>,

    /// 角色列表（可选）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub characters: Option<Vec<CharacterInfo>>,

    /// 章节标题（可选）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub chapter_title: Option<String>,

    /// 章节正文（可选）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub chapter_content: Option<String>,

    /// 前情介绍（可选）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub previous_plots: Option<String>,

    /// 用户意见（可选）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_feedback: Option<String>,
}
