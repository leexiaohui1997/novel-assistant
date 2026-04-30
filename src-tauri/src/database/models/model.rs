use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

/// AI 模型实体
/// 对应数据库中的 ai_models 表
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
#[serde(rename_all = "camelCase")]
pub struct Model {
    pub id: Uuid,
    pub provider_id: Uuid,
    pub model_id: String,
    pub alias: String,
    pub is_default: bool,
    pub is_enabled: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// 带供应商信息的模型 DTO
/// 用于列表展示场景，JOIN ai_providers 带出供应商名称
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
#[serde(rename_all = "camelCase")]
pub struct ModelWithProvider {
    pub id: Uuid,
    pub provider_id: Uuid,
    pub provider_name: String,
    pub model_id: String,
    pub alias: String,
    pub is_default: bool,
    pub is_enabled: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// 新增模型项（单条）
/// 由前端勾选并编辑名称后提交
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NewModelItem {
    pub model_id: String,
    pub alias: String,
}

/// 批量添加模型请求参数
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BatchAddModels {
    pub provider_id: Uuid,
    pub models: Vec<NewModelItem>,
}
