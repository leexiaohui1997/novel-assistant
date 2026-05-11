use serde::Serialize;

/// novel_terms 提示词片段模板的顶层上下文
///
/// 描述一段"小说相关名词"列表的 Markdown 片段所需的全部数据。
/// 该上下文被 `fragments/novel_terms.tera` 模板消费。
///
/// # 字段
/// - `groups`: 按"名词类型"聚合的分组列表；顺序由调用方决定，
///   模板按顺序原样渲染（参见 `fragments::group_terms_by_type`）。
/// - `show_id`: 是否在每条名词后输出 `[ID: xxx]` 片段。
#[derive(Debug, Serialize)]
pub struct NovelTermsContext {
    /// 分组后的名词列表
    pub groups: Vec<NovelTermGroup>,
    /// 是否展示名词 ID
    pub show_id: bool,
}

/// 单个名词类型分组
///
/// # 字段
/// - `type_label`: 分组标题（中文标签，如 "人物"、"地域 & 场景"）。
/// - `items`: 该类型下的名词条目列表。
#[derive(Debug, Serialize)]
pub struct NovelTermGroup {
    /// 分组标题（中文标签）
    pub type_label: String,
    /// 分组内的名词条目
    pub items: Vec<NovelTermItem>,
}

/// 单条名词条目
///
/// # 字段
/// - `id`: 名词主键（`Uuid` 默认连字符小写字符串形式）。
/// - `name`: 名词名称。
/// - `description`: 名词描述；空字符串表示模板层应省略描述段。
#[derive(Debug, Serialize)]
pub struct NovelTermItem {
    /// 名词 ID（字符串形式）
    pub id: String,
    /// 名词名称
    pub name: String,
    /// 名词描述（空字符串代表无描述）
    pub description: String,
}
