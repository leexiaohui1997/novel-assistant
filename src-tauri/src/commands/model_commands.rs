use tauri::State;
use uuid::Uuid;

use crate::ai::model_fetchers::{FetchError, ModelInfo, ProviderType, ProviderTypeInfo};
use crate::database::models::model::{BatchAddModels, Model, ModelWithProvider};
use crate::utils::pagination::{PaginatedResult, PaginationParams};
use crate::AppState;

/// 获取当前支持的供应商拉取类型列表
#[tauri::command]
pub async fn get_provider_types() -> Result<Vec<ProviderTypeInfo>, String> {
    Ok(ProviderType::all()
        .iter()
        .map(|pt| ProviderTypeInfo {
            id: pt.to_string(),
            name: pt.display_name().to_string(),
        })
        .collect())
}

/// 从供应商拉取可用模型列表
#[tauri::command]
pub async fn fetch_provider_models(
    state: State<'_, AppState>,
    provider_id: String,
) -> Result<Vec<ModelInfo>, String> {
    // 根据供应商 ID 查询数据库获取供应商信息
    let provider_repo = state.provider_repo.read().await;
    let uuid = Uuid::parse_str(&provider_id).map_err(|e| e.to_string())?;
    let provider = provider_repo
        .find_by_id(uuid)
        .await
        .map_err(|e| e.to_string())?;

    // 检查 api_key 是否为空
    let api_key = provider
        .api_key
        .ok_or_else(|| "供应商缺少 API Key".to_string())?;

    // 从供应商的 model_fetch_type 字段解析拉取策略类型
    let provider_type: ProviderType = provider.model_fetch_type.parse().map_err(|e: String| e)?;

    // 获取 FetcherRegistry 并拉取模型
    let registry = state.fetcher_registry.read().await;
    registry
        .fetch(provider_type, &provider.base_url, &api_key)
        .await
        .map_err(|e| match e {
            FetchError::RequestFailed(msg) => msg,
            _ => e.to_string(),
        })
}

/// 批量添加模型到数据库
#[tauri::command]
pub async fn add_models(
    state: State<'_, AppState>,
    payload: BatchAddModels,
) -> Result<Vec<Model>, String> {
    let repo = state.model_repo.read().await;
    repo.batch_create(payload.provider_id, &payload.models)
        .await
        .map_err(|e| e.to_string())
}

/// 获取所有模型（不分页，带供应商名称）
///
/// # 参数
/// - `enabled_only`: 如果为 Some(true)，只返回启用的模型；如果为 Some(false)，只返回禁用的模型；如果为 None，返回所有模型
#[tauri::command]
pub async fn get_all_models(
    state: State<'_, AppState>,
    enabled_only: Option<bool>,
) -> Result<Vec<ModelWithProvider>, String> {
    let repo = state.model_repo.read().await;
    repo.find_all_with_provider(enabled_only)
        .await
        .map_err(|e| e.to_string())
}

/// 分页获取模型列表（带供应商名称）
#[tauri::command]
pub async fn get_models_with_pagination(
    state: State<'_, AppState>,
    page: i64,
    page_size: i64,
) -> Result<PaginatedResult<ModelWithProvider>, String> {
    let repo = state.model_repo.read().await;
    let params = PaginationParams { page, page_size };
    repo.find_with_pagination(&params)
        .await
        .map_err(|e| e.to_string())
}

/// 删除模型
#[tauri::command]
pub async fn delete_model(state: State<'_, AppState>, id: String) -> Result<(), String> {
    let repo = state.model_repo.read().await;
    let uuid = Uuid::parse_str(&id).map_err(|e| e.to_string())?;
    repo.delete(uuid).await.map_err(|e| e.to_string())
}

/// 切换模型启用状态
#[tauri::command]
pub async fn toggle_model_enabled(
    state: State<'_, AppState>,
    id: String,
    is_enabled: bool,
) -> Result<Model, String> {
    let repo = state.model_repo.read().await;
    let uuid = Uuid::parse_str(&id).map_err(|e| e.to_string())?;
    repo.toggle_enabled(uuid, is_enabled)
        .await
        .map_err(|e| e.to_string())
}

/// 更新模型别名
#[tauri::command]
pub async fn update_model_alias(
    state: State<'_, AppState>,
    id: String,
    alias: String,
) -> Result<Model, String> {
    let repo = state.model_repo.read().await;
    let uuid = Uuid::parse_str(&id).map_err(|e| e.to_string())?;
    repo.update_alias(uuid, &alias)
        .await
        .map_err(|e| e.to_string())
}
