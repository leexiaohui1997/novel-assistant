use serde::Serialize;

/// generate_introduction 模板的上下文数据
#[derive(Debug, Serialize)]
pub struct GenerateIntroductionContext {
    /// 标题（可选）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,

    /// 频道名称（男频/女频，可选）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub channel_name: Option<String>,

    /// 已选标签信息（可选）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub selected_tags: Option<String>,

    /// 用户意见（可选）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_feedback: Option<String>,
}
