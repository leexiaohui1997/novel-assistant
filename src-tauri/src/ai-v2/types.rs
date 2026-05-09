use serde::{Deserialize, Serialize};

use crate::string_enum;

string_enum! {
    /// 会话类型
    ///
    /// 用于区分会话所属的业务场景，序列化到数据库与前端时均使用枚举成员名（如 `"Default"`）。
    ///
    /// 成员说明：
    /// - `Default`：默认（通用会话）
    /// - `NovelProfile`：小说资料
    /// - `NovelWorkbench`：小说工作台
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
    pub enum ConversationType {
        Default,
        NovelProfile,
        NovelWorkbench,
    }
}

impl Default for ConversationType {
    /// 默认会话类型为 `Default`
    fn default() -> Self {
        ConversationType::Default
    }
}

string_enum! {
    /// 会话状态
    ///
    /// 用于描述一次会话在业务流程中的生命周期阶段，序列化到数据库与前端时
    /// 均使用枚举成员名（如 `"Init"`、`"Computing"`），保持与 `ConversationType` 风格一致。
    ///
    /// 成员说明：
    /// - `Init`：初始化（默认值，会话刚创建，尚未进入可交互状态）
    /// - `Ready`：已就绪（可正常进行用户交互）
    /// - `Computing`：计算中（正在调用模型 / 执行任务）
    /// - `WaitingNextTurn`：等待下一轮次中（多轮驱动器两轮之间的中间态）
    /// - `Completed`：已完成（本轮业务目标已达成）
    /// - `Terminated`：已终止（被用户主动终止）
    /// - `Error`：异常（发生错误且未恢复）
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
    pub enum ConversationStatus {
        Init,
        Ready,
        Computing,
        WaitingNextTurn,
        Completed,
        Terminated,
        Error,
    }
}

impl Default for ConversationStatus {
    /// 默认会话状态为 `Init`
    fn default() -> Self {
        ConversationStatus::Init
    }
}

string_enum! {
    lower_case;
    /// 会话消息类型
    ///
    /// 字面量与主流大模型 API 的 `role` 字段对齐：`"system"` / `"user"` / `"assistant"`。
    /// 序列化、反序列化、`Display`、`FromStr` 全部统一为全小写。
    ///
    /// 成员说明：
    /// - `System`：系统提示（通常作为会话开头的角色 / 设定）
    /// - `User`：用户发言
    /// - `Assistant`：AI 助手回复
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum MessageType {
        System,
        User,
        Assistant,
    }
}
