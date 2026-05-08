use serde::{Deserialize, Serialize};

use super::types::ConversationType;

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
