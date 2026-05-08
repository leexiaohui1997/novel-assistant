use crate::database::models::ai_conversation::{AiConversation, CreateAiConversation};

use super::super::dto::CreateConversationInput;
use super::AiService;

impl AiService {
    /// 创建一条会话
    ///
    /// # 参数
    /// - `input`: 会话创建入参（未传字段使用 `CreateConversationInput` 的默认值）
    ///
    /// # 返回
    /// - `Ok(AiConversation)`: 新建的完整会话（含数据库生成的 `created_at` / `updated_at`）
    /// - `Err(String)`: 以中文描述的错误信息
    pub async fn create_conversation(
        &self,
        input: CreateConversationInput,
    ) -> Result<AiConversation, String> {
        let payload = CreateAiConversation {
            title: input.title,
            conversation_type: input.conversation_type.to_string(),
            conversation_params: input.conversation_params,
            is_pinned: input.is_pinned,
            remark: input.remark,
            status: input.status.to_string(),
            prompt: input.prompt,
        };

        let repo = self.conversation_repo.read().await;
        repo.create(payload)
            .await
            .map_err(|e| format!("创建会话失败: {}", e))
    }
}
