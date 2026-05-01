// Action Router 模块
//
// 负责注册和管理所有 Action Handler

use std::collections::HashMap;
use std::sync::Arc;

use super::handler::ActionHandler;

/// Action 路由器
///
/// 类比 Web 框架中的 Router，负责注册和管理所有 Action Handler
pub struct ActionRouter {
    /// 已注册的 Action Handler 映射表
    handlers: HashMap<String, Arc<dyn ActionHandler>>,
}

impl ActionRouter {
    /// 创建新的 ActionRouter
    pub fn new() -> Self {
        Self {
            handlers: HashMap::new(),
        }
    }

    /// 注册一个 Action Handler
    ///
    /// 类比: router.post('/users', userController.create)
    ///
    /// # 参数
    /// - `handler`: 实现了 ActionHandler trait 的处理器
    pub fn register(&mut self, handler: Arc<dyn ActionHandler>) {
        let name = handler.name().to_string();
        self.handlers.insert(name, handler);
    }

    /// 获取指定的 Action Handler
    ///
    /// # 参数
    /// - `name`: Action 名称
    ///
    /// # 返回
    /// - `Some(Arc<dyn ActionHandler>)`: 找到对应的 Handler
    /// - `None`: 未找到
    pub fn get(&self, name: &str) -> Option<Arc<dyn ActionHandler>> {
        self.handlers.get(name).cloned()
    }

    /// 列出所有已注册的 Action 名称
    pub fn list(&self) -> Vec<&str> {
        self.handlers.keys().map(|k| k.as_str()).collect()
    }

    /// 检查 Action 是否已注册
    pub fn contains(&self, name: &str) -> bool {
        self.handlers.contains_key(name)
    }
}
