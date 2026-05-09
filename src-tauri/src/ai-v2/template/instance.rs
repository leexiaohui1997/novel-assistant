//! 模板实例抽象（TemplateInstance trait）
//!
//! 为每个业务模板提供统一的“参数校验 → 渲染”流水线：
//! - 关联常量 `TEMPLATE_ID` 指向 `TemplateManager` 中注册的模板 ID；
//! - 关联类型 `Params` 派生 `Deserialize + Validate + Serialize`，承载强类型校验；
//! - 默认 `validate` / `render` 实现统一流程，业务端通常只需定义 `Params`。

use async_trait::async_trait;
use serde::{de::DeserializeOwned, Serialize};
use serde_json::Value;
use validator::Validate;

use super::error::TemplateError;
use super::manager::TemplateManager;

/// 模板实例 trait
#[async_trait]
pub trait TemplateInstance {
    /// 模板 ID（形如 `prompts/xxx`、`skills/xxx`）
    const TEMPLATE_ID: &'static str;

    /// 强类型参数结构体
    type Params: DeserializeOwned + Validate + Serialize + Send;

    /// 参数校验：反序列化 → validator 约束校验 → 返回强类型参数
    fn validate(&self, data: &Value) -> Result<Self::Params, TemplateError> {
        let params: Self::Params = serde_json::from_value(data.clone())
            .map_err(|e| TemplateError::InvalidContext(e.to_string()))?;
        params.validate().map_err(TemplateError::Validation)?;
        Ok(params)
    }

    /// 渲染（异步）：校验通过后，将 `Params` 序列化为 JSON 对象再交给 `TemplateManager`
    ///
    /// 业务实例如需注入 DB 等 IO 依赖，可覆写本方法。
    async fn render(
        &self,
        manager: &TemplateManager,
        data: Value,
    ) -> Result<String, TemplateError> {
        let params = self.validate(&data)?;
        let ctx = serde_json::to_value(&params)
            .map_err(|e| TemplateError::InvalidContext(e.to_string()))?;
        manager.render(Self::TEMPLATE_ID, &ctx).await
    }
}
