//! 技能系统提示词生成服务
//!
//! 1. 根据会话 ID 定位会话类型；
//! 2. 经 [`crate::ai_v2::initializer::ConversationInitializer`] 做"启用 + 白名单"过滤；
//! 3. 渲染 `prompts/skills.tera` 生成首轮 System 注入文本。

use std::sync::Arc;
use uuid::Uuid;

use crate::ai_v2::initializer::resolve_initializer;
use crate::ai_v2::types::ConversationType;
use crate::ai_v2::TemplateManager;
use crate::database::repositories::AiConversationRepository;

use super::registry::SkillRegistry;
use super::traits::AiSkill;

/// 根据会话 ID 生成技能系统提示词
///
/// # 参数
/// - `conversation_id`: 会话 ID
/// - `conversation_repo`: 会话仓储，用于获取会话类型
/// - `skill_registry`: 技能注册中心
/// - `template_manager`: Tera 模板管理器
///
/// # 返回
/// - 格式化后的技能系统提示词字符串（无技能时返回空串）
pub async fn generate_skill_prompt(
    conversation_id: Uuid,
    conversation_repo: &dyn AiConversationRepository,
    skill_registry: &SkillRegistry,
    template_manager: &TemplateManager,
) -> Result<String, String> {
    let ct = load_conversation_type(conversation_id, conversation_repo).await?;
    let skills = filter_skills(ct, skill_registry);
    if skills.is_empty() {
        return Ok(String::new());
    }
    render_skill_prompt(&skills, template_manager).await
}

/// 读取并解析会话类型
async fn load_conversation_type(
    conversation_id: Uuid,
    conversation_repo: &dyn AiConversationRepository,
) -> Result<ConversationType, String> {
    let conversation = conversation_repo
        .find_by_id(conversation_id)
        .await
        .map_err(|e| format!("查询会话失败: {}", e))?
        .ok_or("会话不存在".to_string())?;

    conversation
        .conversation_type
        .parse()
        .map_err(|_| format!("未知的会话类型: {}", conversation.conversation_type))
}

/// 按初始化器的启用标志 + 白名单过滤注册中心中的技能
fn filter_skills(ct: ConversationType, registry: &SkillRegistry) -> Vec<Arc<dyn AiSkill>> {
    let initializer = resolve_initializer(ct);
    if !initializer.is_skill_enabled() {
        return Vec::new();
    }

    let allowed_names = initializer.allowed_skills();
    registry
        .iter()
        .filter(|s| match &allowed_names {
            Some(names) => names.contains(&s.name().to_string()),
            None => true,
        })
        .cloned()
        .collect()
}

/// 渲染 `prompts/skills.tera`
async fn render_skill_prompt(
    skills: &[Arc<dyn AiSkill>],
    template_manager: &TemplateManager,
) -> Result<String, String> {
    let skill_infos: Vec<serde_json::Value> = skills
        .iter()
        .map(|s| {
            serde_json::json!({
                "name": s.name(),
                "description": s.description(),
                "schema": s.params_schema()
            })
        })
        .collect();
    let context_data = serde_json::json!({
        "skills": skill_infos
    });

    template_manager
        .render("prompts/skills.tera", &context_data)
        .await
        .map_err(|e| format!("渲染技能提示词失败: {}", e))
}
