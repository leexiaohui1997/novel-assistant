use tauri::State;
use uuid::Uuid;

use crate::database::models::novel_term::{NovelTerm, TermType, ALL_TERM_TYPES};
use crate::database::repositories::NovelTermQuery;
use crate::utils::pagination::PaginatedResult;
use crate::AppState;

/// 将前端传入的 UUID 字符串解析为 `Uuid`
fn parse_uuid(field_label: &str, raw: &str) -> Result<Uuid, String> {
    Uuid::parse_str(raw).map_err(|e| format!("{} 解析失败: {}", field_label, e))
}

/// 列出全部合法的名词类型字面量（snake_case），用于错误文案
fn legal_term_type_values() -> String {
    ALL_TERM_TYPES
        .iter()
        .map(|t| t.as_str())
        .collect::<Vec<_>>()
        .join("、")
}

/// 将前端字符串解析为 `TermType`
///
/// 由于 `string_enum!` 默认模式生成的 `FromStr` 仅匹配成员名原样，
/// 这里通过 `serde_json::from_value` 借助派生的 `Deserialize` 走 snake_case 解码。
fn parse_term_type(raw: &str) -> Result<TermType, String> {
    let value = serde_json::Value::String(raw.to_string());
    serde_json::from_value::<TermType>(value).map_err(|_| {
        format!(
            "无效的名词类型 \"{}\"，必须是以下值之一: {}",
            raw,
            legal_term_type_values()
        )
    })
}

/// 解析可选名词类型：`None` 或 trim 后空串视为未设置
fn parse_optional_term_type(raw: Option<String>) -> Result<Option<TermType>, String> {
    let Some(value) = raw else {
        return Ok(None);
    };
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return Ok(None);
    }
    parse_term_type(trimmed).map(Some)
}

/// 创建小说名词
#[tauri::command]
pub async fn create_novel_term(
    novel_id: String,
    term_type: String,
    name: String,
    description: Option<String>,
    state: State<'_, AppState>,
) -> Result<NovelTerm, String> {
    let novel_uuid = parse_uuid("novelId", &novel_id)?;
    let term_type = parse_term_type(&term_type)?;
    let now = chrono::Utc::now();

    let term = NovelTerm {
        id: Uuid::nil(), // 由 Repo 自行生成
        novel_id: novel_uuid,
        term_type,
        name,
        description,
        created_at: now,
        updated_at: now,
    };

    let repo = state.novel_term_repo.read().await;
    repo.create(&term).await.map_err(|e| {
        tracing::error!(op = "create_novel_term", novel_id = %novel_uuid, error = %e);
        format!("创建名词失败: {}", e)
    })
}

/// 更新小说名词
#[tauri::command]
pub async fn update_novel_term(
    id: String,
    term_type: String,
    name: String,
    description: Option<String>,
    state: State<'_, AppState>,
) -> Result<NovelTerm, String> {
    let id_uuid = parse_uuid("id", &id)?;
    let term_type = parse_term_type(&term_type)?;

    let repo = state.novel_term_repo.read().await;
    let existing = repo
        .find_by_id(&id_uuid)
        .await
        .map_err(|e| {
            tracing::error!(op = "update_novel_term", id = %id_uuid, error = %e);
            format!("更新名词失败: {}", e)
        })?
        .ok_or_else(|| format!("更新名词失败: 名词 {} 不存在", id_uuid))?;

    let term = NovelTerm {
        id: existing.id,
        novel_id: existing.novel_id,
        term_type,
        name,
        description,
        created_at: existing.created_at,
        updated_at: chrono::Utc::now(),
    };

    repo.update(&term).await.map_err(|e| {
        tracing::error!(op = "update_novel_term", id = %id_uuid, error = %e);
        format!("更新名词失败: {}", e)
    })
}

/// 删除小说名词
#[tauri::command]
pub async fn delete_novel_term(id: String, state: State<'_, AppState>) -> Result<(), String> {
    let id_uuid = parse_uuid("id", &id)?;
    let repo = state.novel_term_repo.read().await;
    repo.delete(&id_uuid).await.map_err(|e| {
        tracing::error!(op = "delete_novel_term", id = %id_uuid, error = %e);
        format!("删除名词失败: {}", e)
    })
}

/// 根据 ID 获取小说名词
#[tauri::command]
pub async fn get_novel_term_by_id(
    id: String,
    state: State<'_, AppState>,
) -> Result<NovelTerm, String> {
    let id_uuid = parse_uuid("id", &id)?;
    let repo = state.novel_term_repo.read().await;
    let term = repo.find_by_id(&id_uuid).await.map_err(|e| {
        tracing::error!(op = "get_novel_term_by_id", id = %id_uuid, error = %e);
        format!("查询名词失败: {}", e)
    })?;
    term.ok_or_else(|| format!("查询名词失败: 名词 {} 不存在", id_uuid))
}

/// 组合查询小说名词列表
#[tauri::command]
#[allow(clippy::too_many_arguments)]
pub async fn get_novel_terms(
    novel_id: String,
    term_type: Option<String>,
    name: Option<String>,
    description_keyword: Option<String>,
    page: i64,
    page_size: i64,
    state: State<'_, AppState>,
) -> Result<PaginatedResult<NovelTerm>, String> {
    let novel_uuid = parse_uuid("novelId", &novel_id)?;
    let term_type = parse_optional_term_type(term_type)?;

    let query = NovelTermQuery {
        novel_id: novel_uuid,
        term_type,
        name,
        description_keyword,
        page,
        page_size,
    };

    let repo = state.novel_term_repo.read().await;
    repo.find_with_query(&query).await.map_err(|e| {
        tracing::error!(op = "get_novel_terms", novel_id = %novel_uuid, error = %e);
        format!("查询名词失败: {}", e)
    })
}
