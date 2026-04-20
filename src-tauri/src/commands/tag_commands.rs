use tauri::State;

use crate::AppState;

/// 获取指定目标读者的标签列表
#[tauri::command]
pub async fn get_tags_by_audience(
    target_audience: String,
    state: State<'_, AppState>,
) -> Result<Vec<crate::database::models::tag::Tag>, String> {
    // 验证参数
    if !["male", "female"].contains(&target_audience.as_str()) {
        return Err("无效的目标读者类型，必须是 male 或 female".to_string());
    }

    let tag_repo = state.tag_repo.read().await;
    tag_repo
        .find_by_target_audience(&target_audience)
        .await
        .map_err(|e| format!("获取标签失败: {}", e))
}

/// 根据 ID 列表获取标签
#[tauri::command]
pub async fn get_tags_by_ids(
    ids: Vec<i64>,
    state: State<'_, AppState>,
) -> Result<Vec<crate::database::models::tag::Tag>, String> {
    let tag_repo = state.tag_repo.read().await;
    tag_repo
        .find_by_ids(&ids)
        .await
        .map_err(|e| format!("获取标签失败: {}", e))
}
