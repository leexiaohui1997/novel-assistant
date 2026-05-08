use std::future::Future;
use std::pin::Pin;

/// AI 工具统一接口
///
/// 所有内置或自定义的 AI 工具都必须实现此 trait。
pub trait AiTool: Send + Sync {
    /// 工具的唯一标识符（例如：search_novel）
    fn name(&self) -> &str;

    /// 工具的自然语言描述，用于指导 AI 何时调用该工具
    fn description(&self) -> &str;

    /// 工具的参数定义（JSON Schema 格式）
    ///
    /// AI 模型将根据此 Schema 生成符合规范的参数对象。
    fn parameters_schema(&self) -> serde_json::Value;

    /// 执行工具逻辑
    ///
    /// # 参数
    /// - `args`: 由 AI 生成的、符合 `parameters_schema` 定义的 JSON 对象
    ///
    /// # 返回
    /// - `Ok(String)`: 工具执行结果（通常是一段文本或 JSON 字符串）
    /// - `Err(String)`: 错误描述
    fn execute(
        &self,
        args: serde_json::Value,
    ) -> Pin<Box<dyn Future<Output = Result<String, String>> + Send>>;
}
