//! 会话类型初始化模块
//!
//! 职责：
//! - 定义 [`ConversationInitializer`] trait 作为各会话类型"基类"层的方法签名；
//! - 为每种 [`crate::ai_v2::ConversationType`] 提供独立实例（`default` / `novel_profile` /
//!   `novel_workbench`），各自承担业务初始化逻辑；
//! - 提供统一调度入口 [`run_conversation_initializer`]，由 `AiService::create_conversation`
//!   在会话落库后立即调用，完成"分发到具体类型 → 成功后统一置 `Ready` / 失败置 `Error`"
//!   的模板方法流程。
//!
//! # 扩展方式
//! 新增会话类型时：
//! ① 在 `types.rs` 的 [`crate::ai_v2::ConversationType`] 枚举追加成员；
//! ② 在本目录新增实现了 [`ConversationInitializer`] 的结构体；
//! ③ 在 [`run_conversation_initializer`] 的 `match` 中登记新分支。其余代码无需改动。

mod default;
mod novel_profile;
mod novel_workbench;

use async_trait::async_trait;
use tracing::error;
use uuid::Uuid;

use crate::database::models::ai_conversation::AiConversation;

use super::service::AiService;
use super::types::{ConversationStatus, ConversationType};

pub use default::DefaultInitializer;
pub use novel_profile::NovelProfileInitializer;
pub use novel_workbench::NovelWorkbenchInitializer;

/// 会话类型初始化器 trait
///
/// 每个 [`ConversationType`] 成员对应一个实现了本 trait 的结构体，由
/// [`run_conversation_initializer`] 根据会话的 `conversation_type` 字符串动态路由。
///
/// # 约束
/// 实现者只负责自身类型的业务初始化（例如写入预设系统消息、准备上下文等），
/// **不应自行修改会话状态**；会话状态的推进（`Init → Ready` 或 `Init → Error`）
/// 由统一调度入口 [`run_conversation_initializer`] 在收尾阶段完成。
#[async_trait]
pub trait ConversationInitializer: Send + Sync {
    /// 声明本初始化器归属的会话类型
    ///
    /// 用于：① 自校验（可在路由层做 `assert` 防御）；② 日志与错误定位。
    fn conversation_type(&self) -> ConversationType;

    /// 执行该类型的业务初始化
    ///
    /// # 参数
    /// - `service`: 当前 [`AiService`] 引用（实现者可借此访问仓储：`conversation_repo` /
    ///   `message_repo` / `model_repo` / `provider_repo`）
    /// - `conversation`: 已落库的会话实体（状态此时必为 `Init`）
    ///
    /// # 返回
    /// - `Ok(())`：初始化成功，调用方会随后把状态置为 `Ready`
    /// - `Err(String)`：中文错误描述，调用方会把状态置为 `Error` 并将错误冒泡
    async fn initialize(
        &self,
        service: &AiService,
        conversation: &AiConversation,
    ) -> Result<(), String>;
}

/// 根据会话类型路由到对应的初始化器实例
///
/// 将 `match` 分支独立封装，确保 [`run_conversation_initializer`] 自身的圈复杂度保持 < 5。
/// 新增会话类型时，仅需在此处追加一条 `match` 分支。
fn resolve_initializer(ct: ConversationType) -> &'static dyn ConversationInitializer {
    match ct {
        ConversationType::Default => &DefaultInitializer,
        ConversationType::NovelProfile => &NovelProfileInitializer,
        ConversationType::NovelWorkbench => &NovelWorkbenchInitializer,
    }
}

/// 状态收尾：把会话状态更新为目标值（`Ready` 或 `Error`）
///
/// 本函数是 [`run_conversation_initializer`] 的"基类统一收尾"环节。
/// 若底层仓储返回错误，则**仅记录日志**，不向上冒泡，避免覆盖业务原因错误。
///
/// # 参数
/// - `service`: 当前 [`AiService`] 引用
/// - `conversation_id`: 待更新的会话 ID
/// - `status`: 目标状态（通常为 [`ConversationStatus::Ready`] / [`ConversationStatus::Error`]）
pub(crate) async fn finalize_status(
    service: &AiService,
    conversation_id: Uuid,
    status: ConversationStatus,
) {
    let repo = service.conversation_repo.read().await;
    if let Err(e) = repo
        .update_status(conversation_id, &status.to_string())
        .await
    {
        error!(
            "更新会话状态失败 (id={}, target_status={}): {}",
            conversation_id, status, e
        );
    }
}

/// 将会话异常原因写入 `prompt` 字段
///
/// 该字段语义为"会话异常/状态原因"，在初始化失败时由本函数写入。
/// 若底层仓储返回错误，则**仅记录日志**，不向上冒泡，避免覆盖业务原始错误。
///
/// # 参数
/// - `service`: 当前 [`AiService`] 引用
/// - `conversation_id`: 待更新的会话 ID
/// - `reason`: 异常原因文本（通常来自 `initialize()` 返回的 `Err(String)`）
async fn record_failure_reason(service: &AiService, conversation_id: Uuid, reason: &str) {
    let repo = service.conversation_repo.read().await;
    if let Err(e) = repo.update_prompt(conversation_id, Some(reason)).await {
        error!(
            "写入会话异常原因失败 (id={}, reason={}): {}",
            conversation_id, reason, e
        );
    }
}

/// 会话初始化统一调度入口
///
/// 由 `AiService::create_conversation` 在落库成功后立即调用。执行步骤：
/// 1. 将 `conversation.conversation_type` 字符串解析为 [`ConversationType`] 枚举；
/// 2. 路由到对应的初始化器并调用其 `initialize()`；
/// 3. 成功 → 把会话状态置为 [`ConversationStatus::Ready`]；
///    失败 → 把会话状态置为 [`ConversationStatus::Error`] 并冒泡原始错误。
///
/// # 参数
/// - `service`: 当前 [`AiService`] 引用
/// - `conversation`: 已落库的会话实体
///
/// # 返回
/// - `Ok(())`：初始化及状态收尾均成功
/// - `Err(String)`：中文错误描述（可能来自类型解析或 `initialize()` 自身）
///
/// # 错误场景
/// - `conversation.conversation_type` 不是已知枚举值 → 返回 `"未知的会话类型: {value}"`
/// - 具体类型的 `initialize()` 返回错误 → 会话状态被置为 `Error` 后，错误原样冒泡
pub(crate) async fn run_conversation_initializer(
    service: &AiService,
    conversation: &AiConversation,
) -> Result<(), String> {
    let ct: ConversationType = conversation
        .conversation_type
        .parse()
        .map_err(|_| format!("未知的会话类型: {}", conversation.conversation_type))?;

    let initializer = resolve_initializer(ct);

    match initializer.initialize(service, conversation).await {
        Ok(()) => {
            finalize_status(service, conversation.id, ConversationStatus::Ready).await;
            Ok(())
        }
        Err(e) => {
            record_failure_reason(service, conversation.id, &e).await;
            finalize_status(service, conversation.id, ConversationStatus::Error).await;
            Err(e)
        }
    }
}
