use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

/// AI 会话消息实体
///
/// 对应数据库中的 `ai_conversation_messages` 表，表示一条消息（系统提示 / 用户发言 / AI 回复）。
///
/// 供应商名称与模型名称采用「写时快照」策略：写入时从关联的 provider / model 读取一次并保存，
/// 后续即使供应商改名或模型被软删，历史消息仍保留当时的名称，避免历史记录被反向篡改。
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
#[serde(rename_all = "camelCase")]
pub struct AiConversationMessage {
    /// 消息 ID（UUID）
    pub id: Uuid,

    /// 所属会话 ID（外键 → `ai_conversations.id`，级联删除）
    pub conversation_id: Uuid,

    /// 消息类型（业务语义字符串，对应 [`crate::ai_v2::MessageType`]）
    pub message_type: String,

    /// 消息内容
    pub content: String,

    /// 思考内容（允许为空，通常用于保存模型的思维链 / reasoning 文本）
    pub thinking_content: Option<String>,

    /// 输入 Tokens（默认 0）
    pub input_tokens: i64,

    /// 输出 Tokens（默认 0）
    pub output_tokens: i64,

    /// 消息在会话内的序号（从 1 开始，按写入顺序递增）
    pub sequence: i64,

    /// 模型 ID（允许为空；不做联动删除）
    pub model_id: Option<Uuid>,

    /// 供应商名称快照（写时拷贝，允许为空）
    pub provider_name: Option<String>,

    /// 模型名称快照（写时拷贝，允许为空）
    pub model_name: Option<String>,

    /// 扩展字段 1（允许为空）
    pub ext_1: Option<String>,

    /// 扩展字段 2（允许为空）
    pub ext_2: Option<String>,

    /// 扩展字段 3（允许为空）
    pub ext_3: Option<String>,

    /// 创建时间
    pub created_at: DateTime<Utc>,

    /// 更新时间
    pub updated_at: DateTime<Utc>,
}

/// 创建 AI 会话消息的 Repository 层入参
///
/// - `id` 由 Service 层生成后传入（便于上层在写入前就拿到 ID）
/// - `sequence` 由 Service 层通过 `count_by_conversation + 1` 计算后传入
/// - `created_at` / `updated_at` 由数据库默认值填充，因此不在此 DTO 中
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateAiConversationMessage {
    /// 消息 ID（UUID），由上层生成
    pub id: Uuid,

    /// 所属会话 ID
    pub conversation_id: Uuid,

    /// 消息类型字符串（由 Service 层从 `MessageType` 枚举转换而来）
    pub message_type: String,

    /// 消息内容
    pub content: String,

    /// 思考内容（可选）
    pub thinking_content: Option<String>,

    /// 输入 Tokens
    pub input_tokens: i64,

    /// 输出 Tokens
    pub output_tokens: i64,

    /// 消息序号
    pub sequence: i64,

    /// 模型 ID（可选）
    pub model_id: Option<Uuid>,

    /// 供应商名称快照（可选）
    pub provider_name: Option<String>,

    /// 模型名称快照（可选）
    pub model_name: Option<String>,

    /// 扩展字段 1 / 2 / 3（均可选）
    pub ext_1: Option<String>,
    pub ext_2: Option<String>,
    pub ext_3: Option<String>,
}
