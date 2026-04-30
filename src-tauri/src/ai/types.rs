use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// 消息角色
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MessageRole {
    System,
    User,
    Assistant,
}

/// 单条消息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub role: MessageRole,
    pub content: String,
}

/// AI 请求数据
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AiRequestData {
    /// 模型 ID
    pub model_id: Uuid,

    /// 消息数组
    pub messages: Vec<Message>,
}

/// AI 聊天响应
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AiChatResponse {
    /// AI 生成的内容
    pub content: String,

    /// 原始响应对象（包含 usage 等信息）
    pub response: serde_json::Value,
}
