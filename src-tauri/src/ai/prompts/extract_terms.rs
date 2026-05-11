use serde::Serialize;

/// 提取名词提示词模板的上下文
///
/// 包含从正文中提取名词所需的全部信息。
#[derive(Debug, Serialize)]
pub struct ExtractTermsContext {
    /// 现有名词列表（Markdown 格式，由 fragments 生成）
    pub existing_terms_md: String,
    /// 可用名词类型列表
    pub available_types: String,
    /// 正文内容
    pub content: String,
}
