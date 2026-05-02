use tauri::State;
use uuid::Uuid;

use crate::database::models::novel::{NewNovel, Novel, NovelWithTags, UpdateNovel};
use crate::database::repositories::QueryOptions;
use crate::utils::pagination::{PaginatedResult, PaginationParams};
use crate::AppState;

/// 创建新小说
///
/// 接收前端提交的小说信息及标签 ID 列表，调用仓储层完成持久化。
#[tauri::command]
pub async fn create_novel(state: State<'_, AppState>, novel: NewNovel) -> Result<Novel, String> {
    let repo = state.novel_repo.read().await;
    repo.create(&novel).await.map_err(|e| e.to_string())
}

/// 获取所有小说列表
///
/// 返回按创建时间倒序排列的所有小说，用于作品管理页面展示。
/// with_tags 为 true 时额外返回关联的标签信息。
#[tauri::command]
pub async fn get_novels(
    state: State<'_, AppState>,
    with_tags: Option<bool>,
) -> Result<Vec<NovelWithTags>, String> {
    let repo = state.novel_repo.read().await;
    let options = QueryOptions {
        with_tags: with_tags.unwrap_or(false),
    };
    repo.find_all(&options).await.map_err(|e| e.to_string())
}

/// 分页获取小说列表
///
/// 支持分页查询，用于作品管理页面的列表展示。
/// with_tags 为 true 时额外返回关联的标签信息。
#[tauri::command]
pub async fn get_novels_with_pagination(
    state: State<'_, AppState>,
    page: i64,
    page_size: i64,
    with_tags: Option<bool>,
) -> Result<PaginatedResult<NovelWithTags>, String> {
    let repo = state.novel_repo.read().await;
    let params = PaginationParams { page, page_size };
    let options = QueryOptions {
        with_tags: with_tags.unwrap_or(false),
    };
    repo.find_with_pagination(&params, &options)
        .await
        .map_err(|e| e.to_string())
}

/// 根据 ID 获取小说信息
///
/// 根据前端传入的小说 ID 返回对应的小说详情。
/// with_tags 为 true 时额外返回关联的标签信息。
#[tauri::command]
pub async fn get_novel_by_id(
    state: State<'_, AppState>,
    id: String,
    with_tags: Option<bool>,
) -> Result<NovelWithTags, String> {
    let repo = state.novel_repo.read().await;
    let uuid = Uuid::parse_str(&id).map_err(|e| e.to_string())?;
    let options = QueryOptions {
        with_tags: with_tags.unwrap_or(false),
    };
    repo.find_by_id(uuid, &options)
        .await
        .map_err(|e| e.to_string())
}

/// 更新小说信息
///
/// 接收前端提交的小说修改信息及标签 ID 列表，调用仓储层完成更新。
#[tauri::command]
pub async fn update_novel(
    state: State<'_, AppState>,
    id: String,
    novel: UpdateNovel,
) -> Result<Novel, String> {
    let repo = state.novel_repo.read().await;
    let uuid = Uuid::parse_str(&id).map_err(|e| e.to_string())?;
    repo.update(uuid, &novel).await.map_err(|e| e.to_string())
}

/// 删除小说
///
/// 根据小说ID删除对应的小说及其所有关联数据（章节、分卷、历史版本、标签关联等）
#[tauri::command]
pub async fn delete_novel(state: State<'_, AppState>, id: String) -> Result<(), String> {
    let repo = state.novel_repo.read().await;
    let uuid = Uuid::parse_str(&id).map_err(|e| e.to_string())?;
    repo.delete(uuid).await.map_err(|e| e.to_string())
}
