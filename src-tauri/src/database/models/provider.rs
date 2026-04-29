use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

/// 供应商实体模型
/// 对应数据库中的 ai_providers 表，用于查询和展示供应商信息
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
#[serde(rename_all = "camelCase")]
pub struct Provider {
    pub id: Uuid,
    pub name: String,
    pub base_url: String,
    pub api_key: Option<String>,
    pub model_fetch_type: String,
    pub is_enabled: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// 创建供应商请求参数
/// 用于接收前端创建新供应商时提交的表单数据
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NewProvider {
    pub name: String,
    pub base_url: String,
    pub api_key: Option<String>,
    pub model_fetch_type: Option<String>,
    pub is_enabled: Option<bool>,
}

/// 更新供应商请求参数
/// 用于接收前端编辑供应商时提交的表单数据
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateProvider {
    pub name: String,
    pub base_url: String,
    pub api_key: Option<String>,
    pub model_fetch_type: Option<String>,
    pub is_enabled: Option<bool>,
}
