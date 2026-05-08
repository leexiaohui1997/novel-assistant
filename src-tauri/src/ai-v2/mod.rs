//! AI 服务（v2）模块
//!
//! 提供基于 AI 会话与消息的新一代能力抽象，按职责拆分为：
//! - `types`：共享类型（如 `ConversationType`）
//! - `dto`：入参 / 出参 DTO
//! - `service`：`AiService` 及其方法
pub mod dto;
pub mod service;
pub mod types;

pub use dto::CreateConversationInput;
pub use service::AiService;
pub use types::ConversationType;
