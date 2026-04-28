use tauri::State;
use uuid::Uuid;

use crate::database::models::chapter::{
    Chapter, ChapterQuery, ChapterResponse, NewChapter, NewVolume, UpdateChapter, UpdateVolume,
    Volume, VolumeUpsert,
};
use crate::utils::pagination::{PaginatedResult, PaginationParams};
use crate::AppState;

/// 将数据库 `Chapter` 列表转换为接口层 `ChapterResponse` 列表。
///
/// `max_sequence` 为 `None` 时，所有章节的 `deletable` 均为 `false`。
fn build_chapter_responses(
    chapters: Vec<Chapter>,
    max_sequence: Option<i64>,
) -> Vec<ChapterResponse> {
    chapters
        .into_iter()
        .map(|c| {
            let deletable = max_sequence.is_some_and(|max| c.sequence >= 0 && c.sequence == max);
            ChapterResponse::from_chapter(c, deletable)
        })
        .collect()
}

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
///
/// 返回接口层 DTO `ChapterResponse`，在数据库字段基础上扩展 `deletable` 标识：
/// 仅当章节为所属分卷下的最后一个非草稿章节时为 `true`。
#[tauri::command]
pub async fn get_chapters_with_pagination(
    state: State<'_, AppState>,
    novel_id: String,
    page: i64,
    page_size: i64,
    query: ChapterQuery,
) -> Result<PaginatedResult<ChapterResponse>, String> {
    let novel_uuid = Uuid::parse_str(&novel_id).map_err(|e| e.to_string())?;
    let pagination = PaginationParams { page, page_size };
    let repo = state.chapter_repo.read().await;
    let page_result = repo
        .get_chapters_with_pagination(novel_uuid, &query, &pagination)
        .await
        .map_err(|e| e.to_string())?;

    // 草稿模式或未指定 volume_sequence 时无需计算 deletable
    let max_sequence = match (query.is_draft, query.volume_sequence) {
        (false, Some(vseq)) => repo
            .get_max_sequence_in_volume(novel_uuid, vseq)
            .await
            .map_err(|e| e.to_string())?,
        _ => None,
    };

    Ok(PaginatedResult {
        data: build_chapter_responses(page_result.data, max_sequence),
        total: page_result.total,
    })
}
