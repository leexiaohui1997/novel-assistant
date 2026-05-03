// Action Executor 模块
//
// 负责执行 Action

use std::sync::Arc;
use tokio::sync::RwLock;

use super::context::ActionContext;
use super::error::ActionError;
use super::router::ActionRouter;
use crate::ai::service::AiService;
use crate::database::repositories::{CharacterRepository, NovelRepository, TagRepository};

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

    /// 小说仓储引用
    novel_repo: Arc<RwLock<Box<dyn NovelRepository + Send + Sync>>>,

    /// 角色仓储引用
    character_repo: Arc<RwLock<Box<dyn CharacterRepository + Send + Sync>>>,
}

impl ActionExecutor {
    /// 创建新的 ActionExecutor
    pub fn new(
        router: Arc<RwLock<ActionRouter>>,
        ai_service: Arc<AiService>,
        tag_repo: Arc<RwLock<Box<dyn TagRepository + Send + Sync>>>,
        novel_repo: Arc<RwLock<Box<dyn NovelRepository + Send + Sync>>>,
        character_repo: Arc<RwLock<Box<dyn CharacterRepository + Send + Sync>>>,
    ) -> Self {
        Self {
            router,
            ai_service,
            tag_repo,
            novel_repo,
            character_repo,
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
    /// - `model_id`: 可选的模型 ID，用于指定使用哪个 AI 模型
    ///
    /// # 返回
    /// - `Ok(serde_json::Value)`: 执行结果
    /// - `Err(ActionError)`: 执行失败
    ///
    /// # 示例
    ///
    /// ```rust
    /// // 后端内部调用
    /// let result = executor.execute("text_summary", json!({ "text": "..." }), None).await?;
    /// ```
    pub async fn execute(
        &self,
        action_name: &str,
        action_params: serde_json::Value,
        model_id: Option<String>,
    ) -> Result<serde_json::Value, ActionError> {
        // 1. 查找 Handler
        let handler = {
            let router = self.router.read().await;
            router
                .get(action_name)
                .ok_or_else(|| ActionError::NotFound(action_name.to_string()))?
        };

        // 2. 构建 Context
        let mut ctx = ActionContext::new(
            action_params,
            self.ai_service.clone(),
            self.tag_repo.clone(),
            self.novel_repo.clone(),
            self.character_repo.clone(),
        );

        // 3. 如果提供了 model_id，添加到 context 中
        if let Some(id) = model_id {
            ctx.set_model_id(id);
        }

        // 4. 执行 Handler
        let response = handler.handle(ctx).await?;

        // 5. 返回数据
        Ok(response.data)
    }

    /// 列出所有已注册的 Action
    pub async fn list_actions(&self) -> Vec<String> {
        let router = self.router.read().await;
        router.list().into_iter().map(|s| s.to_string()).collect()
    }
}
