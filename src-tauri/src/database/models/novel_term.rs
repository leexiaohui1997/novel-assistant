use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::Type;
use uuid::Uuid;

use crate::string_enum;

// 使用宏定义小说名词类型枚举
string_enum! {
    /// 小说名词类型枚举
    ///
    /// 持久化与序列化均使用 snake_case 字面量（如 `Character → "character"`），
    /// 与现有 `Gender` / `CharacterType` 风格一致。
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Type)]
    #[serde(rename_all = "snake_case")]
    #[sqlx(type_name = "TEXT", rename_all = "snake_case")]
    pub enum TermType {
        Character, // 人物类
        Location,  // 地域 & 场景类
        Faction,   // 势力 & 组织类
        Item,      // 物品 & 道具类
        Skill,     // 功法 & 技能 & 术法类
        Race,      // 生灵 & 种族类
        Era,       // 时间 & 纪元 & 设定类
        Culture,   // 文化 & 典籍 & 规则类
        Emotion,   // 情感 & 专属代称类
    }
}

impl TermType {
    /// 返回当前枚举对应的中文展示标签
    ///
    /// 通过 `match` 穷举保证后续新增成员时编译期即可发现遗漏。
    pub fn label(&self) -> &'static str {
        match self {
            TermType::Character => "人物",
            TermType::Location => "地域 & 场景",
            TermType::Faction => "势力 & 组织",
            TermType::Item => "物品 & 道具",
            TermType::Skill => "功法 & 技能 & 术法",
            TermType::Race => "生灵 & 种族",
            TermType::Era => "时间 & 纪元 & 设定",
            TermType::Culture => "文化 & 典籍 & 规则",
            TermType::Emotion => "情感 & 专属代称",
        }
    }
}

/// 所有名词类型枚举成员列表（顺序与对外展示顺序一致）
pub const ALL_TERM_TYPES: &[TermType] = &[
    TermType::Character,
    TermType::Location,
    TermType::Faction,
    TermType::Item,
    TermType::Skill,
    TermType::Race,
    TermType::Era,
    TermType::Culture,
    TermType::Emotion,
];

/// 小说名词实体模型
///
/// 对应数据库中的 `novel_terms` 表，存储小说中出现的人物、地域、势力等"名词"。
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct NovelTerm {
    pub id: Uuid,
    pub novel_id: Uuid,
    pub term_type: TermType,
    pub name: String,
    pub description: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
