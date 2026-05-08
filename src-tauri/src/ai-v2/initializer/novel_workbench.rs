//! [`ConversationType::NovelWorkbench`] 的初始化器
//!
//! 当前为占位实现：仅返回 `Ok(())`，不执行任何业务副作用；
//! 后续需求会在此补齐"小说工作台"会话的初始化逻辑。

use async_trait::async_trait;

use crate::database::models::ai_conversation::AiConversation;

use super::super::service::AiService;
use super::super::types::ConversationType;
use super::ConversationInitializer;

/// 小说工作台会话类型的初始化器（零字段）
#[derive(Debug, Default, Clone, Copy)]
pub struct NovelWorkbenchInitializer;

#[async_trait]
impl ConversationInitializer for NovelWorkbenchInitializer {
    fn conversation_type(&self) -> ConversationType {
        ConversationType::NovelWorkbench
    }

    async fn initialize(
        &self,
        _service: &AiService,
        _conversation: &AiConversation,
    ) -> Result<(), String> {
        // 占位实现：当前无需任何初始化副作用
        Ok(())
    }
}
