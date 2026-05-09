//! 推荐模型推导工具函数
//!
//! 规则：
//! 1. 优先取 `ai_call_logs` 中最近一次调用所对应的模型（要求该模型仍 `is_enabled = true`）。
//! 2. 若最近记录不存在 / 对应模型已被删除或禁用，回退到按 `is_enabled = true` 过滤、
//!    `is_default DESC, created_at ASC` 排序取第一条模型。
//! 3. 以上均无命中时返回错误。

use crate::database::models::model::Model;
use crate::database::repositories::{AiCallLogRepository, ModelRepository};

/// 查询"推荐模型"，详见模块级文档。
///
/// # 参数
/// - `ai_call_log_repo`: AI 调用日志仓储
/// - `model_repo`: 模型仓储
///
/// # 返回
/// - `Ok(Model)`: 推荐模型实体
/// - `Err(String)`: 查询过程中的错误或无可用模型
pub async fn find_recommended_model(
    ai_call_log_repo: &(dyn AiCallLogRepository + Send + Sync),
    model_repo: &(dyn ModelRepository + Send + Sync),
) -> Result<Model, String> {
    if let Some(model) = try_pick_from_latest_log(ai_call_log_repo, model_repo).await? {
        return Ok(model);
    }

    fallback_pick_default(model_repo).await
}

/// 尝试从最近一条调用记录中挑选推荐模型。
///
/// 返回 `Ok(None)` 表示"未命中，请走回退分支"。
async fn try_pick_from_latest_log(
    ai_call_log_repo: &(dyn AiCallLogRepository + Send + Sync),
    model_repo: &(dyn ModelRepository + Send + Sync),
) -> Result<Option<Model>, String> {
    let latest = ai_call_log_repo
        .find_latest()
        .await
        .map_err(|e| format!("查询推荐模型失败: {}", e))?;

    let Some(log) = latest else {
        return Ok(None);
    };

    let model = model_repo
        .find_enabled_by_id(log.model_id)
        .await
        .map_err(|e| format!("查询推荐模型失败: {}", e))?;

    Ok(model)
}

/// 回退分支：按 `is_default DESC, created_at ASC` 取第一条启用模型。
async fn fallback_pick_default(
    model_repo: &(dyn ModelRepository + Send + Sync),
) -> Result<Model, String> {
    model_repo
        .find_first_enabled_preferring_default()
        .await
        .map_err(|e| format!("查询推荐模型失败: {}", e))?
        .ok_or_else(|| "未找到可用的推荐模型".to_string())
}
