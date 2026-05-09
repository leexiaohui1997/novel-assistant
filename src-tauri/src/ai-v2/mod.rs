//! AI 服务（v2）模块
//!
//! 提供基于 AI 会话与消息的新一代能力抽象，按职责拆分为：
//! - `types`：共享类型（如 `ConversationType`、`MessageType`）
//! - `dto`：入参 / 出参 DTO
//! - `service`：`AiService` 及其方法
//! - `template`：Tera 模板管理模块（加载 `templates/v2/` 下的 prompts/skills）
//! - `initializer`：会话类型初始化器（trait + 统一调度入口）
pub mod dto;
pub mod initializer;
pub mod service;
pub mod skills;
pub mod template;
pub mod tools;
pub mod types;
pub mod utils;

pub use dto::{CreateConversationInput, InsertConversationMessageInput};
pub use initializer::ConversationInitializer;
pub use service::AiService;
pub use skills::{AiSkill, SkillRegistry};
pub use template::{TemplateError, TemplateInstance, TemplateManager};
pub use tools::builtin::SearchNovelTool;
pub use tools::{AiTool, ToolRegistry};
pub use types::{ConversationStatus, ConversationType, MessageType};
