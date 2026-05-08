use std::sync::Arc;
use uuid::Uuid;

use crate::ai_v2::initializer::resolve_initializer;
use crate::ai_v2::types::ConversationType;
use crate::ai_v2::TemplateManager;
use crate::database::repositories::AiConversationRepository;

use super::registry::ToolRegistry;
use super::traits::AiTool;

/// 根据会话 ID 生成工具系统提示词
///
/// # 参数
/// - `conversation_id`: 会话 ID
/// - `conversation_repo`: 会话仓储，用于获取会话类型
/// - `tool_registry`: 工具注册中心
/// - `template_manager`: Tera 模板管理器
///
/// # 返回
/// - 格式化后的工具系统提示词字符串
pub async fn generate_tool_prompt(
    conversation_id: Uuid,
    conversation_repo: &dyn AiConversationRepository,
    tool_registry: &ToolRegistry,
    template_manager: &TemplateManager,
) -> Result<String, String> {
    // 1. 获取会话信息以确定类型
    let conversation = conversation_repo
        .find_by_id(conversation_id)
        .await
        .map_err(|e| format!("查询会话失败: {}", e))?
        .ok_or("会话不存在".to_string())?;

    let ct: ConversationType = conversation
        .conversation_type
        .parse()
        .map_err(|_| format!("未知的会话类型: {}", conversation.conversation_type))?;

    // 2. 获取初始化器配置
    let initializer = resolve_initializer(ct);
    if !initializer.is_tool_enabled() {
        return Ok(String::new());
    }

    // 3. 从注册中心过滤工具
    let allowed_names = initializer.allowed_tools();

    let tools: Vec<Arc<dyn AiTool>> = tool_registry
        .iter()
        .filter(|t| match &allowed_names {
            Some(names) => names.contains(&t.name().to_string()),
            None => true,
        })
        .cloned()
        .collect();

    if tools.is_empty() {
        return Ok(String::new());
    }

    // 4. 准备模板上下文
    let tool_infos: Vec<serde_json::Value> = tools
        .iter()
        .map(|t| {
            serde_json::json!({
                "name": t.name(),
                "description": t.description(),
                "schema": t.parameters_schema()
            })
        })
        .collect();
    let context_data = serde_json::json!({
        "tools": tool_infos
    });

    // 5. 渲染模板
    template_manager
        .render("prompts/tools.tera", &context_data)
        .map_err(|e| format!("渲染工具提示词失败: {}", e))
}
