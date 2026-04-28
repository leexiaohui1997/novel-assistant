use tauri::State;
use uuid::Uuid;

use crate::database::models::provider::{NewProvider, Provider, UpdateProvider};
use crate::utils::pagination::{PaginatedResult, PaginationParams};
use crate::AppState;

/// 创建新供应商
#[tauri::command]
pub async fn create_provider(
    state: State<'_, AppState>,
    provider: NewProvider,
) -> Result<Provider, String> {
    let repo = state.provider_repo.read().await;
    repo.create(&provider).await.map_err(|e| e.to_string())
}

/// 分页获取供应商列表
#[tauri::command]
pub async fn get_providers_with_pagination(
    state: State<'_, AppState>,
    page: i64,
    page_size: i64,
) -> Result<PaginatedResult<Provider>, String> {
    let repo = state.provider_repo.read().await;
    let params = PaginationParams { page, page_size };
    repo.find_with_pagination(&params)
        .await
        .map_err(|e| e.to_string())
}

/// 更新供应商信息
#[tauri::command]
pub async fn update_provider(
    state: State<'_, AppState>,
    id: String,
    provider: UpdateProvider,
) -> Result<Provider, String> {
    let repo = state.provider_repo.read().await;
    let uuid = Uuid::parse_str(&id).map_err(|e| e.to_string())?;
    repo.update(uuid, &provider)
        .await
        .map_err(|e| e.to_string())
}

/// 删除供应商
#[tauri::command]
pub async fn delete_provider(state: State<'_, AppState>, id: String) -> Result<(), String> {
    let repo = state.provider_repo.read().await;
    let uuid = Uuid::parse_str(&id).map_err(|e| e.to_string())?;
    repo.delete(uuid).await.map_err(|e| e.to_string())
}
