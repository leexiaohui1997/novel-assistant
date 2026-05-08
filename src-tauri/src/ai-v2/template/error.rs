//! 模板模块错误类型
//!
//! `TemplateError` 统一描述模板加载、上下文校验、参数校验与渲染四类失败场景，
//! 并通过 `#[source]` 链保留底层错误，便于上层使用 `tracing` 打印完整错误栈。

use std::io;

use thiserror::Error;
use validator::ValidationErrors;

/// 模板管理模块统一错误类型
#[derive(Debug, Error)]
pub enum TemplateError {
    /// 根据模板 ID 未找到对应模板
    #[error("模板未找到: {id}")]
    NotFound { id: String },

    /// 渲染上下文非法（必须为 JSON 对象 / 反序列化失败）
    #[error("模板上下文非法: {0}")]
    InvalidContext(String),

    /// 模板参数校验失败（validator 派生的约束未通过）
    #[error("模板参数校验失败")]
    Validation(#[source] ValidationErrors),

    /// Tera 模板渲染 / 加载阶段的底层错误
    #[error("模板渲染失败")]
    Render(#[source] tera::Error),

    /// 文件系统 I/O 错误（扫描模板目录时发生）
    #[error("模板 I/O 失败")]
    Io(#[source] io::Error),
}

impl From<ValidationErrors> for TemplateError {
    fn from(value: ValidationErrors) -> Self {
        TemplateError::Validation(value)
    }
}

impl From<tera::Error> for TemplateError {
    fn from(value: tera::Error) -> Self {
        TemplateError::Render(value)
    }
}

impl From<io::Error> for TemplateError {
    fn from(value: io::Error) -> Self {
        TemplateError::Io(value)
    }
}
