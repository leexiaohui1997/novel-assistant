use tauri::State;

use crate::database::models::character::{Character, Gender};
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
        id: String::new(), // ID 将在 repository 中生成
        novel_id,
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
    let character_repo = state.character_repo.read().await;
    character_repo
        .find_by_novel_id(&novel_id)
        .await
        .map_err(|e| format!("获取角色列表失败: {}", e))
}

/// 根据 ID 获取角色
#[tauri::command]
pub async fn get_character_by_id(
    id: String,
    state: State<'_, AppState>,
) -> Result<Character, String> {
    let character_repo = state.character_repo.read().await;
    character_repo
        .find_by_id(&id)
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
    let existing_character = character_repo
        .find_by_id(&id)
        .await
        .map_err(|e| format!("获取角色失败: {}", e))?
        .ok_or_else(|| format!("角色 {} 不存在", id))?;

    let character = Character {
        id,
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
    let character_repo = state.character_repo.read().await;
    character_repo
        .delete_character(&id)
        .await
        .map_err(|e| format!("删除角色失败: {}", e))
}
