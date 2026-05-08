use std::collections::HashMap;
use std::sync::Arc;

use super::traits::AiTool;

/// AI 工具注册中心
///
/// 负责管理所有可用的 AI 工具实例。
pub struct ToolRegistry {
    tools: HashMap<String, Arc<dyn AiTool>>,
}

impl ToolRegistry {
    /// 创建一个新的空注册中心
    pub fn new() -> Self {
        Self {
            tools: HashMap::new(),
        }
    }

    /// 注册一个工具
    ///
    /// # 参数
    /// - `tool`: 实现了 [`AiTool`] trait 的工具实例
    pub fn register(&mut self, tool: Arc<dyn AiTool>) {
        let name = tool.name().to_string();
        self.tools.insert(name, tool);
    }

    /// 根据名称获取工具
    ///
    /// # 参数
    /// - `name`: 工具的唯一标识符
    pub fn get_tool(&self, name: &str) -> Option<&Arc<dyn AiTool>> {
        self.tools.get(name)
    }

    /// 获取所有已注册工具的列表（用于向 AI 模型提供工具元数据）
    pub fn list_tools(&self) -> Vec<Arc<dyn AiTool>> {
        self.tools.values().cloned().collect()
    }
}
