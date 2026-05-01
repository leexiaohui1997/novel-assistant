pub mod builtin;
pub mod context;
pub mod error;
pub mod executor;
pub mod handler;
/// AI Actions 模块
///
/// 提供可扩展的 AI 动作系统，支持注册、管理和执行各种 AI 驱动的动作。
///
/// # 架构设计
///
/// 采用路由-控制器模式：
/// - **ActionHandler**: 类似 Controller，定义具体的动作逻辑
/// - **ActionRouter**: 类似 Router，注册和管理所有 Action
/// - **ActionExecutor**: 类似 Dispatcher，负责查找并执行对应的 Action
///
/// # 使用示例
///
/// ```rust
/// // 后端内部调用
/// let result = execute_action("text_summary", json!({ "text": "..." })).await?;
///
/// // 前端通过 Tauri Command 调用
/// const result = await invoke('execute_action', {
///   actionName: 'text_summary',
///   actionParams: { text: '...' }
/// });
/// ```
pub mod router;

// 重新导出常用类型，方便外部使用
pub use context::ActionContext;
pub use error::ActionError;
pub use executor::ActionExecutor;
pub use handler::{ActionHandler, ActionResponse};
pub use router::ActionRouter;
