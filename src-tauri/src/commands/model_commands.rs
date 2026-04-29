use tauri::State;
use uuid::Uuid;

use crate::ai::model_fetchers::{FetchError, ModelInfo, ProviderType};
use crate::AppState;

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

    // 获取 FetcherRegistry 并拉取模型
    let registry = state.fetcher_registry.read().await;
    registry
        .fetch(ProviderType::Default, &provider.base_url, &api_key)
        .await
        .map_err(|e| match e {
            FetchError::RequestFailed(msg) => msg,
            _ => e.to_string(),
        })
}
