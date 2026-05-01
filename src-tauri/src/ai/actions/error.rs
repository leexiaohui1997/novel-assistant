// Action Error 模块
//
// 定义 ActionError 类型

use thiserror::Error;

/// Action 执行错误类型
#[derive(Debug, Error)]
pub enum ActionError {
    /// Action 未找到
    #[error("Action '{0}' 不存在")]
    NotFound(String),

    /// 输入参数无效
    #[error("输入参数无效: {0}")]
    InvalidInput(String),

    /// 执行失败
    #[error("执行失败: {0}")]
    ExecutionFailed(String),

    /// AI 服务调用失败
    #[error("AI 服务调用失败: {0}")]
    AiServiceError(String),
}
