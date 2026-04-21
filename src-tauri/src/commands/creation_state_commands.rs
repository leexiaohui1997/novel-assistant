use tauri::State;
use uuid::Uuid;

use crate::AppState;

/// 获取指定小说的创作状态记录
///
/// 从数据库中读取指定小说的状态记录，如果不存在则返回 None
#[tauri::command]
pub async fn get_creation_state(
    state: State<'_, AppState>,
    novel_id: String,
) -> Result<Option<serde_json::Value>, String> {
    let uuid = Uuid::parse_str(&novel_id).map_err(|e| e.to_string())?;
    let repo = state.creation_state_repo.read().await;

    match repo.get_state(&uuid).await.map_err(|e| e.to_string())? {
        Some(record) => {
            // 将记录转换为 JSON 对象，但不包含 id 和时间戳
            Ok(Some(serde_json::json!({
                "novelId": record.novel_id,
                "createdAt": record.created_at,
                "updatedAt": record.updated_at
            })))
        }
        None => Ok(None),
    }
}

/// 创建或更新指定小说的创作状态记录
///
/// 如果记录不存在则创建，存在则更新 updated_at 时间戳
#[tauri::command]
pub async fn upsert_creation_state(
    state: State<'_, AppState>,
    novel_id: String,
) -> Result<(), String> {
    let uuid = Uuid::parse_str(&novel_id).map_err(|e| e.to_string())?;
    let repo = state.creation_state_repo.read().await;

    repo.upsert_state(&uuid).await.map_err(|e| e.to_string())?;
    Ok(())
}
