// Action Executor 模块
//
// 负责执行 Action

use std::sync::Arc;
use tokio::sync::RwLock;

use super::context::ActionContext;
use super::error::ActionError;
use super::router::ActionRouter;
use crate::ai::service::AiService;
use crate::database::repositories::TagRepository;

/// Action 执行器
///
/// 类比 Web 框架中的 Middleware Dispatcher
/// 负责查找并执行对应的 Action Handler
pub struct ActionExecutor {
    /// Action 路由器（线程安全）
    router: Arc<RwLock<ActionRouter>>,

    /// AI 服务引用
    ai_service: Arc<AiService>,

    /// 标签仓储引用
    tag_repo: Arc<RwLock<Box<dyn TagRepository + Send + Sync>>>,
}

impl ActionExecutor {
    /// 创建新的 ActionExecutor
    pub fn new(
        router: Arc<RwLock<ActionRouter>>,
        ai_service: Arc<AiService>,
        tag_repo: Arc<RwLock<Box<dyn TagRepository + Send + Sync>>>,
    ) -> Self {
        Self {
            router,
            ai_service,
            tag_repo,
        }
    }

    /// 执行指定的 Action
    ///
    /// 这是统一的入口函数，支持：
    /// 1. 后端内部直接调用
    /// 2. 通过 Tauri Command 由前端调用
    ///
    /// # 参数
    /// - `action_name`: Action 名称
    /// - `action_params`: Action 参数（JSON 格式）
    ///
    /// # 返回
    /// - `Ok(serde_json::Value)`: 执行结果
    /// - `Err(ActionError)`: 执行失败
    ///
    /// # 示例
    ///
    /// ```rust
    /// // 后端内部调用
    /// let result = executor.execute("text_summary", json!({ "text": "..." })).await?;
    /// ```
    pub async fn execute(
        &self,
        action_name: &str,
        action_params: serde_json::Value,
    ) -> Result<serde_json::Value, ActionError> {
        // 1. 查找 Handler
        let handler = {
            let router = self.router.read().await;
            router
                .get(action_name)
                .ok_or_else(|| ActionError::NotFound(action_name.to_string()))?
        };

        // 2. 构建 Context
        let ctx = ActionContext::new(
            action_params,
            self.ai_service.clone(),
            self.tag_repo.clone(),
        );

        // 3. 执行 Handler
        let response = handler.handle(ctx).await?;

        // 4. 返回数据
        Ok(response.data)
    }

    /// 列出所有已注册的 Action
    pub async fn list_actions(&self) -> Vec<String> {
        let router = self.router.read().await;
        router.list().into_iter().map(|s| s.to_string()).collect()
    }
}
