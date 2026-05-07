use chrono::{DateTime, TimeZone, Utc};
use tauri::State;

use crate::database::models::ai_call_log::{AiCallLog, ModelUsageRow, TokensSummary};
use crate::utils::pagination::PaginatedResult;
use crate::AppState;

/// 将前端传入的毫秒时间戳转为 UTC DateTime
fn ms_to_utc(ms: i64) -> Result<DateTime<Utc>, String> {
    Utc.timestamp_millis_opt(ms)
        .single()
        .ok_or_else(|| "非法的时间戳".to_string())
}

/// 获取 Tokens 看板汇总（总请求数 / 输入 / 输出 tokens）
#[tauri::command]
pub async fn get_tokens_summary(
    state: State<'_, AppState>,
    start_ms: i64,
    end_ms: i64,
) -> Result<TokensSummary, String> {
    let start = ms_to_utc(start_ms)?;
    let end = ms_to_utc(end_ms)?;
    let repo = state.call_log_repo.read().await;
    repo.summary(start, end).await.map_err(|e| e.to_string())
}

/// 获取模型使用汇总（按 provider + model 聚合）
#[tauri::command]
pub async fn get_tokens_model_usage(
    state: State<'_, AppState>,
    start_ms: i64,
    end_ms: i64,
) -> Result<Vec<ModelUsageRow>, String> {
    let start = ms_to_utc(start_ms)?;
    let end = ms_to_utc(end_ms)?;
    let repo = state.call_log_repo.read().await;
    repo.group_by_model(start, end)
        .await
        .map_err(|e| e.to_string())
}

/// 分页获取调用记录明细（按 call_time 倒序）
#[tauri::command]
pub async fn list_ai_call_logs(
    state: State<'_, AppState>,
    start_ms: i64,
    end_ms: i64,
    page: i64,
    page_size: i64,
) -> Result<PaginatedResult<AiCallLog>, String> {
    let start = ms_to_utc(start_ms)?;
    let end = ms_to_utc(end_ms)?;
    let repo = state.call_log_repo.read().await;
    repo.list_by_range(start, end, page, page_size)
        .await
        .map_err(|e| e.to_string())
}
