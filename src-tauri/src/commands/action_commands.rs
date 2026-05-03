// Action 相关的 Tauri Commands

use tauri::State;

use crate::AppState;

/// 执行指定的 Action
///
/// # 参数
/// - `action_name`: Action 名称
/// - `action_params`: Action 参数（JSON 格式）
/// - `model_id`: 可选的模型 ID，用于指定使用哪个 AI 模型
///
/// # 返回
/// - `Ok(serde_json::Value)`: 执行结果
/// - `Err(String)`: 错误信息
#[tauri::command]
pub async fn execute_action(
    state: State<'_, AppState>,
    action_name: String,
    action_params: serde_json::Value,
    model_id: Option<String>,
) -> Result<serde_json::Value, String> {
    state
        .action_executor
        .execute(&action_name, action_params, model_id)
        .await
        .map_err(|e| format!("Action 执行失败: {}", e))
}

/// 列出所有已注册的 Action
///
/// # 返回
/// - `Ok(Vec<String>)`: Action 名称列表
#[tauri::command]
pub async fn list_actions(state: State<'_, AppState>) -> Result<Vec<String>, String> {
    Ok(state.action_executor.list_actions().await)
}
