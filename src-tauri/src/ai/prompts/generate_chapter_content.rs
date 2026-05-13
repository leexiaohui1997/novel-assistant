use serde::Serialize;

use super::types::CharacterInfo;

/// generate_chapter_content 模板的上下文数据
///
/// 用于 AI 生成章节正文，包含小说信息、角色、前情、大纲、
/// 章节内容、引用内容、用户意见和目标字数等完整上下文
#[derive(Debug, Serialize)]
pub struct GenerateChapterContentContext {
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

    /// 角色列表（可选）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub characters: Option<Vec<CharacterInfo>>,

    /// 前情介绍（可选）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub previous_plots: Option<String>,

    /// 上一章正文（可选）
    ///
    /// 紧邻上一章的完整正文文本，作为"前情介绍（大纲剧情汇总）"
    /// 的细粒度补充，帮助 AI 在开篇过渡、情节延续、人物状态等
    /// 微观细节上自然衔接。
    /// 上一章不存在或正文为空时为 `None`，由模板兜底为"暂无上一章正文"。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub previous_chapter_content: Option<String>,

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

    /// 章节标题（可选）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub chapter_title: Option<String>,

    /// 章节正文（可选，已有正文时在提示词中指导续写或改写）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub chapter_content: Option<String>,

    /// 引用内容（可选，字符串数组，如"正文第n个字~第m个字"）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub references: Option<Vec<String>>,

    /// 目标字数（默认 2500）
    pub target_word_count: i64,

    /// 用户意见（可选）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_feedback: Option<String>,

    /// 已有名词列表 Markdown 片段（可选）
    ///
    /// 由 `render_novel_terms_fragment` 装配后写入；在主模板的
    /// `## 名词列表` 小节直接渲染。空字符串/缺失时由模板兜底为"暂无名词"。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub existing_terms_md: Option<String>,
}
