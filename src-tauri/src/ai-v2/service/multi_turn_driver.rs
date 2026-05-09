//! 多轮次 AI 驱动器（ai-v2）
//!
//! 本模块将"一次用户提示词 → AI 多轮回合 → 最终回复"的完整流程拆分为两个阶段：
//! 1. **准备阶段** [`prepare_ai_response`]：主线程同步完成参数校验、推荐模型解析、
//!    会话定位/创建、用户消息落库，并把后台执行所需的一切打包为 [`PreparedAiContext`]。
//! 2. **执行阶段** [`run_prepared_ai_response`]：由调用方在 `tokio::spawn` 中驱动，
//!    驱动多轮循环并通过 `on_success` / `on_error` 回调上报结果。
//!
//! 设计要点：
//! - 不直接持有 `SqlitePool` / `sqlx::Transaction`，写入全部走 `AiService` 既有方法
//! - 任意函数圈复杂度 < 5
//! - 所有错误信息均为中文

use std::sync::Arc;

use tokio::sync::RwLock;
use uuid::Uuid;

use crate::ai_v2::utils::find_recommended_model;
use crate::database::models::ai_conversation::AiConversation;
use crate::database::models::ai_conversation_message::AiConversationMessage;
use crate::database::repositories::AiCallLogRepository;

use super::super::dto::{CreateConversationInput, InsertConversationMessageInput};
use super::super::types::{ConversationStatus, ConversationType, MessageType};
use super::AiService;

/// 多轮驱动器最大轮次（达到该轮次仍有回喂时强制收尾）
const MAX_TURNS: usize = 50;

/// 成功回调函数类型
pub type OnSuccessCallback = Box<dyn FnOnce(AiConversationMessage) + Send>;
/// 错误回调函数类型
pub type OnErrorCallback = Box<dyn FnOnce(String) + Send>;

/// 准备阶段入参
pub struct PrepareAiParams {
    /// 用户提示词（必填，trim 后不能为空）
    pub user_prompt: String,
    /// 会话 ID（可选；`None` 时新建会话）
    pub conversation_id: Option<Uuid>,
    /// 会话类型（可选；仅在新建会话时生效，默认 `Default`）
    pub conversation_type: Option<ConversationType>,
    /// 会话标题（可选；仅在新建会话时生效，默认 `None`）
    pub title: Option<String>,
    /// 模型 ID（可选；`None` 时走推荐模型）
    pub model_id: Option<Uuid>,
}

/// 准备阶段产出的后台执行上下文
pub struct PreparedAiContext {
    /// 已定位 / 创建完成的会话 ID
    pub conversation_id: Uuid,
    /// 实际将被使用的模型 ID
    pub model_id: Uuid,
    /// v2 AiService 句柄（后台执行多轮循环所需）
    pub ai_service: Arc<AiService>,
}

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

/// 准备阶段：参数校验 → 推荐模型解析 → 会话定位/创建 → 插入用户消息
///
/// # 参数
/// - `ai_service`：v2 AI 服务句柄
/// - `call_log_repo`：AI 调用日志仓储（仅用于推导推荐模型）
/// - `params`：准备阶段入参
///
/// # 返回
/// - `Ok((conversation_id, PreparedAiContext))`：会话 ID 与后台执行上下文
/// - `Err(String)`：任一步骤的中文错误描述（不生成 `PreparedAiContext`）
pub async fn prepare_ai_response(
    ai_service: Arc<AiService>,
    call_log_repo: Arc<RwLock<Box<dyn AiCallLogRepository + Send + Sync>>>,
    params: PrepareAiParams,
) -> Result<(Uuid, PreparedAiContext), String> {
    let trimmed = validate_user_prompt(&params.user_prompt)?;

    let model_id = resolve_model_id(&ai_service, &call_log_repo, params.model_id).await?;

    let conversation = ai_service
        .locate_or_create_conversation(
            params.conversation_id,
            params.conversation_type,
            params.title,
        )
        .await
        .map_err(locate_error_to_string)?;

    ai_service
        .persist_user_message(conversation.id, trimmed)
        .await?;

    tracing::info!(
        "prepare_ai_response 完成: conversation_id={}, model_id={}",
        conversation.id,
        model_id
    );

    let ctx = PreparedAiContext {
        conversation_id: conversation.id,
        model_id,
        ai_service,
    };
    Ok((conversation.id, ctx))
}

/// 执行阶段：驱动多轮循环并通过回调上报结果
///
/// # 参数
/// - `ctx`：准备阶段产出的上下文
/// - `on_success`：成功回调（接收最后一轮 assistant 消息）
/// - `on_error`：错误回调（接收中文错误描述）
pub async fn run_prepared_ai_response(
    ctx: PreparedAiContext,
    on_success: Option<OnSuccessCallback>,
    on_error: Option<OnErrorCallback>,
) {
    tracing::info!(
        "run_prepared_ai_response 启动: conversation_id={}, model_id={}",
        ctx.conversation_id,
        ctx.model_id
    );

    let result = ctx
        .ai_service
        .run_multi_turn_loop(ctx.conversation_id, ctx.model_id)
        .await;

    match result {
        Ok(message) => {
            tracing::info!(
                "run_prepared_ai_response 结束: conversation_id={} 成功",
                ctx.conversation_id
            );
            if let Some(cb) = on_success {
                cb(message);
            }
        }
        Err(err) => {
            tracing::warn!(
                "run_prepared_ai_response 结束: conversation_id={} 失败 - {}",
                ctx.conversation_id,
                err
            );
            if let Some(cb) = on_error {
                cb(err);
            }
        }
    }
}

/// 校验用户提示词非空，返回 trim 后的引用
fn validate_user_prompt(raw: &str) -> Result<&str, String> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return Err("用户提示词不能为空".to_string());
    }
    Ok(trimmed)
}

/// 解析真实模型 ID：`Some` 时直接返回；`None` 时走推荐模型
async fn resolve_model_id(
    ai_service: &AiService,
    call_log_repo: &Arc<RwLock<Box<dyn AiCallLogRepository + Send + Sync>>>,
    explicit: Option<Uuid>,
) -> Result<Uuid, String> {
    if let Some(id) = explicit {
        return Ok(id);
    }

    let call_log_guard = call_log_repo.read().await;
    let model_guard = ai_service.model_repo.read().await;
    let model = find_recommended_model(call_log_guard.as_ref(), model_guard.as_ref()).await?;
    Ok(model.id)
}

/// 把 `LocateError` 统一转为 `String`（prepare 阶段不区分参数 / 运行时错误）
fn locate_error_to_string(err: LocateError) -> String {
    match err {
        LocateError::Param(e) | LocateError::Runtime(e) => e,
    }
}

impl AiService {
    /// 步骤 2：根据 `conversation_id` 定位或新建会话
    pub(crate) async fn locate_or_create_conversation(
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
    pub(crate) async fn persist_user_message(
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
    pub(crate) async fn run_multi_turn_loop(
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
}

/// 定位会话阶段的错误分支
pub(crate) enum LocateError {
    /// 参数级错误：调用方拼错 ID，不视为对话异常
    Param(String),
    /// 运行时错误：DB / 初始化失败等
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
