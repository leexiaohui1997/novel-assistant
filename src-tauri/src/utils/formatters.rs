use crate::database::models::tag::{Tag, TagType};

/// 格式化频道显示文本
///
/// # 参数
/// - `target_reader`: 目标读者标识 ("male" 或 "female")
///
/// # 返回
/// - 格式化后的字符串，如 "男频（male）" 或 "女频（female）"
pub fn format_channel(target_reader: &str) -> String {
    match target_reader {
        "male" => "男频(male)".to_string(),
        "female" => "女频(female)".to_string(),
        "both" => "男频+女频(both)".to_string(),
        _ => "未知频道".to_string(),
    }
}

/// 格式化标签为 JSON 对象
///
/// # 参数
/// - `tags`: 标签列表
///
/// # 返回
/// - JSON 字符串，格式为 { "类型": ["标签1", "标签2"] }
pub fn format_tags(tags: &[Tag]) -> String {
    use std::collections::HashMap;

    let mut grouped: HashMap<&str, Vec<String>> = HashMap::new();
    for t in tags {
        let type_label = match t.tag_type {
            TagType::MainCategory => "主分类",
            TagType::Theme => "主题",
            TagType::Character => "角色",
            TagType::Plot => "情节",
        };
        grouped.entry(type_label).or_default().push(t.name.clone());
    }

    serde_json::to_string(&grouped).unwrap_or_default()
}
