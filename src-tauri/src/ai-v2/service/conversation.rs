use crate::database::models::ai_conversation::{AiConversation, CreateAiConversation};

use super::super::dto::CreateConversationInput;
use super::super::initializer::run_conversation_initializer;
use super::super::types::ConversationStatus;
use super::AiService;

impl AiService {
    /// 创建一条会话
    ///
    /// 流程：
    /// 1. 根据 `input` 构建 [`CreateAiConversation`] 并落库（初始状态为 `Init`）；
    /// 2. 调用 [`run_conversation_initializer`] 执行对应 [`crate::ai_v2::ConversationType`]
    ///    的初始化逻辑；
    /// 3. 初始化成功 → 把返回实体的 `status` 同步修正为 `Ready`；
    ///    初始化失败 → 会话记录保留，状态已被置为 `Error`，错误以中文描述冒泡返回。
    ///
    /// # 参数
    /// - `input`: 会话创建入参（未传字段使用 `CreateConversationInput` 的默认值）
    ///
    /// # 返回
    /// - `Ok(AiConversation)`: 新建的完整会话（状态为 `Ready`，含数据库生成的时间字段）
    /// - `Err(String)`: 以中文描述的错误信息（落库失败 / 初始化失败）
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

        let mut conversation = {
            let repo = self.conversation_repo.read().await;
            repo.create(payload)
                .await
                .map_err(|e| format!("创建会话失败: {}", e))?
        };

        run_conversation_initializer(self, &conversation)
            .await
            .map_err(|e| format!("会话初始化失败: {}", e))?;

        // 初始化成功：本地对象的 status 同步为 Ready，避免调用方再读一次 DB
        conversation.status = ConversationStatus::Ready.to_string();
        Ok(conversation)
    }
}
