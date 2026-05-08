//! AI 会话消息相关的 Service 方法
//!
//! 本模块提供 `AiService::insert_conversation_message`，职责链：
//! 1. 计算消息序号 `sequence = count + 1`
//! 2. 解析模型 / 供应商快照（可选）
//! 3. 组装 `CreateAiConversationMessage` 并写入数据库
//! 4. 同步更新会话的 `last_message_at`
//!
//! 每个步骤拆为独立辅助函数，保持主函数圈复杂度 < 5。

use uuid::Uuid;

use crate::database::{
    error::DbError,
    models::ai_conversation_message::{AiConversationMessage, CreateAiConversationMessage},
};

use super::super::dto::InsertConversationMessageInput;
use super::AiService;

/// 模型 / 供应商快照三元组
///
/// 统一描述写入消息时需要持久化的 `(model_id, provider_name, model_name)` 快照。
struct ModelSnapshot {
    model_id: Option<Uuid>,
    provider_name: Option<String>,
    model_name: Option<String>,
}

impl ModelSnapshot {
    /// 未提供模型 ID 时使用的空快照
    fn empty() -> Self {
        Self {
            model_id: None,
            provider_name: None,
            model_name: None,
        }
    }
}

impl AiService {
    /// 插入一条会话消息
    ///
    /// # 行为
    /// 1. 自动计算 `sequence`（当前会话消息数 + 1，从 1 开始递增）
    /// 2. 若 `input.model_id` 存在，则查询模型与供应商，写入 `provider_name` / `model_name` 快照；
    ///    若查询失败，返回中文错误，**不写入任何数据**。
    /// 3. `input_tokens` / `output_tokens` 未传时按 `0` 写入。
    /// 4. 写入成功后，同步更新会话的 `last_message_at`；若同步失败，透传中文错误。
    ///
    /// # 参数
    /// - `input`: 插入消息的入参 DTO
    ///
    /// # 返回
    /// - `Ok(AiConversationMessage)`: 新建的完整消息实体（含数据库生成的时间字段）
    /// - `Err(String)`: 中文错误信息
    pub async fn insert_conversation_message(
        &self,
        input: InsertConversationMessageInput,
    ) -> Result<AiConversationMessage, String> {
        let conversation_id = input.conversation_id;
        let sequence = self.next_message_sequence(conversation_id).await?;
        let snapshot = self.resolve_model_snapshot(input.model_id).await?;
        let payload = build_create_payload(input, sequence, snapshot);

        let created = self.persist_message(payload).await?;
        self.touch_conversation_last_message_at(conversation_id, created.created_at)
            .await?;

        Ok(created)
    }

    /// 计算下一条消息的 `sequence`（count + 1）
    async fn next_message_sequence(&self, conversation_id: Uuid) -> Result<i64, String> {
        let repo = self.message_repo.read().await;
        repo.count_by_conversation(conversation_id)
            .await
            .map(|count| count + 1)
            .map_err(|e| format!("统计会话消息数量失败: {}", e))
    }

    /// 解析模型 / 供应商快照
    ///
    /// - `None`：返回空快照
    /// - `Some(id)`：查询模型 → 查询供应商 → 组装快照
    ///
    /// 任一查询失败（如记录不存在）返回中文错误。
    async fn resolve_model_snapshot(
        &self,
        model_id: Option<Uuid>,
    ) -> Result<ModelSnapshot, String> {
        let Some(id) = model_id else {
            return Ok(ModelSnapshot::empty());
        };

        let model = {
            let repo = self.model_repo.read().await;
            repo.find_by_id(id)
                .await
                .map_err(|e| snapshot_error("模型不存在", id, e))?
        };

        let provider = {
            let repo = self.provider_repo.read().await;
            repo.find_by_id(model.provider_id)
                .await
                .map_err(|e| snapshot_error("供应商不存在", model.provider_id, e))?
        };

        Ok(ModelSnapshot {
            model_id: Some(id),
            provider_name: Some(provider.name),
            model_name: Some(model.alias),
        })
    }

    /// 将组装好的 payload 持久化为消息实体
    async fn persist_message(
        &self,
        payload: CreateAiConversationMessage,
    ) -> Result<AiConversationMessage, String> {
        let repo = self.message_repo.read().await;
        repo.create(payload)
            .await
            .map_err(|e| format!("插入会话消息失败: {}", e))
    }

    /// 同步更新会话的 `last_message_at`
    ///
    /// 取值为新消息的 `created_at`，保证 `ai_conversations.last_message_at`
    /// 与 `ai_conversation_messages.created_at` 严格一致，避免使用 `Utc::now()`
    /// 因调度间隙带来的毫秒级偏差。
    async fn touch_conversation_last_message_at(
        &self,
        conversation_id: Uuid,
        at: chrono::DateTime<chrono::Utc>,
    ) -> Result<(), String> {
        let repo = self.conversation_repo.read().await;
        repo.touch_last_message_at(conversation_id, at)
            .await
            .map_err(|e| format!("更新会话最后消息时间失败: {}", e))
    }
}

/// 根据 DTO + 计算出的序号 + 模型快照，组装仓储层入参
///
/// 独立成纯函数，便于未来单测；同时避免把结构体字段映射混进 async 主流程。
fn build_create_payload(
    input: InsertConversationMessageInput,
    sequence: i64,
    snapshot: ModelSnapshot,
) -> CreateAiConversationMessage {
    CreateAiConversationMessage {
        id: Uuid::new_v4(),
        conversation_id: input.conversation_id,
        message_type: input.message_type.to_string(),
        content: input.content,
        thinking_content: input.thinking_content,
        input_tokens: input.input_tokens.unwrap_or(0),
        output_tokens: input.output_tokens.unwrap_or(0),
        sequence,
        model_id: snapshot.model_id,
        provider_name: snapshot.provider_name,
        model_name: snapshot.model_name,
        ext_1: input.ext_1,
        ext_2: input.ext_2,
        ext_3: input.ext_3,
    }
}

/// 组装模型 / 供应商快照错误的中文提示
fn snapshot_error(kind: &str, id: Uuid, source: DbError) -> String {
    format!("{}: {} ({})", kind, id, source)
}
