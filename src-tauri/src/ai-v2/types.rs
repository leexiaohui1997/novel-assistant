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
