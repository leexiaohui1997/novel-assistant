//! 多轮次 AI 驱动器（ai-v2）
//!
//! 串联以下关键环节，构成一次完整的"用户提示词 → AI 多轮回合 → 最终回复"流程：
//! 1. 参数预校验：`user_prompt` 非空
//! 2. 会话定位 / 创建：依据 `conversation_id` 分支
//! 3. 用户消息落库：写入 `User` 类型消息
//! 4. 多轮循环（`MAX_TURNS = 50`）：每轮以 `multi_turn = true` 调用单轮驱动器
//!    - 解析 assistant 输出中的 `tools-use` / `skills-use` 代码块
//!    - 有回喂：技能响应 system 消息（`ext_1 = "skills"`）→ 工具响应 system 消息（`ext_1 = "tools"`）
//!    - 无回喂：把会话状态扭转为 `Completed` 后退出
//! 5. 上限兜底：达到 `MAX_TURNS` 仍有回喂 → 把会话状态扭转为 `Completed`，正常返回
//! 6. 异常路径：单轮内部已置 `Error`；Markdown 解析失败由本驱动器显式置 `Error`
//!
//! 设计要点：
//! - 不直接持有 `SqlitePool` / `sqlx::Transaction`，写入全部走 `AiService` 既有方法
//! - 任意私有函数圈复杂度 < 5；主入口按"参数校验 → 主循环 → 异常分发"委托
//! - 所有错误信息均为中文，与 [`super::driver`] 风格一致

use uuid::Uuid;

use crate::database::models::ai_conversation::AiConversation;
use crate::database::models::ai_conversation_message::AiConversationMessage;

use super::super::dto::{CreateConversationInput, InsertConversationMessageInput};
use super::super::types::{ConversationStatus, ConversationType, MessageType};
use super::AiService;

/// 多轮驱动器最大轮次（达到该轮次仍有回喂时强制收尾）
const MAX_TURNS: usize = 50;

/// 成功回调函数类型
type OnSuccessCallback = Box<dyn FnOnce(AiConversationMessage) + Send>;
/// 错误回调函数类型
type OnErrorCallback = Box<dyn FnOnce(String) + Send>;

/// 一次本轮 AI 输出解析出的"待回喂"代码块
struct FeedbackBlock {
    /// 工具 / 技能名（取自代码块的 `info` 段；空字符串表示未指定）
    name: String,
    /// 代码块内容（参数 JSON 原文）
    content: String,
}

/// 本轮解析出的回喂请求
#[derive(Default)]
struct ParsedFeedback {
    tools: Vec<FeedbackBlock>,
    skills: Vec<FeedbackBlock>,
}

impl ParsedFeedback {
    /// 当前是否有任何待回喂请求
    fn is_empty(&self) -> bool {
        self.tools.is_empty() && self.skills.is_empty()
    }
}

impl AiService {
    /// 多轮次 AI 驱动器
    ///
    /// # 参数
    /// - `user_prompt`：用户提示词（必填，trim 后不能为空）
    /// - `model_id`：模型 ID（必填）
    /// - `conversation_id`：会话 ID（可选；`None` 时新建会话）
    /// - `conversation_type`：会话类型（可选；仅在新建会话时生效，默认 `Default`）
    /// - `title`：会话标题（可选；仅在新建会话时生效，默认 `None`）
    /// - `on_success`：成功回调，接收**最后一轮**新建的 assistant 消息
    /// - `on_error`：错误回调，接收中文错误描述
    ///
    /// # 返回
    /// - `Ok(AiConversationMessage)`：最后一轮 assistant 消息实体
    /// - `Err(String)`：中文错误描述（参数级错误**不**触发 `on_error`）
    pub async fn drive_ai_response_multi_turn(
        &self,
        user_prompt: String,
        model_id: Uuid,
        conversation_id: Option<Uuid>,
        conversation_type: Option<ConversationType>,
        title: Option<String>,
        on_success: Option<OnSuccessCallback>,
        on_error: Option<OnErrorCallback>,
    ) -> Result<AiConversationMessage, String> {
        let trimmed = user_prompt.trim();
        if trimmed.is_empty() {
            return Err("用户提示词不能为空".to_string());
        }

        let conversation = match self
            .locate_or_create_conversation(conversation_id, conversation_type, title)
            .await
        {
            Ok(c) => c,
            Err(LocateError::Param(e)) => return Err(e),
            Err(LocateError::Runtime(e)) => return self.dispatch_error(on_error, e),
        };

        if let Err(e) = self.persist_user_message(conversation.id, trimmed).await {
            return self.dispatch_error(on_error, e);
        }

        match self.run_multi_turn_loop(conversation.id, model_id).await {
            Ok(message) => self.dispatch_success(on_success, message),
            Err(e) => self.dispatch_error(on_error, e),
        }
    }

    /// 步骤 2：根据 `conversation_id` 定位或新建会话
    async fn locate_or_create_conversation(
        &self,
        conversation_id: Option<Uuid>,
        conversation_type: Option<ConversationType>,
        title: Option<String>,
    ) -> Result<AiConversation, LocateError> {
        match conversation_id {
            Some(id) => self.locate_existing_conversation(id).await,
            None => self
                .create_new_conversation(conversation_type, title)
                .await
                .map_err(LocateError::Runtime),
        }
    }

    /// 定位既有会话（不存在 → 参数级错误，不触发 `on_error`）
    async fn locate_existing_conversation(&self, id: Uuid) -> Result<AiConversation, LocateError> {
        let repo = self.conversation_repo.read().await;
        let conversation = repo
            .find_by_id(id)
            .await
            .map_err(|e| LocateError::Runtime(format!("查询会话失败: {}", e)))?;
        conversation.ok_or_else(|| LocateError::Param(format!("会话不存在: {}", id)))
    }

    /// 新建会话（使用默认值填充未指定字段）
    async fn create_new_conversation(
        &self,
        conversation_type: Option<ConversationType>,
        title: Option<String>,
    ) -> Result<AiConversation, String> {
        let input = CreateConversationInput {
            title,
            conversation_type: conversation_type.unwrap_or_default(),
            ..Default::default()
        };
        self.create_conversation(input).await
    }

    /// 步骤 3：用户消息落库
    async fn persist_user_message(
        &self,
        conversation_id: Uuid,
        prompt: &str,
    ) -> Result<(), String> {
        let input = InsertConversationMessageInput {
            conversation_id,
            message_type: MessageType::User,
            content: prompt.to_string(),
            thinking_content: None,
            input_tokens: None,
            output_tokens: None,
            model_id: None,
            ext_1: None,
            ext_2: None,
            ext_3: None,
        };
        self.insert_conversation_message(input).await.map(|_| ())
    }

    /// 步骤 4 + 5：多轮循环主流程
    ///
    /// - 每轮调用单轮驱动器 → 解析输出 → 落库回喂 → 进入下一轮
    /// - 无回喂或达到 `MAX_TURNS` 时把会话状态扭转为 `Completed` 并返回
    async fn run_multi_turn_loop(
        &self,
        conversation_id: Uuid,
        model_id: Uuid,
    ) -> Result<AiConversationMessage, String> {
        let mut last_message: Option<AiConversationMessage> = None;
        for turn in 0..MAX_TURNS {
            let message = self
                .drive_ai_response_internal(conversation_id, model_id, true, None, None)
                .await?;

            let feedback = match parse_feedback(&message.content) {
                Ok(f) => f,
                Err(e) => return self.fail_with_explicit_error(conversation_id, e).await,
            };

            if feedback.is_empty() {
                self.complete_conversation(conversation_id).await?;
                return Ok(message);
            }

            self.persist_feedback_messages(conversation_id, &feedback)
                .await?;

            last_message = Some(message);
            // 是否已达到上限的判断放在循环尾部，避免因边界条件少跑一轮
            if turn + 1 >= MAX_TURNS {
                break;
            }
        }

        // 达到 MAX_TURNS 仍有回喂：把状态扭转为 Completed，正常返回最后一轮 assistant 消息
        self.complete_conversation(conversation_id).await?;
        last_message.ok_or_else(|| "多轮驱动器异常：未捕获任何 assistant 消息".to_string())
    }

    /// 把会话状态显式写入 `Completed`（覆盖 `WaitingNextTurn`）
    async fn complete_conversation(&self, conversation_id: Uuid) -> Result<(), String> {
        let repo = self.conversation_repo.read().await;
        repo.update_status(conversation_id, &ConversationStatus::Completed.to_string())
            .await
            .map_err(|e| format!("扭转会话状态为已完成失败: {}", e))
    }

    /// Markdown 解析失败时的兜底：显式把会话置为 `Error`
    async fn fail_with_explicit_error(
        &self,
        conversation_id: Uuid,
        err: String,
    ) -> Result<AiConversationMessage, String> {
        let repo = self.conversation_repo.read().await;
        if let Err(e) = repo
            .update_status_and_prompt(
                conversation_id,
                &ConversationStatus::Error.to_string(),
                Some(&err),
            )
            .await
        {
            tracing::error!("多轮驱动器异常收尾更新会话状态失败: {}", e);
        }
        Err(err)
    }

    /// 步骤 7：把本轮回喂结果落库为 system 消息
    ///
    /// 顺序：先技能（`ext_1 = "skills"`）后工具（`ext_1 = "tools"`），与首轮系统提示注入顺序保持一致。
    async fn persist_feedback_messages(
        &self,
        conversation_id: Uuid,
        feedback: &ParsedFeedback,
    ) -> Result<(), String> {
        if !feedback.skills.is_empty() {
            let content = self.render_skill_responses(&feedback.skills).await;
            self.insert_feedback_message(conversation_id, content, "skills")
                .await?;
        }
        if !feedback.tools.is_empty() {
            let content = self.execute_tool_calls(&feedback.tools).await;
            self.insert_feedback_message(conversation_id, content, "tools")
                .await?;
        }
        Ok(())
    }

    /// 插入一条回喂 system 消息
    async fn insert_feedback_message(
        &self,
        conversation_id: Uuid,
        content: String,
        ext_1: &str,
    ) -> Result<(), String> {
        let input = InsertConversationMessageInput {
            conversation_id,
            message_type: MessageType::System,
            content,
            thinking_content: None,
            input_tokens: None,
            output_tokens: None,
            model_id: None,
            ext_1: Some(ext_1.to_string()),
            ext_2: None,
            ext_3: None,
        };
        self.insert_conversation_message(input).await.map(|_| ())
    }

    /// 工具调用：依次 execute，失败一律转中文文本，不中断循环
    async fn execute_tool_calls(&self, blocks: &[FeedbackBlock]) -> String {
        let mut parts: Vec<String> = Vec::with_capacity(blocks.len());
        for block in blocks {
            let response = self.execute_single_tool(block).await;
            parts.push(format_response_block(
                "tools-response",
                &block.name,
                &response,
            ));
        }
        parts.join("\n\n")
    }

    /// 执行单个工具调用，返回响应文本
    async fn execute_single_tool(&self, block: &FeedbackBlock) -> String {
        if block.name.is_empty() {
            return "未知工具: ".to_string();
        }

        let registry = self.tool_registry.read().await;
        let tool = match registry.get_tool(&block.name) {
            Some(t) => t.clone(),
            None => return format!("未知工具: {}", block.name),
        };
        drop(registry);

        let args = match parse_params_json(&block.content) {
            Ok(v) => v,
            Err(e) => return format!("工具参数 JSON 无效: {}", e),
        };

        match tool.execute(args).await {
            Ok(output) => output,
            Err(e) => format!("工具执行失败: {}", e),
        }
    }

    /// 技能渲染：优先 `render_with_instance`，回退 `TemplateManager::render`
    async fn render_skill_responses(&self, blocks: &[FeedbackBlock]) -> String {
        let mut parts: Vec<String> = Vec::with_capacity(blocks.len());
        for block in blocks {
            let response = self.render_single_skill(block).await;
            parts.push(format_response_block(
                "skills-response",
                &block.name,
                &response,
            ));
        }
        parts.join("\n\n")
    }

    /// 渲染单个技能调用，返回响应文本
    async fn render_single_skill(&self, block: &FeedbackBlock) -> String {
        if block.name.is_empty() {
            return "未知技能: ".to_string();
        }

        let registry = self.skill_registry.read().await;
        let skill = match registry.get_skill(&block.name) {
            Some(s) => s.clone(),
            None => return format!("未知技能: {}", block.name),
        };
        drop(registry);

        let params = match parse_params_json(&block.content) {
            Ok(v) => v,
            Err(e) => return format!("技能参数 JSON 无效: {}", e),
        };

        match dispatch_skill_render(skill.as_ref(), &self.template_manager, &params) {
            Ok(text) => text,
            Err(e) => format!("技能渲染失败: {}", e),
        }
    }

    /// 触发成功回调
    fn dispatch_success(
        &self,
        on_success: Option<OnSuccessCallback>,
        message: AiConversationMessage,
    ) -> Result<AiConversationMessage, String> {
        if let Some(callback) = on_success {
            callback(message.clone());
        }
        Ok(message)
    }

    /// 触发错误回调
    fn dispatch_error(
        &self,
        on_error: Option<OnErrorCallback>,
        err: String,
    ) -> Result<AiConversationMessage, String> {
        if let Some(callback) = on_error {
            callback(err.clone());
        }
        Err(err)
    }
}

/// 定位会话阶段的错误分支
enum LocateError {
    /// 参数级错误：调用方拼错 ID，不视为对话异常，不触发 `on_error`
    Param(String),
    /// 运行时错误：DB / 初始化失败等，需要触发 `on_error`
    Runtime(String),
}

/// 解析本轮 assistant 输出中的 `tools-use` / `skills-use` 代码块
fn parse_feedback(content: &str) -> Result<ParsedFeedback, String> {
    let mut feedback = ParsedFeedback::default();

    let walk = crate::utils::markdown::walk_code_blocks(
        content,
        |label, _info| label == "tools-use" || label == "skills-use",
        |label, info, body| collect_block(&mut feedback, label, info, body),
    );

    walk.map_err(|e| format!("本轮 AI 输出解析失败: {}", e))?;
    Ok(feedback)
}

/// 把命中的代码块按 label 分配到工具组 / 技能组
fn collect_block(feedback: &mut ParsedFeedback, label: &str, info: &str, body: &str) {
    let block = FeedbackBlock {
        name: info.trim().to_string(),
        content: body.to_string(),
    };
    if label == "tools-use" {
        feedback.tools.push(block);
    } else if label == "skills-use" {
        feedback.skills.push(block);
    }
}

/// 解析参数 JSON：空字符串视为 `{}`
fn parse_params_json(raw: &str) -> Result<serde_json::Value, String> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return Ok(serde_json::json!({}));
    }
    serde_json::from_str(trimmed).map_err(|e| e.to_string())
}

/// 按需求 5.8 / 6.8 拼装单个响应代码块
fn format_response_block(label: &str, name: &str, body: &str) -> String {
    format!("```{} {}\n{}\n```", label, name, body)
}

/// 技能渲染分派：优先实例渲染，回退通用渲染
fn dispatch_skill_render(
    skill: &dyn crate::ai_v2::skills::AiSkill,
    manager: &crate::ai_v2::template::TemplateManager,
    params: &serde_json::Value,
) -> Result<String, String> {
    if let Some(result) = skill.render_with_instance(manager, params) {
        return result.map_err(|e| e.to_string());
    }
    manager
        .render(skill.template_id(), params)
        .map_err(|e| e.to_string())
}
