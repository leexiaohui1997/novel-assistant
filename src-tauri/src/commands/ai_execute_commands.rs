//! `execute_ai` Tauri 命令
//!
//! 事件驱动式的统一 AI 执行入口：前端通过 `invoke('execute_ai', payload)` 触发；
//! 命令在主线程同步完成"准备阶段"（推荐模型解析 / 会话定位或创建 / 用户消息落库），
//! 然后组装事件名 `ai-service/{conversationId}/{msTimestamp}` 同步返回给前端。
//!
//! 随后 `run_prepared_ai_response` 在 `tokio::spawn` 中异步执行：
//! - 成功：派发 `{ status: "ok", data: AiConversationMessage }` 事件
//! - 失败：派发 `{ status: "error", message: String }` 事件

use chrono::Utc;
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter, State};
use uuid::Uuid;

use crate::ai_v2::service::multi_turn_driver::{
    prepare_ai_response, run_prepared_ai_response, OnErrorCallback, OnSuccessCallback,
    PrepareAiParams, PreparedAiContext,
};
use crate::ai_v2::types::ConversationType;
use crate::database::models::ai_conversation_message::AiConversationMessage;
use crate::AppState;

/// `execute_ai` 命令请求载荷（驼峰形式）
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExecuteAiPayload {
    /// 用户提示词（必填）
    pub user_prompt: String,
    /// 会话 ID（可选；`None` 时新建会话）
    pub conversation_id: Option<Uuid>,
    /// 会话类型（可选；仅在新建会话时生效，默认 `Default`）
    pub conversation_type: Option<ConversationType>,
    /// 会话标题（可选；仅在新建会话时生效，默认 `None`）
    pub title: Option<String>,
    /// 模型 ID（可选；未传走推荐模型）
    pub model_id: Option<Uuid>,
}

/// 成功事件的 payload（前端监听使用）
#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
struct SuccessEventPayload {
    /// 固定字面量 `"ok"`
    status: &'static str,
    /// 最后一轮 assistant 消息实体（沿用其 serde 驼峰序列化）
    data: AiConversationMessage,
}

/// 失败事件的 payload
#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
struct ErrorEventPayload {
    /// 固定字面量 `"error"`
    status: &'static str,
    /// 中文错误描述
    message: String,
}

/// `execute_ai` 命令主入口
///
/// - 主线程同步完成参数校验 + 准备阶段 + 事件名组装
/// - 成功时以 `tokio::spawn` 启动后台多轮循环并立即返回事件名
/// - 失败时直接返回 `Err(String)`，不派发任何事件
#[tauri::command]
pub async fn execute_ai(
    app: AppHandle,
    state: State<'_, AppState>,
    payload: ExecuteAiPayload,
) -> Result<String, String> {
    if payload.user_prompt.trim().is_empty() {
        return Err("用户提示词不能为空".to_string());
    }

    let ms_timestamp = Utc::now().timestamp_millis();

    let (conversation_id, ctx) = prepare_ai_response(
        state.ai_service.clone(),
        state.call_log_repo.clone(),
        payload.into(),
    )
    .await?;

    let event_name = build_event_name(conversation_id, ms_timestamp);

    spawn_background_run(app, event_name.clone(), ctx);

    Ok(event_name)
}

/// 组装事件名 `ai-service/{conversationId}/{msTimestamp}`
///
/// - `conversation_id`：使用标准 UUID v4 的连字符小写形式
/// - `ms_timestamp`：13 位毫秒时间戳
fn build_event_name(conversation_id: Uuid, ms_timestamp: i64) -> String {
    format!("ai-service/{}/{}", conversation_id, ms_timestamp)
}

/// 后台驱动执行阶段：在 `tokio::spawn` 中跑 `run_prepared_ai_response`，
/// 并通过 `on_success` / `on_error` 回调派发事件。
fn spawn_background_run(app: AppHandle, event_name: String, ctx: PreparedAiContext) {
    let on_success = build_on_success(app.clone(), event_name.clone());
    let on_error = build_on_error(app, event_name);

    tokio::spawn(async move {
        run_prepared_ai_response(ctx, Some(on_success), Some(on_error)).await;
    });
}

/// 构造成功回调：派发 `{ status: "ok", data }`
fn build_on_success(app: AppHandle, event_name: String) -> OnSuccessCallback {
    Box::new(move |message| {
        let payload = SuccessEventPayload {
            status: "ok",
            data: message,
        };
        emit_event(&app, &event_name, &payload, "成功");
    })
}

/// 构造错误回调：派发 `{ status: "error", message }`
fn build_on_error(app: AppHandle, event_name: String) -> OnErrorCallback {
    Box::new(move |message| {
        let payload = ErrorEventPayload {
            status: "error",
            message,
        };
        emit_event(&app, &event_name, &payload, "失败");
    })
}

/// 统一的事件派发辅助：成功 / 失败 payload 共享日志口径
fn emit_event<P: Serialize + Clone>(app: &AppHandle, event_name: &str, payload: &P, kind: &str) {
    match app.emit(event_name, payload.clone()) {
        Ok(()) => tracing::info!("execute_ai 派发{}事件: {}", kind, event_name),
        Err(e) => tracing::warn!(
            "execute_ai 派发{}事件失败: event={}, error={}",
            kind,
            event_name,
            e
        ),
    }
}

impl From<ExecuteAiPayload> for PrepareAiParams {
    fn from(p: ExecuteAiPayload) -> Self {
        PrepareAiParams {
            user_prompt: p.user_prompt,
            conversation_id: p.conversation_id,
            conversation_type: p.conversation_type,
            title: p.title,
            model_id: p.model_id,
        }
    }
}
