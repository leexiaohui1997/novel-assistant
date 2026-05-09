//! AI 服务驱动器（ai-v2）
//!
//! 串联以下关键环节：
//! 1. 会话有效性校验（末条消息必须为 `User`，且会话状态不为 `Computing`）
//! 2. 状态推进到 `Computing`，同时清空历史异常原因
//! 3. 加载历史消息 → 构造 `role/content` 数组 → 复用 v1 `AiService::chat`
//! 4. 二次状态校验（若期间外部把状态改为非 `Computing`，则优雅终止）
//! 5. 事务收尾：写入一条 `assistant` 消息 + 推进会话为 `Completed`
//! 6. 通过 Tauri `AppHandle` 向前端派发事件（成功 / 异常）
//!
//! 设计要点：
//! - 任意私有函数圈复杂度 < 5，主入口 `drive_ai_response` 按步骤委托
//! - 仅调用仓储层提供的抽象方法，不直接操作 `SqlitePool` 或 `sqlx::Transaction`
//! - 从"调用 AI"开始的任何失败都走 [`handle_failure`]：置 `Error` + 派发 `error` 事件
//! - "状态已被改为非 Computing" 分支**不触发**异常路径（视为用户主动中止）
//! - 事件名为 `AiConversation/Response/<conversation_id>`，前端按会话 ID 精确监听

use uuid::Uuid;

use crate::ai::types::{AiRequestData, Message, MessageRole};
use crate::database::models::ai_conversation::AiConversation;
use crate::database::models::ai_conversation_message::{
    AiConversationMessage, CreateAiConversationMessage,
};

use super::super::skills::generate_skill_prompt;
use super::super::tools::generate_tool_prompt;
use super::super::types::{ConversationStatus, MessageType};
use super::AiService;

/// 成功回调函数类型
type OnSuccessCallback = Box<dyn FnOnce(AiConversationMessage) + Send>;
/// 错误回调函数类型
type OnErrorCallback = Box<dyn FnOnce(String) + Send>;

impl AiService {
    /// 驱动一次 AI 响应流程
    ///
    /// # 参数
    /// - `conversation_id`: 会话 ID
    /// - `model_id`: 模型 ID（v2 数据层 ID）
    /// - `on_success`: 可选的成功回调，接收新建的 assistant 消息实体
    /// - `on_error`: 可选的错误回调，接收中文错误描述
    ///
    /// # 返回
    /// - `Ok(AiConversationMessage)`: 新建的 `assistant` 消息实体
    /// - `Err(String)`: 中文错误描述
    pub async fn drive_ai_response(
        &self,
        conversation_id: Uuid,
        model_id: Uuid,
        on_success: Option<OnSuccessCallback>,
        on_error: Option<OnErrorCallback>,
    ) -> Result<AiConversationMessage, String> {
        self.drive_ai_response_internal(conversation_id, model_id, false, on_success, on_error)
            .await
    }

    /// 单轮驱动器的内部实现（需求 3.2）
    ///
    /// 与公开方法 [`Self::drive_ai_response`] 的唯一差异在于 `multi_turn` 参数：
    /// - `multi_turn = false`：保持原语义，收尾写入 `Completed`
    /// - `multi_turn = true`：由多轮驱动器调用，收尾写入 `WaitingNextTurn`
    pub(crate) async fn drive_ai_response_internal(
        &self,
        conversation_id: Uuid,
        model_id: Uuid,
        multi_turn: bool,
        on_success: Option<OnSuccessCallback>,
        on_error: Option<OnErrorCallback>,
    ) -> Result<AiConversationMessage, String> {
        let (_conversation, messages) = self.validate_conversation(conversation_id).await?;
        self.begin_computing(conversation_id).await?;

        match self
            .run_to_finalization(conversation_id, model_id, messages, multi_turn)
            .await
        {
            Ok(DriveOutcome::Success(message)) => {
                if let Some(callback) = on_success {
                    callback(message.clone());
                }
                Ok(message)
            }
            Ok(DriveOutcome::Aborted(err)) => {
                // 状态已被外部改为非 Computing：不置 Error、不派发 error 事件
                Err(err)
            }
            Err(err) => {
                if let Some(callback) = on_error {
                    callback(err.clone());
                }
                Err(self.handle_failure(conversation_id, err).await)
            }
        }
    }

    /// 需求 2：会话有效性校验
    ///
    /// 依次检查：会话存在 / 状态非 `Computing` / 至少一条消息 / 末条为 `User`。
    async fn validate_conversation(
        &self,
        conversation_id: Uuid,
    ) -> Result<(AiConversation, Vec<AiConversationMessage>), String> {
        let conversation = self.load_conversation(conversation_id).await?;
        ensure_not_computing(&conversation)?;

        let messages = self.list_messages(conversation_id).await?;
        ensure_last_message_is_drivable(&messages)?;

        Ok((conversation, messages))
    }

    /// 读取会话实体（不存在时返回中文错误）
    async fn load_conversation(&self, conversation_id: Uuid) -> Result<AiConversation, String> {
        let repo = self.conversation_repo.read().await;
        let conversation = repo
            .find_by_id(conversation_id)
            .await
            .map_err(|e| format!("查询会话失败: {}", e))?;
        conversation.ok_or_else(|| format!("会话不存在: {}", conversation_id))
    }

    /// 读取会话的全部历史消息（按 `sequence ASC`）
    async fn list_messages(
        &self,
        conversation_id: Uuid,
    ) -> Result<Vec<AiConversationMessage>, String> {
        let repo = self.message_repo.read().await;
        repo.list_by_conversation(conversation_id)
            .await
            .map_err(|e| format!("加载会话消息失败: {}", e))
    }

    /// 需求 3：推进为 Computing 并清空 prompt（原子写）
    async fn begin_computing(&self, conversation_id: Uuid) -> Result<(), String> {
        let repo = self.conversation_repo.read().await;
        repo.update_status_and_prompt(
            conversation_id,
            &ConversationStatus::Computing.to_string(),
            None,
        )
        .await
        .map_err(|e| format!("推进会话状态为计算中失败: {}", e))
    }

    /// 需求 4 + 5 + 6：调用 AI → 二次状态校验 → 事务写入 assistant 消息并置 Completed
    ///
    /// 所有失败冒泡给主入口统一交给 [`Self::handle_failure`]；
    /// 仅"状态已被改为非 Computing"这一分支会返回 [`DriveOutcome::Aborted`]，
    /// 主入口据此跳过 `handle_failure`。
    async fn run_to_finalization(
        &self,
        conversation_id: Uuid,
        model_id: Uuid,
        history: Vec<AiConversationMessage>,
        multi_turn: bool,
    ) -> Result<DriveOutcome, String> {
        let ai_response = self.call_ai(conversation_id, model_id, &history).await?;

        if !self.still_computing(conversation_id).await? {
            return Ok(DriveOutcome::Aborted(
                "会话状态已变更，驱动流程终止".to_string(),
            ));
        }

        let sequence = next_sequence_of(&history);
        let snapshot = self.resolve_driver_model_snapshot(model_id).await?;
        let payload =
            build_assistant_payload(conversation_id, sequence, &ai_response, model_id, snapshot);

        let target_status = pick_target_status(multi_turn);
        let message = self
            .persist_finalized_message(payload, target_status)
            .await?;
        Ok(DriveOutcome::Success(message))
    }

    /// 注入工具提示词到消息列表头部
    async fn build_tool_system_prompt(
        &self,
        conversation_id: Uuid,
    ) -> Result<Option<String>, String> {
        let repo = self.conversation_repo.read().await;
        let registry_guard = self.tool_registry.read().await;
        crate::ai_v2::service::driver::build_tool_system_prompt(
            conversation_id,
            repo.as_ref(),
            &registry_guard,
            &self.template_manager,
        )
        .await
    }

    /// 注入技能提示词到消息列表头部
    async fn build_skill_system_prompt(
        &self,
        conversation_id: Uuid,
    ) -> Result<Option<String>, String> {
        let repo = self.conversation_repo.read().await;
        let registry_guard = self.skill_registry.read().await;
        crate::ai_v2::service::driver::build_skill_system_prompt(
            conversation_id,
            repo.as_ref(),
            &registry_guard,
            &self.template_manager,
        )
        .await
    }

    /// 复用 v1 [`crate::ai::service::AiService::chat`] 完成实际 AI 调用
    async fn call_ai(
        &self,
        conversation_id: Uuid,
        model_id: Uuid,
        history: &[AiConversationMessage],
    ) -> Result<AiChatOutput, String> {
        let mut messages = build_messages(history)?;

        // 注入工具提示词
        if let Some(tool_prompt) = self.build_tool_system_prompt(conversation_id).await? {
            messages.insert(
                0,
                Message {
                    role: MessageRole::System,
                    content: tool_prompt,
                },
            );
        }

        // 注入技能提示词（插入到最前，确保顺序为：skills → tools → 业务消息）
        if let Some(skill_prompt) = self.build_skill_system_prompt(conversation_id).await? {
            messages.insert(
                0,
                Message {
                    role: MessageRole::System,
                    content: skill_prompt,
                },
            );
        }

        let request = AiRequestData {
            model_id: Some(model_id),
            messages,
        };

        let response = self.v1_ai_service.chat(request).await?;
        let (input_tokens, output_tokens) = extract_usage(&response.response);

        Ok(AiChatOutput {
            content: response.content,
            thinking_content: response.thinking_content,
            input_tokens,
            output_tokens,
        })
    }

    /// 需求 5：再次读取会话，判断状态是否仍为 `Computing`
    async fn still_computing(&self, conversation_id: Uuid) -> Result<bool, String> {
        let conversation = self.load_conversation(conversation_id).await?;
        Ok(conversation.status == ConversationStatus::Computing.to_string())
    }

    /// 解析模型 / 供应商快照（供写消息时持久化名称）
    ///
    /// 失败时返回中文错误；会话将在主入口走异常路径。
    async fn resolve_driver_model_snapshot(
        &self,
        model_id: Uuid,
    ) -> Result<DriverModelSnapshot, String> {
        let model = {
            let repo = self.model_repo.read().await;
            repo.find_by_id(model_id)
                .await
                .map_err(|e| format!("查询模型失败 ({}): {}", model_id, e))?
        };

        let provider = {
            let repo = self.provider_repo.read().await;
            repo.find_by_id(model.provider_id)
                .await
                .map_err(|e| format!("查询供应商失败 ({}): {}", model.provider_id, e))?
        };

        Ok(DriverModelSnapshot {
            provider_name: Some(provider.name),
            model_name: Some(model.alias),
        })
    }

    /// 通过事务版仓储方法写入消息并把会话状态推进到目标值
    ///
    /// - 单轮模式：目标为 `Completed`
    /// - 多轮模式：目标为 `WaitingNextTurn`（由多轮驱动器后续显式扭转为 `Completed`）
    async fn persist_finalized_message(
        &self,
        payload: CreateAiConversationMessage,
        target_status: ConversationStatus,
    ) -> Result<AiConversationMessage, String> {
        let repo = self.message_repo.read().await;
        repo.finalize_assistant_turn(payload, &target_status.to_string())
            .await
            .map_err(|e| format!("写入 AI 响应并完成会话失败: {}", e))
    }

    /// 需求 8：统一异常处理器
    ///
    /// - 原子把 `status = Error`、`prompt = <err>`
    /// - 返回原始错误文本，主入口据此冒泡
    async fn handle_failure(&self, conversation_id: Uuid, err: String) -> String {
        let repo = self.conversation_repo.read().await;
        if let Err(e) = repo
            .update_status_and_prompt(
                conversation_id,
                &ConversationStatus::Error.to_string(),
                Some(&err),
            )
            .await
        {
            tracing::error!("异常收尾更新会话状态失败: {}", e);
        }
        drop(repo);

        err
    }
}

/// 主流程的两种成功返回分支
enum DriveOutcome {
    /// AI 成功，assistant 消息已入库
    Success(AiConversationMessage),
    /// 二次状态校验未命中 `Computing`：静默终止，不触发异常处理
    Aborted(String),
}

/// 从 v1 `AiChatResponse` 中抽取的驱动器所需子集
struct AiChatOutput {
    content: String,
    thinking_content: Option<String>,
    input_tokens: i32,
    output_tokens: i32,
}

/// 驱动器写消息时的模型 / 供应商名称快照
struct DriverModelSnapshot {
    provider_name: Option<String>,
    model_name: Option<String>,
}

/// 确保会话当前状态不是 `Computing`（需求 2.3）
fn ensure_not_computing(conversation: &AiConversation) -> Result<(), String> {
    if conversation.status == ConversationStatus::Computing.to_string() {
        return Err("会话正在计算中，无法重复驱动".to_string());
    }
    Ok(())
}

/// 确保末条消息满足可驱动条件（需求 3.1）
///
/// 允许以下任一情况：
/// - 末条是 `User`（单轮原语义，用户驱动）
/// - 末条是 `System` 且 `ext_1 ∈ {"tools", "skills"}`（多轮回喂驱动）
fn ensure_last_message_is_drivable(messages: &[AiConversationMessage]) -> Result<(), String> {
    let last = messages.last().ok_or("会话无可驱动的消息".to_string())?;
    if is_user_message(last) || is_feedback_system_message(last) {
        return Ok(());
    }
    Err("会话最后一条消息不具备可驱动条件".to_string())
}

/// 判断是否为用户消息
fn is_user_message(msg: &AiConversationMessage) -> bool {
    msg.message_type == MessageType::User.to_string()
}

/// 判断是否为多轮回喂形态的 system 消息（`ext_1 ∈ {"tools", "skills"}`）
fn is_feedback_system_message(msg: &AiConversationMessage) -> bool {
    if msg.message_type != MessageType::System.to_string() {
        return false;
    }
    matches!(msg.ext_1.as_deref(), Some("tools") | Some("skills"))
}

/// 根据 `multi_turn` 选择单轮收尾目标状态（需求 3.3 / 3.4）
fn pick_target_status(multi_turn: bool) -> ConversationStatus {
    if multi_turn {
        ConversationStatus::WaitingNextTurn
    } else {
        ConversationStatus::Completed
    }
}

/// 把历史消息映射为 v1 `ai::types::Message` 列表（按 `sequence ASC` 已保序）
fn build_messages(history: &[AiConversationMessage]) -> Result<Vec<Message>, String> {
    history.iter().map(map_history_message).collect()
}

/// 单条历史消息 → v1 `Message`；非法 `message_type` 以中文错误返回
fn map_history_message(msg: &AiConversationMessage) -> Result<Message, String> {
    let role = match msg.message_type.as_str() {
        s if s == MessageType::System.to_string() => MessageRole::System,
        s if s == MessageType::User.to_string() => MessageRole::User,
        s if s == MessageType::Assistant.to_string() => MessageRole::Assistant,
        other => return Err(format!("无法识别的消息类型: {}", other)),
    };
    Ok(Message {
        role,
        content: msg.content.clone(),
    })
}

/// 从 `AiChatResponse.response` 中抽取 `prompt_tokens` / `completion_tokens`；缺失按 0
fn extract_usage(response: &serde_json::Value) -> (i32, i32) {
    let usage = match response.get("usage") {
        Some(v) => v,
        None => return (0, 0),
    };
    let input = usage
        .get("prompt_tokens")
        .and_then(|v| v.as_i64())
        .unwrap_or(0) as i32;
    let output = usage
        .get("completion_tokens")
        .and_then(|v| v.as_i64())
        .unwrap_or(0) as i32;
    (input, output)
}

/// 根据当前历史消息数量计算新 assistant 消息的 `sequence`（count + 1）
fn next_sequence_of(history: &[AiConversationMessage]) -> i64 {
    history.len() as i64 + 1
}

/// 构建工具系统提示词
async fn build_tool_system_prompt(
    conversation_id: Uuid,
    conversation_repo: &dyn crate::database::repositories::AiConversationRepository,
    tool_registry: &crate::ai_v2::tools::ToolRegistry,
    template_manager: &crate::ai_v2::template::TemplateManager,
) -> Result<Option<String>, String> {
    let prompt = generate_tool_prompt(
        conversation_id,
        conversation_repo,
        tool_registry,
        template_manager,
    )
    .await?;

    if prompt.is_empty() {
        Ok(None)
    } else {
        Ok(Some(prompt))
    }
}

/// 构建技能系统提示词
async fn build_skill_system_prompt(
    conversation_id: Uuid,
    conversation_repo: &dyn crate::database::repositories::AiConversationRepository,
    skill_registry: &crate::ai_v2::skills::SkillRegistry,
    template_manager: &crate::ai_v2::template::TemplateManager,
) -> Result<Option<String>, String> {
    let prompt = generate_skill_prompt(
        conversation_id,
        conversation_repo,
        skill_registry,
        template_manager,
    )
    .await?;

    if prompt.is_empty() {
        Ok(None)
    } else {
        Ok(Some(prompt))
    }
}

/// 组装 `CreateAiConversationMessage`（assistant 类型）
fn build_assistant_payload(
    conversation_id: Uuid,
    sequence: i64,
    ai: &AiChatOutput,
    model_id: Uuid,
    snapshot: DriverModelSnapshot,
) -> CreateAiConversationMessage {
    CreateAiConversationMessage {
        id: Uuid::new_v4(),
        conversation_id,
        message_type: MessageType::Assistant.to_string(),
        content: ai.content.clone(),
        thinking_content: ai.thinking_content.clone(),
        input_tokens: ai.input_tokens as i64,
        output_tokens: ai.output_tokens as i64,
        sequence,
        model_id: Some(model_id),
        provider_name: snapshot.provider_name,
        model_name: snapshot.model_name,
        ext_1: None,
        ext_2: None,
        ext_3: None,
    }
}
