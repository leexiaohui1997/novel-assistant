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
