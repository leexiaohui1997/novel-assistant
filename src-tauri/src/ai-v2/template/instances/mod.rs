//! 模板实例集合
//!
//! 子目录与 `templates/v2/` 物理目录层级一一对应：
//! - [`prompts`] 对应 `templates/v2/prompts/`
//!
//! 每个具体模板对应 `instances/{category}/{name}.rs` 中实现
//! [`super::TemplateInstance`] 的结构体。

pub mod prompts;
