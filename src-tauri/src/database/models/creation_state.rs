use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

/// 创作状态实体模型
/// 对应数据库中的 creation_states 表
/// 当前阶段仅存储 novel_id 作为标识，后续可根据需要添加具体状态字段
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct CreationState {
    pub id: i64,
    pub novel_id: Uuid,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
