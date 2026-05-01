// Action Handler 模块
//
// 定义 ActionHandler trait

use async_trait::async_trait;

use super::context::ActionContext;
use super::error::ActionError;

/// Action 响应结构
pub struct ActionResponse {
    /// 响应数据
    pub data: serde_json::Value,

    /// 元数据（耗时、token 使用等）
    pub metadata: std::collections::HashMap<String, serde_json::Value>,
}

/// Action 处理器 trait
///
/// 类比 Web 框架中的 Controller/Handler
/// 每个具体的 Action 需要实现这个 trait
#[async_trait]
pub trait ActionHandler: Send + Sync {
    /// Action 名称（唯一标识，类似路由路径）
    fn name(&self) -> &str;

    /// Action 描述
    fn description(&self) -> &str;

    /// 执行 Action
    ///
    /// # 参数
    /// - `ctx`: Action 执行上下文，包含输入参数和依赖服务
    ///
    /// # 返回
    /// - `Ok(ActionResponse)`: 执行成功，返回响应数据
    /// - `Err(ActionError)`: 执行失败，返回错误信息
    async fn handle(&self, ctx: ActionContext) -> Result<ActionResponse, ActionError>;
}
