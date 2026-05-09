//! Tera 模板管理模块对外入口
//!
//! 按职责拆分为：
//! - `error`：`TemplateError` 错误类型
//! - `manager`：`TemplateManager` 加载与渲染实现
//! - `instance`：`TemplateInstance` trait 与默认流水线
//!
//! 仅公开三个符号：`TemplateManager`、`TemplateInstance`、`TemplateError`。

mod error;
mod instance;
mod manager;

pub mod instances;

pub use error::TemplateError;
pub use instance::TemplateInstance;
pub use manager::TemplateManager;
