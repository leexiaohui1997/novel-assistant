use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

/// AI 会话实体
///
/// 对应数据库中的 `ai_conversations` 表，表示一次与 AI 的多轮对话上下文。
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
#[serde(rename_all = "camelCase")]
pub struct AiConversation {
    /// 会话 ID（UUID）
    pub id: Uuid,

    /// 会话标题（允许为空，由业务层根据首条消息自动生成或用户手动设置）
    pub title: Option<String>,

    /// 会话类型（业务语义字符串，不做数据库枚举约束）
    ///
    /// 对应枚举定义见 [`crate::ai_v2::ConversationType`]
    pub conversation_type: String,

    /// 会话参数（TEXT，JSON 字符串，业务层自行编解码；允许为空）
    pub conversation_params: Option<String>,

    /// 是否置顶
    pub is_pinned: bool,

    /// 备注（允许为空）
    pub remark: Option<String>,

    /// 会话状态（业务语义字符串，不做数据库枚举约束）
    ///
    /// 对应枚举定义见 [`crate::ai_v2::ConversationStatus`]，默认 `"Init"`
    pub status: String,

    /// 会话提示（TEXT，会话级别的非结构化提示文本，允许为空）
    pub prompt: Option<String>,

    /// 创建时间
    pub created_at: DateTime<Utc>,

    /// 更新时间
    pub updated_at: DateTime<Utc>,

    /// 最后一条消息的接收时间（允许为空）
    pub last_message_at: Option<DateTime<Utc>>,
}

/// 创建 AI 会话的入参 DTO
///
/// `id` 由 Repository 内部生成，时间字段由数据库默认值填充，因此不在此 DTO 中。
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateAiConversation {
    /// 会话标题（可选）
    pub title: Option<String>,

    /// 会话类型字符串（如 `"Default"`），由 Service 层从枚举转换而来
    pub conversation_type: String,

    /// 会话参数（可选 JSON 字符串）
    pub conversation_params: Option<String>,

    /// 是否置顶
    pub is_pinned: bool,

    /// 备注（可选）
    pub remark: Option<String>,

    /// 会话状态字符串（如 `"Init"`），由 Service 层从枚举转换而来
    pub status: String,

    /// 会话提示（可选，非结构化文本）
    pub prompt: Option<String>,
}
