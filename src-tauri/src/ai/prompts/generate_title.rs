use serde::Serialize;

/// generate_title 模板的上下文数据
#[derive(Debug, Serialize)]
pub struct GenerateTitleContext {
    /// 频道名称（男频/女频，可选）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub channel_name: Option<String>,

    /// 标签信息（可选）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tags: Option<String>,

    /// 作品简介（可选）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub introduction: Option<String>,

    /// 用户意见（可选）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_feedback: Option<String>,
}
