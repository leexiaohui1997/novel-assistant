use std::collections::BTreeMap;
use std::sync::Arc;

use serde_json::{json, Value};
use tokio::sync::RwLock;

use crate::database::models::tag::{Tag, TagType, TargetAudience};
use crate::database::repositories::TagRepository;

use super::super::traits::AiTool;

/// 标签查询工具
///
/// 让 AI 在与用户协作创作小说时（特别是「优化基础信息」「推荐标签」等场景），
/// 主动按分类或按名称查询系统标签库，了解可用标签全集。
///
/// 仅做只读查询，入参均可选。
pub struct SearchTagsTool {
    tag_repo: Arc<RwLock<Box<dyn TagRepository + Send + Sync>>>,
}

impl SearchTagsTool {
    pub fn new(tag_repo: Arc<RwLock<Box<dyn TagRepository + Send + Sync>>>) -> Self {
        Self { tag_repo }
    }
}

/// 解析 `categories` 入参：
/// - 缺省 / null / 空数组 → `Ok(None)`（不按分类筛选）
/// - 非数组（类型错误）→ `Ok(None)`（参考需求边界：等同未提供）
/// - 数组中含非法值 → `Err(String)`，错误信息携带非法值与允许取值列表
fn parse_categories(args: &Value) -> Result<Option<Vec<TagType>>, String> {
    let Some(arr) = args.get("categories").and_then(|v| v.as_array()) else {
        return Ok(None);
    };
    if arr.is_empty() {
        return Ok(None);
    }

    let mut out: Vec<TagType> = Vec::with_capacity(arr.len());
    for item in arr {
        let Some(s) = item.as_str() else {
            return Err(format!(
                "非法的分类值类型，期望字符串，允许取值: {}",
                allowed_categories_hint()
            ));
        };
        let parsed: Result<TagType, _> = serde_json::from_value(Value::String(s.to_string()));
        match parsed {
            Ok(t) => out.push(t),
            Err(_) => {
                return Err(format!(
                    "非法的分类: {}，允许取值: {}",
                    s,
                    allowed_categories_hint()
                ));
            }
        }
    }
    Ok(Some(out))
}

/// 解析 `names` 入参：缺省 / null / 空数组 → `None`；保持原值（不 trim）
fn parse_names(args: &Value) -> Option<Vec<String>> {
    let arr = args.get("names")?.as_array()?;
    if arr.is_empty() {
        return None;
    }
    let names: Vec<String> = arr
        .iter()
        .filter_map(|v| v.as_str().map(|s| s.to_string()))
        .collect();
    if names.is_empty() {
        None
    } else {
        Some(names)
    }
}

/// 解析 `target_audience` 入参（单值字符串）：
/// - 缺省 / null / 空串 / 类型错误 → `Ok(None)`（不按频道筛选）
/// - 非法值 → `Err(String)`，错误信息携带非法值与允许取值列表
fn parse_target_audience(args: &Value) -> Result<Option<TargetAudience>, String> {
    let Some(raw) = args.get("target_audience") else {
        return Ok(None);
    };
    if raw.is_null() {
        return Ok(None);
    }
    let Some(s) = raw.as_str() else {
        return Ok(None);
    };
    if s.is_empty() {
        return Ok(None);
    }
    let parsed: Result<TargetAudience, _> = serde_json::from_value(Value::String(s.to_string()));
    parsed.map(Some).map_err(|_| {
        format!(
            "非法的频道: {}，允许取值: {}",
            s,
            allowed_target_audiences_hint()
        )
    })
}

/// 拉取标签：
/// - 提供 `names` 走 `find_by_names`
/// - 否则走 `find_all`
async fn fetch_tags(
    tag_repo: Arc<RwLock<Box<dyn TagRepository + Send + Sync>>>,
    names: Option<&[String]>,
) -> Result<Vec<Tag>, String> {
    let repo = tag_repo.read().await;
    match names {
        Some(list) => {
            let refs: Vec<&str> = list.iter().map(|s| s.as_str()).collect();
            repo.find_by_names(&refs)
                .await
                .map_err(|e| format!("查询标签失败: {}", e))
        }
        None => repo
            .find_all()
            .await
            .map_err(|e| format!("查询标签失败: {}", e)),
    }
}

/// 内存中按分类过滤；`categories=None` 时不过滤
fn filter_by_categories(tags: Vec<Tag>, categories: Option<&[TagType]>) -> Vec<Tag> {
    let Some(cats) = categories else {
        return tags;
    };
    tags.into_iter()
        .filter(|t| cats.iter().any(|c| *c == t.tag_type))
        .collect()
}

/// 内存中按频道精确过滤；`target_audience=None` 时不过滤
fn filter_by_target_audience(tags: Vec<Tag>, target_audience: Option<TargetAudience>) -> Vec<Tag> {
    let Some(ta) = target_audience else {
        return tags;
    };
    tags.into_iter()
        .filter(|t| t.target_audience == ta)
        .collect()
}

/// 将 `TagType` 转换为返回 JSON 中分组的 key（snake_case），与入参 categories 取值保持一致
fn tag_type_key(t: TagType) -> &'static str {
    match t {
        TagType::MainCategory => "main_category",
        TagType::Theme => "theme",
        TagType::Character => "character",
        TagType::Plot => "plot",
    }
}

/// 将 `TargetAudience` 转换为返回 JSON 中标签条目的频道字段值（snake_case），与入参 target_audience 取值保持一致
fn target_audience_key(a: TargetAudience) -> &'static str {
    match a {
        TargetAudience::Male => "male",
        TargetAudience::Female => "female",
        TargetAudience::Both => "both",
    }
}

fn allowed_categories_hint() -> &'static str {
    "main_category | theme | character | plot"
}

fn allowed_target_audiences_hint() -> &'static str {
    "male | female | both"
}

/// 将标签按分类分组，并序列化为目标 JSON 字符串：
/// - key 为分类标识（snake_case）
/// - value 为标签数组，每条仅含 `id` / `name` / `description`
/// - 空分类省略 key；空命中输出 `"{}"`
/// - 同一分类下按 `id` 升序
fn group_and_serialize(tags: Vec<Tag>) -> Result<String, String> {
    let mut grouped: BTreeMap<&'static str, Vec<Value>> = BTreeMap::new();
    let mut sorted = tags;
    sorted.sort_by_key(|t| t.id);
    for t in sorted {
        let key = tag_type_key(t.tag_type);
        let entry = json!({
            "id": t.id,
            "name": t.name,
            "description": t.description,
            "target_audience": target_audience_key(t.target_audience),
        });
        grouped.entry(key).or_default().push(entry);
    }
    serde_json::to_string(&grouped).map_err(|e| format!("序列化结果失败: {}", e))
}

impl AiTool for SearchTagsTool {
    fn name(&self) -> &str {
        "search_tags"
    }

    fn description(&self) -> &str {
        "查询系统标签库：支持按分类（main_category/theme/character/plot）筛选、按频道（male/female/both）筛选、按名称精准匹配筛选；返回按分类分组的标签结构（每条含 id、name、description、target_audience），便于直接生成可被前端选中的标签建议。"
    }

    fn parameters_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "categories": {
                    "type": "array",
                    "description": "按分类筛选（可选）。缺省、null 或空数组表示不限分类。",
                    "items": {
                        "type": "string",
                        "enum": ["main_category", "theme", "character", "plot"]
                    }
                },
                "target_audience": {
                    "type": "string",
                    "description": "按频道精确筛选（可选，单值）。缺省、null 或空串表示不限频道；注意 both 仅匹配标签自身频道为「通用」的标签，不会自动并入 male/female。",
                    "enum": ["male", "female", "both"]
                },
                "names": {
                    "type": "array",
                    "description": "按标签名称精准匹配筛选（可选，区分大小写、不做 trim）。缺省、null 或空数组表示不限名称。",
                    "items": { "type": "string" }
                }
            }
        })
    }

    fn execute(
        &self,
        args: Value,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<String, String>> + Send>> {
        let tag_repo = self.tag_repo.clone();
        Box::pin(async move {
            let categories = parse_categories(&args)?;
            let target_audience = parse_target_audience(&args)?;
            let names = parse_names(&args);

            let tags = fetch_tags(tag_repo, names.as_deref()).await?;
            let filtered = filter_by_categories(tags, categories.as_deref());
            let filtered = filter_by_target_audience(filtered, target_audience);
            group_and_serialize(filtered)
        })
    }
}
