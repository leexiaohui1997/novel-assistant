use serde::Serialize;

/// recommend_tags 模板的上下文数据
#[derive(Debug, Serialize)]
pub struct RecommendTagsContext {
    /// 频道名称（男频/女频）
    pub channel_name: String,

    /// 标题（可选）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,

    /// 简介（可选）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub introduction: Option<String>,

    /// 已选标签信息（可选）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub selected_tags_info: Option<String>,

    /// 可用标签列表
    pub main_categories: String,
    pub themes: String,
    pub characters: String,
    pub plots: String,

    /// 限制数量
    pub main_limit: usize,
    pub theme_limit: usize,
    pub character_limit: usize,
    pub plot_limit: usize,

    /// 剩余可推荐数量
    pub main_remaining: usize,
    pub theme_remaining: usize,
    pub character_remaining: usize,
    pub plot_remaining: usize,

    /// 用户意见（可选）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_feedback: Option<String>,
}
