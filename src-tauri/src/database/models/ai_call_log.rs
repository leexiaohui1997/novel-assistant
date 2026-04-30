use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

/// AI 调用记录实体
/// 对应数据库中的 ai_call_logs 表
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
#[serde(rename_all = "camelCase")]
pub struct AiCallLog {
    pub id: Uuid,
    pub provider_id: Uuid,
    pub model_id: Uuid,
    pub model_name: String,
    pub provider_name: String,
    pub input_tokens: i32,
    pub output_tokens: i32,
    pub total_tokens: i32,
    pub duration_ms: i64,
    pub message: String,
    pub response: Option<String>,
    pub status: String,
    pub error_message: Option<String>,
    pub call_time: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
}

/// 创建 AI 调用记录请求
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateAiCallLog {
    pub provider_id: Uuid,
    pub model_id: Uuid,
    pub model_name: String,
    pub provider_name: String,
    pub input_tokens: i32,
    pub output_tokens: i32,
    pub total_tokens: i32,
    pub duration_ms: i64,
    pub message: String,
    pub response: Option<String>,
    pub status: String,
    pub error_message: Option<String>,
}
