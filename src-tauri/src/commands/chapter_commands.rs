use tauri::State;
use uuid::Uuid;

use crate::database::models::chapter::{
    Chapter, ChapterQuery, NewChapter, NewVolume, UpdateChapter, UpdateVolume, Volume, VolumeUpsert,
};
use crate::utils::pagination::{PaginatedResult, PaginationParams};
use crate::AppState;

#[tauri::command]
pub async fn create_volume(
    state: State<'_, AppState>,
    payload: NewVolume,
) -> Result<Volume, String> {
    let repo = state.chapter_repo.read().await;
    repo.create_volume(&payload)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn update_volume(
    state: State<'_, AppState>,
    volume_id: i64,
    payload: UpdateVolume,
) -> Result<Volume, String> {
    let repo = state.chapter_repo.read().await;
    repo.update_volume(volume_id, &payload)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn delete_volume(
    state: State<'_, AppState>,
    novel_id: String,
    volume_id: i64,
) -> Result<(), String> {
    let uuid = Uuid::parse_str(&novel_id).map_err(|e| e.to_string())?;
    let repo = state.chapter_repo.read().await;
    repo.delete_volume(uuid, volume_id)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn create_chapter(
    state: State<'_, AppState>,
    novel_id: String,
    payload: NewChapter,
) -> Result<Chapter, String> {
    let novel_uuid = Uuid::parse_str(&novel_id).map_err(|e| e.to_string())?;
    let repo = state.chapter_repo.read().await;
    repo.create_chapter(novel_uuid, &payload)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn update_chapter(
    state: State<'_, AppState>,
    chapter_id: String,
    payload: UpdateChapter,
) -> Result<Chapter, String> {
    let uuid = Uuid::parse_str(&chapter_id).map_err(|e| e.to_string())?;
    let repo = state.chapter_repo.read().await;
    repo.update_chapter(uuid, &payload)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn delete_chapter(
    state: State<'_, AppState>,
    novel_id: String,
    chapter_id: String,
) -> Result<(), String> {
    let novel_uuid = Uuid::parse_str(&novel_id).map_err(|e| e.to_string())?;
    let chapter_uuid = Uuid::parse_str(&chapter_id).map_err(|e| e.to_string())?;
    let repo = state.chapter_repo.read().await;
    repo.delete_chapter(novel_uuid, chapter_uuid)
        .await
        .map_err(|e| e.to_string())
}

/// 查询小说下全部分卷（按 sequence 升序）。
#[tauri::command]
pub async fn get_volumes(
    state: State<'_, AppState>,
    novel_id: String,
) -> Result<Vec<Volume>, String> {
    let novel_uuid = Uuid::parse_str(&novel_id).map_err(|e| e.to_string())?;
    let repo = state.chapter_repo.read().await;
    repo.get_volumes(novel_uuid)
        .await
        .map_err(|e| e.to_string())
}

/// 批量编辑分卷（事务写入）。
///
/// 前端一次性提交最新分卷列表，后端自动判定新增 / 修改 / 删除。
#[tauri::command]
pub async fn batch_update_volumes(
    state: State<'_, AppState>,
    novel_id: String,
    payload: Vec<VolumeUpsert>,
) -> Result<Vec<Volume>, String> {
    let novel_uuid = Uuid::parse_str(&novel_id).map_err(|e| e.to_string())?;
    let repo = state.chapter_repo.read().await;
    repo.batch_update_volumes(novel_uuid, &payload)
        .await
        .map_err(|e| e.to_string())
}

/// 分页查询章节。
#[tauri::command]
pub async fn get_chapters_with_pagination(
    state: State<'_, AppState>,
    novel_id: String,
    page: i64,
    page_size: i64,
    query: ChapterQuery,
) -> Result<PaginatedResult<Chapter>, String> {
    let novel_uuid = Uuid::parse_str(&novel_id).map_err(|e| e.to_string())?;
    let pagination = PaginationParams { page, page_size };
    let repo = state.chapter_repo.read().await;
    repo.get_chapters_with_pagination(novel_uuid, &query, &pagination)
        .await
        .map_err(|e| e.to_string())
}
