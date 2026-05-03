use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::Type;
use uuid::Uuid;

use crate::string_enum;

// 使用宏定义性别枚举
string_enum! {
    /// 角色性别枚举
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Type)]
    #[serde(rename_all = "snake_case")]
    #[sqlx(type_name = "TEXT", rename_all = "snake_case")]
    pub enum Gender {
        Male,     // 男
        Female,   // 女
        Other,    // 其他
        Unknown,  // 未知
    }
}

/// 角色实体模型
/// 对应数据库中的 characters 表，存储小说中的角色信息
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct Character {
    pub id: Uuid,
    pub novel_id: Uuid,
    pub name: String,
    pub gender: Gender,
    pub background: Option<String>,
    pub appearance: Option<String>,
    pub personality: Option<String>,
    pub additional_info: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
