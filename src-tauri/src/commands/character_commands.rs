use tauri::State;
use uuid::Uuid;

use crate::database::models::character::{Character, Gender};
use crate::utils::pagination::{PaginatedResult, PaginationParams};
use crate::AppState;

/// 创建角色
#[tauri::command]
pub async fn create_character(
    novel_id: String,
    name: String,
    gender: String,
    background: Option<String>,
    appearance: Option<String>,
    personality: Option<String>,
    additional_info: Option<String>,
    state: State<'_, AppState>,
) -> Result<Character, String> {
    // 验证性别参数
    let gender = match gender.as_str() {
        "male" => Gender::Male,
        "female" => Gender::Female,
        "other" => Gender::Other,
        "unknown" => Gender::Unknown,
        _ => return Err("无效的性别类型，必须是 male、female、other 或 unknown".to_string()),
    };

    let character = Character {
        id: Uuid::new_v4(),
        novel_id: Uuid::parse_str(&novel_id).map_err(|e| e.to_string())?,
        name,
        gender,
        background,
        appearance,
        personality,
        additional_info,
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    };

    let character_repo = state.character_repo.read().await;
    character_repo
        .create_character(&character)
        .await
        .map_err(|e| format!("创建角色失败: {}", e))
}

/// 根据小说 ID 获取所有角色
#[tauri::command]
pub async fn get_characters_by_novel(
    novel_id: String,
    state: State<'_, AppState>,
) -> Result<Vec<Character>, String> {
    let novel_uuid = Uuid::parse_str(&novel_id).map_err(|e| e.to_string())?;
    let character_repo = state.character_repo.read().await;
    character_repo
        .find_by_novel_id(&novel_uuid)
        .await
        .map_err(|e| format!("获取角色列表失败: {}", e))
}

/// 分页查询角色
#[tauri::command]
pub async fn get_characters_with_pagination(
    page: i64,
    page_size: i64,
    novel_id: Option<String>,
    state: State<'_, AppState>,
) -> Result<PaginatedResult<Character>, String> {
    let params = PaginationParams { page, page_size };
    let novel_uuid = novel_id
        .map(|id| Uuid::parse_str(&id).map_err(|e| e.to_string()))
        .transpose()?;

    let character_repo = state.character_repo.read().await;
    character_repo
        .find_with_pagination(&params, novel_uuid.as_ref())
        .await
        .map_err(|e| format!("获取角色列表失败: {}", e))
}

/// 根据 ID 获取角色
#[tauri::command]
pub async fn get_character_by_id(
    id: String,
    state: State<'_, AppState>,
) -> Result<Character, String> {
    let id_uuid = Uuid::parse_str(&id).map_err(|e| e.to_string())?;
    let character_repo = state.character_repo.read().await;
    character_repo
        .find_by_id(&id_uuid)
        .await
        .map_err(|e| format!("获取角色失败: {}", e))?
        .ok_or_else(|| format!("角色 {} 不存在", id))
}

/// 更新角色
#[tauri::command]
pub async fn update_character(
    id: String,
    name: String,
    gender: String,
    background: Option<String>,
    appearance: Option<String>,
    personality: Option<String>,
    additional_info: Option<String>,
    state: State<'_, AppState>,
) -> Result<Character, String> {
    // 验证性别参数
    let gender = match gender.as_str() {
        "male" => Gender::Male,
        "female" => Gender::Female,
        "other" => Gender::Other,
        "unknown" => Gender::Unknown,
        _ => return Err("无效的性别类型，必须是 male、female、other 或 unknown".to_string()),
    };

    // 先获取现有角色以保留 novel_id
    let character_repo = state.character_repo.read().await;
    let id_uuid = Uuid::parse_str(&id).map_err(|e| e.to_string())?;
    let existing_character = character_repo
        .find_by_id(&id_uuid)
        .await
        .map_err(|e| format!("获取角色失败: {}", e))?
        .ok_or_else(|| format!("角色 {} 不存在", id))?;

    let character = Character {
        id: id_uuid,
        novel_id: existing_character.novel_id,
        name,
        gender,
        background,
        appearance,
        personality,
        additional_info,
        created_at: existing_character.created_at,
        updated_at: chrono::Utc::now(),
    };

    character_repo
        .update_character(&character)
        .await
        .map_err(|e| format!("更新角色失败: {}", e))
}

/// 删除角色
#[tauri::command]
pub async fn delete_character(id: String, state: State<'_, AppState>) -> Result<(), String> {
    let id_uuid = Uuid::parse_str(&id).map_err(|e| e.to_string())?;
    let character_repo = state.character_repo.read().await;
    character_repo
        .delete_character(&id_uuid)
        .await
        .map_err(|e| format!("删除角色失败: {}", e))
}
