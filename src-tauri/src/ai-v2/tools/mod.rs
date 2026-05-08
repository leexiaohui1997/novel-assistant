//! AI 工具管理模块
//!
//! 提供 AI 工具的抽象接口与注册中心。

pub mod builtin;
mod context;
mod registry;
mod traits;

pub use context::ToolContext;
pub use registry::ToolRegistry;
pub use traits::AiTool;
