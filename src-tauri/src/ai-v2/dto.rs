use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::types::{ConversationType, MessageType};

/// 创建会话的入参 DTO
///
/// 支持从前端或业务层传入。未指定的字段将使用默认值：
/// - `conversation_type` 默认为 `ConversationType::Default`
/// - `is_pinned` 默认为 `false`
/// - `title` / `conversation_params` / `remark` 默认为 `None`
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct CreateConversationInput {
    /// 会话标题（可选）
    pub title: Option<String>,

    /// 会话类型（默认 `Default`）
    pub conversation_type: ConversationType,

    /// 会话参数（可选 JSON 字符串）
    pub conversation_params: Option<String>,

    /// 是否置顶（默认 `false`）
    pub is_pinned: bool,

    /// 备注（可选）
    pub remark: Option<String>,
}

/// 插入会话消息的入参 DTO
///
/// 用于 `AiService::insert_conversation_message`。调用方只需提供业务语义相关字段，
/// 其余（`sequence` / 时间戳 / 供应商与模型名称快照）由 Service 自动计算并填充。
///
/// 未指定的可选字段含义：
/// - `input_tokens` / `output_tokens` 未传按 `0` 写入
/// - `model_id` 未传则不做快照，`provider_name` / `model_name` 同步置空
/// - `thinking_content` / `ext_1` / `ext_2` / `ext_3` 未传保持 `NULL`
///
/// > 注：`conversation_id`、`message_type`、`content` 属于必填字段，
/// > 前端 / 调用方若漏传将直接反序列化失败，而不是被静默置为默认值。
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InsertConversationMessageInput {
    /// 所属会话 ID（必填）
    pub conversation_id: Uuid,

    /// 消息类型（必填，参见 [`MessageType`]）
    pub message_type: MessageType,

    /// 消息内容（必填）
    pub content: String,

    /// 思考内容（可选）
    #[serde(default)]
    pub thinking_content: Option<String>,

    /// 输入 Tokens（可选，未传按 0 处理）
    #[serde(default)]
    pub input_tokens: Option<i64>,

    /// 输出 Tokens（可选，未传按 0 处理）
    #[serde(default)]
    pub output_tokens: Option<i64>,

    /// 模型 ID（可选；提供后 Service 会自动快照供应商名称与模型名称）
    #[serde(default)]
    pub model_id: Option<Uuid>,

    /// 扩展字段 1 / 2 / 3（均可选）
    #[serde(default)]
    pub ext_1: Option<String>,
    #[serde(default)]
    pub ext_2: Option<String>,
    #[serde(default)]
    pub ext_3: Option<String>,
}
