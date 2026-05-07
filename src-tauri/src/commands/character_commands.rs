use tauri::State;
use uuid::Uuid;

use crate::database::models::character::{Character, CharacterType, Gender};
use crate::utils::pagination::{PaginatedResult, PaginationParams};
use crate::AppState;

/// 将字符串解析为性别枚举
fn parse_gender(gender: &str) -> Result<Gender, String> {
    match gender {
        "male" => Ok(Gender::Male),
        "female" => Ok(Gender::Female),
        "other" => Ok(Gender::Other),
        "unknown" => Ok(Gender::Unknown),
        _ => Err("无效的性别类型，必须是 male、female、other 或 unknown".to_string()),
    }
}

/// 将可选字符串解析为角色类型枚举（None 或空串视为未设置）
fn parse_character_type(value: Option<String>) -> Result<Option<CharacterType>, String> {
    let Some(raw) = value else { return Ok(None) };
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return Ok(None);
    }
    match trimmed {
        "protagonist" => Ok(Some(CharacterType::Protagonist)),
        "second_protagonist" => Ok(Some(CharacterType::SecondProtagonist)),
        "third_protagonist" => Ok(Some(CharacterType::ThirdProtagonist)),
        "supporting" => Ok(Some(CharacterType::Supporting)),
        "minor_supporting" => Ok(Some(CharacterType::MinorSupporting)),
        _ => Err(
            "无效的角色类型，必须是 protagonist、second_protagonist、third_protagonist、supporting 或 minor_supporting"
                .to_string(),
        ),
    }
}

/// 创建角色
#[tauri::command]
pub async fn create_character(
    novel_id: String,
    name: String,
    gender: String,
    character_type: Option<String>,
    background: Option<String>,
    appearance: Option<String>,
    personality: Option<String>,
    additional_info: Option<String>,
    state: State<'_, AppState>,
) -> Result<Character, String> {
    let gender = parse_gender(&gender)?;
    let character_type = parse_character_type(character_type)?;

    let character = Character {
        id: Uuid::new_v4(),
        novel_id: Uuid::parse_str(&novel_id).map_err(|e| e.to_string())?,
        name,
        gender,
        character_type,
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
    character_type: Option<String>,
    background: Option<String>,
    appearance: Option<String>,
    personality: Option<String>,
    additional_info: Option<String>,
    state: State<'_, AppState>,
) -> Result<Character, String> {
    let gender = parse_gender(&gender)?;
    let character_type = parse_character_type(character_type)?;

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
        character_type,
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
