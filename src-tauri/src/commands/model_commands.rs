use tauri::State;
use uuid::Uuid;

use crate::ai::model_fetchers::{FetchError, ModelInfo, ProviderType, ProviderTypeInfo};
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
