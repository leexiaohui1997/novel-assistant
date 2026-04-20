use tauri::State;

use crate::database::models::novel::{NewNovel, Novel};
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
#[tauri::command]
pub async fn get_novels(state: State<'_, AppState>) -> Result<Vec<Novel>, String> {
    let repo = state.novel_repo.read().await;
    repo.find_all().await.map_err(|e| e.to_string())
}
