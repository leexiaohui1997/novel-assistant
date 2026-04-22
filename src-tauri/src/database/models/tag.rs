use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::Type;

use crate::string_enum;

// 使用宏定义标签类型枚举
string_enum! {
    /// 标签类型枚举
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Type)]
    #[serde(rename_all = "snake_case")]
    #[sqlx(type_name = "TEXT", rename_all = "snake_case")]
    pub enum TagType {
        MainCategory, // 主分类
        Theme,        // 主题
        Character,    // 角色
        Plot,         // 情节
    }
}

// 使用宏定义目标读者枚举
string_enum! {
    /// 目标读者枚举
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Type)]
    #[serde(rename_all = "snake_case")]
    #[sqlx(type_name = "TEXT", rename_all = "snake_case")]
    pub enum TargetAudience {
        Male,   // 男频
        Female, // 女频
        Both,   // 通用
    }
}

/// 标签实体模型
/// 对应数据库中的 tags 表，存储系统内所有的作品标签
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct Tag {
    pub id: i64,
    pub name: String,
    pub tag_type: TagType,
    pub target_audience: TargetAudience,
    pub description: Option<String>,
    pub created_at: DateTime<Utc>,
}
