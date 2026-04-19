use sqlx::SqlitePool;
use tauri::State;

use crate::database::models::novel::{NewNovel, Novel};
use crate::database::repositories::novel_repo::{NovelRepository, SqliteNovelRepository};

/// 创建新小说
///
/// 接收前端提交的小说信息及标签 ID 列表，调用仓储层完成持久化。
#[tauri::command]
pub async fn create_novel(pool: State<'_, SqlitePool>, novel: NewNovel) -> Result<Novel, String> {
    let repo = SqliteNovelRepository::new(pool.inner().clone());
    repo.create(&novel).await.map_err(|e| e.to_string())
}

/// 获取所有小说列表
///
/// 返回按创建时间倒序排列的所有小说，用于作品管理页面展示。
#[tauri::command]
pub async fn get_novels(pool: State<'_, SqlitePool>) -> Result<Vec<Novel>, String> {
    let repo = SqliteNovelRepository::new(pool.inner().clone());
    repo.find_all().await.map_err(|e| e.to_string())
}
