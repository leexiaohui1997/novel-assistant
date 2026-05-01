// Action Context 模块
//
// 定义 ActionContext

use std::collections::HashMap;
use std::sync::Arc;

use crate::ai::service::AiService;

/// Action 执行上下文
///
/// 封装执行 Action 所需的所有信息和依赖
pub struct ActionContext {
    /// 输入参数（JSON 格式，灵活支持各种 Action）
    pub input: serde_json::Value,

    /// AI 服务引用（用于调用 LLM）
    pub ai_service: Arc<AiService>,

    /// 扩展元数据（未来可添加用户信息、会话 ID 等）
    pub metadata: HashMap<String, serde_json::Value>,
}

impl ActionContext {
    /// 创建新的 ActionContext
    pub fn new(input: serde_json::Value, ai_service: Arc<AiService>) -> Self {
        Self {
            input,
            ai_service,
            metadata: HashMap::new(),
        }
    }
}
