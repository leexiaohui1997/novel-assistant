use async_trait::async_trait;
use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::database::{
    error::DbError,
    models::ai_conversation::{AiConversation, CreateAiConversation},
};

/// AI 会话 Repository Trait
///
/// 定义 `ai_conversations` 表的持久化操作，供业务层（如 `AiService`）依赖注入使用。
#[async_trait]
pub trait AiConversationRepository: Send + Sync {
    /// 创建一条会话记录
    ///
    /// # 参数
    /// - `data`: 会话创建 DTO
    ///
    /// # 返回
    /// - `Ok(AiConversation)`: 新建的完整会话实体（含数据库生成的时间字段）
    /// - `Err(DbError)`: 数据库错误
    async fn create(&self, data: CreateAiConversation) -> Result<AiConversation, DbError>;

    /// 按主键查询一条会话
    ///
    /// # 参数
    /// - `id`: 会话 ID
    ///
    /// # 返回
    /// - `Ok(Some(AiConversation))`: 命中
    /// - `Ok(None)`: 未命中
    /// - `Err(DbError)`: 数据库错误
    async fn find_by_id(&self, id: Uuid) -> Result<Option<AiConversation>, DbError>;

    /// 更新会话的 `last_message_at` 时间戳
    ///
    /// 通常由 Service 层在插入一条新消息后立即调用，用于让"按最后一条消息时间排序"的
    /// 会话列表实时更新。
    ///
    /// # 参数
    /// - `conversation_id`: 会话 ID
    /// - `at`: 要写入的时间戳（UTC）
    ///
    /// # 返回
    /// - `Ok(())`: 更新成功（即便目标会话不存在也视作成功，影响行数为 0）
    /// - `Err(DbError)`: 数据库错误
    async fn touch_last_message_at(
        &self,
        conversation_id: Uuid,
        at: DateTime<Utc>,
    ) -> Result<(), DbError>;

    /// 更新会话的 `status` 字段
    ///
    /// # 参数
    /// - `id`: 会话 ID
    /// - `status`: 目标状态字符串（由枚举 `.to_string()` 得到，例如 `"Ready"` / `"Error"`）
    ///
    /// # 返回
    /// - `Ok(())`: 更新成功（即便目标会话不存在也视作成功，影响行数为 0）
    /// - `Err(DbError)`: 数据库错误
    async fn update_status(&self, id: Uuid, status: &str) -> Result<(), DbError>;

    /// 更新会话的 `prompt` 字段（会话异常/状态原因）
    ///
    /// 字段语义为"会话异常原因"，通常在初始化失败等场景下，由业务层写入中文错误描述。
    /// 允许写入 `NULL` 以清空已有原因。
    ///
    /// # 参数
    /// - `id`: 会话 ID
    /// - `prompt`: 目标原因文本；`None` 表示写入 `NULL`
    ///
    /// # 返回
    /// - `Ok(())`: 更新成功（即便目标会话不存在也视作成功，影响行数为 0）
    /// - `Err(DbError)`: 数据库错误
    async fn update_prompt(&self, id: Uuid, prompt: Option<&str>) -> Result<(), DbError>;

    /// 原子更新会话的 `status` 与 `prompt` 字段
    ///
    /// 以**单条 UPDATE** 同时写入两字段，避免出现"状态已改但原因未写 / 原因已写但状态未改"
    /// 的中间态。典型用途：
    /// - 驱动器推进会话为 `Computing`，同时清空历史异常原因（`prompt = NULL`）
    /// - 驱动器异常收尾，把 `status` 置为 `Error`、`prompt` 写入中文异常原因
    ///
    /// 该方法与 [`Self::update_status`] / [`Self::update_prompt`] 职责不同，
    /// 后两者仍保留以满足单字段变更场景。
    ///
    /// # 参数
    /// - `id`: 会话 ID
    /// - `status`: 目标状态字符串（由枚举 `.to_string()` 得到）
    /// - `prompt`: 目标原因文本；`None` 表示写入 `NULL`
    ///
    /// # 返回
    /// - `Ok(())`: 更新成功（即便目标会话不存在也视作成功，影响行数为 0）
    /// - `Err(DbError)`: 数据库错误
    async fn update_status_and_prompt(
        &self,
        id: Uuid,
        status: &str,
        prompt: Option<&str>,
    ) -> Result<(), DbError>;
}
