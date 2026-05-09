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
}
