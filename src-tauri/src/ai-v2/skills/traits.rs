use crate::ai_v2::template::{TemplateError, TemplateManager};

/// AI 技能统一接口
///
/// 所有内置或自定义的 AI 技能都必须实现此 trait。
pub trait AiSkill: Send + Sync {
    /// 技能的唯一标识符（例如：outline_to_chapter）
    fn name(&self) -> &str;

    /// 技能的自然语言描述，用于指导 AI 何时装载该技能
    fn description(&self) -> &str;

    /// 技能的参数定义（JSON Schema 格式）
    ///
    /// AI 模型将根据此 Schema 生成符合规范的参数对象作为渲染上下文。
    fn params_schema(&self) -> serde_json::Value;

    /// 技能使用的模板 ID
    ///
    /// 供"通用渲染回退"路径使用：多轮驱动器在没有强类型 `TemplateInstance`
    /// 的情况下，会以此 ID + 参数 JSON 调用 [`TemplateManager::render`]。
    ///
    /// 如果该技能同时绑定了 [`crate::ai_v2::template::TemplateInstance`] 实现，
    /// 建议返回值与实例的 `TEMPLATE_ID` 一致，以便两种路径指向同一个模板。
    fn template_id(&self) -> &str;

    /// 使用强类型 [`crate::ai_v2::template::TemplateInstance`] 渲染（可选）
    ///
    /// - 默认实现返回 `None`，表示该技能未绑定实例，由多轮驱动器走通用渲染回退。
    /// - 绑定了实例的技能应在实现内部调用 `instance.render(manager, params.clone())`
    ///   并以 `Some(Ok(..))` / `Some(Err(..))` 包装返回（内部包含参数校验）。
    ///
    /// 约束：[`crate::ai_v2::template::TemplateInstance`] 本身不做动态分发改造，
    /// 是否接入动态分发由每个技能实现自行决定，以保留关联常量 / 关联类型设计。
    fn render_with_instance(
        &self,
        _manager: &TemplateManager,
        _params: &serde_json::Value,
    ) -> Option<Result<String, TemplateError>> {
        None
    }
}
