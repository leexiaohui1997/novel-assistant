// 优化角色信息 Action 实现

use async_trait::async_trait;
use serde::Deserialize;
use uuid::Uuid;

use crate::ai::actions::{
    context_helpers, ActionContext, ActionError, ActionHandler, ActionResponse,
};
use crate::database::models::character::CharacterType;

/// 优化角色输入参数
#[derive(Debug, Deserialize)]
pub struct OptimizeCharacterInput {
    /// 小说 ID
    pub novel_id: String,

    /// 角色信息
    pub character: CharacterInput,

    /// 需要优化的字段列表
    pub optimize_fields: Vec<String>,

    /// 用户优化意见（可选）
    #[serde(default)]
    pub user_feedback: Option<String>,
}

/// 角色输入信息
#[derive(Debug, Deserialize)]
pub struct CharacterInput {
    /// 角色名称
    pub name: String,

    /// 性别
    pub gender: String,

    /// 角色类型（可选，snake_case 字符串，如 "protagonist"）
    #[serde(default)]
    pub character_type: Option<String>,

    /// 背景（可选）
    #[serde(default)]
    pub background: Option<String>,

    /// 外貌（可选）
    #[serde(default)]
    pub appearance: Option<String>,

    /// 性格（可选）
    #[serde(default)]
    pub personality: Option<String>,

    /// 其它描述（可选）
    #[serde(default)]
    pub additional_info: Option<String>,
}

/// AI 返回的优化结果（可能只包含部分字段）
#[derive(Debug, Deserialize, serde::Serialize)]
pub struct OptimizedCharacter {
    /// 角色名称（可选，如果需要优化）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,

    /// 性别（可选，如果需要优化）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gender: Option<String>,

    /// 角色类型（可选，如果需要优化）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub character_type: Option<String>,

    /// 背景（可选，如果需要优化）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub background: Option<String>,

    /// 外貌（可选，如果需要优化）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub appearance: Option<String>,

    /// 性格（可选，如果需要优化）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub personality: Option<String>,

    /// 其它描述（可选，如果需要优化）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub additional_info: Option<String>,
}

/// 将 snake_case 字符串解析为 `CharacterType` 枚举
///
/// 使用 serde_json 进行反序列化，严格匹配 5 个合法枚举值。
/// 用途：兼作白名单校验 + 枚举转换。
fn parse_character_type(value: &str) -> Result<CharacterType, String> {
    serde_json::from_value::<CharacterType>(serde_json::Value::String(value.to_string()))
        .map_err(|_| format!("无效的角色类型值: {}", value))
}

/// 将输入的 snake_case 字符串转为中文标签（例如 "supporting" -> "配角"）
///
/// 若字符串非法，返回 `None`（因为此处是「入参参考」而非硬校验；
/// 硬校验针对 AI 返回值执行）。
fn to_character_type_label(value: &str) -> Option<String> {
    parse_character_type(value)
        .ok()
        .map(context_helpers::character_type_label)
}

/// 校验 AI 返回的 `character_type` 值是否属于合法枚举值
fn validate_character_type(value: &str) -> Result<(), ActionError> {
    parse_character_type(value)
        .map(|_| ())
        .map_err(ActionError::ExecutionFailed)
}

/// 优化角色信息 Action
///
/// 根据小说背景和用户意见，优化角色的指定字段
pub struct OptimizeCharacterAction;

#[async_trait]
impl ActionHandler for OptimizeCharacterAction {
    fn name(&self) -> &str {
        "optimize_character"
    }

    fn description(&self) -> &str {
        "根据小说上下文和用户意见优化角色信息"
    }

    async fn handle(&self, ctx: ActionContext) -> Result<ActionResponse, ActionError> {
        // 1. 反序列化输入参数
        let input: OptimizeCharacterInput = serde_json::from_value(ctx.input)
            .map_err(|e| ActionError::InvalidInput(format!("参数格式错误: {}", e)))?;

        // 2. 解析小说 ID
        let novel_uuid = Uuid::parse_str(&input.novel_id)
            .map_err(|e| ActionError::InvalidInput(format!("小说 ID 格式错误: {}", e)))?;

        // 3. 查询小说基本信息（统一通过 context_helpers::fetch_novel_info）
        let novel_info =
            context_helpers::fetch_novel_info(&ctx.novel_repo, &ctx.tag_repo, novel_uuid)
                .await
                .map_err(|e| ActionError::ExecutionFailed(format!("查询小说信息失败: {}", e)))?;

        // 4. 构建提示词上下文
        use crate::ai::prompts::{CharacterDetail, OptimizeCharacterContext, PromptTemplates};

        let prompt_context = OptimizeCharacterContext {
            title: novel_info.title.clone(),
            channel_name: novel_info.channel_name.clone(),
            tags: if novel_info.tags.is_empty() {
                None
            } else {
                Some(novel_info.tags.clone())
            },
            introduction: if novel_info.description.is_empty() {
                None
            } else {
                Some(novel_info.description.clone())
            },
            character: CharacterDetail {
                name: input.character.name,
                gender: input.character.gender,
                character_type: input
                    .character
                    .character_type
                    .as_deref()
                    .and_then(to_character_type_label),
                background: input.character.background,
                appearance: input.character.appearance,
                personality: input.character.personality,
                additional_info: input.character.additional_info,
            },
            optimize_fields: input.optimize_fields,
            user_feedback: input.user_feedback,
            character_type_options: context_helpers::get_character_type_options(),
        };

        // 5. 使用 Tera 模板渲染提示词
        let templates = PromptTemplates::new()
            .map_err(|e| ActionError::ExecutionFailed(format!("加载模板失败: {}", e)))?;

        let prompt = templates
            .render_optimize_character(&prompt_context)
            .map_err(|e| ActionError::ExecutionFailed(format!("渲染提示词失败: {}", e)))?;

        // 6. 调用 AI 服务生成优化结果
        use crate::ai::types::{AiRequestData, Message, MessageRole};

        // 解析 model_id（如果提供了的话）
        let parsed_model_id = ctx.model_id.and_then(|id| Uuid::parse_str(&id).ok());

        let request_data = AiRequestData {
            model_id: parsed_model_id,
            messages: vec![Message {
                role: MessageRole::User,
                content: prompt.clone(),
            }],
        };

        let ai_response = ctx
            .ai_service
            .chat(request_data)
            .await
            .map_err(|e| ActionError::ExecutionFailed(format!("AI 调用失败: {}", e)))?;

        // 7. 解析 AI 返回的 JSON
        let content = ai_response.content.trim();

        // 尝试去除可能的 Markdown 代码块标记
        let json_str = content
            .trim_start_matches("```json")
            .trim_start_matches("```")
            .trim_end_matches("```")
            .trim();

        let optimized_character: OptimizedCharacter =
            serde_json::from_str(json_str).map_err(|e| {
                ActionError::ExecutionFailed(format!(
                    "解析 AI 返回的 JSON 失败: {}。原始内容: {}",
                    e, content
                ))
            })?;

        // 8. 验证性别字段（如果存在）
        if let Some(gender) = &optimized_character.gender {
            match gender.as_str() {
                "male" | "female" | "other" | "unknown" => {}
                _ => {
                    return Err(ActionError::ExecutionFailed(format!(
                        "无效的性别值: {}。必须是 male、female、other 或 unknown",
                        gender
                    )))
                }
            }
        }

        // 9. 验证角色类型字段（如果存在）
        if let Some(ct) = &optimized_character.character_type {
            validate_character_type(ct)?;
        }

        // 10. 返回优化后的角色数据
        Ok(ActionResponse {
            data: serde_json::to_value(optimized_character)
                .map_err(|e| ActionError::ExecutionFailed(format!("序列化角色数据失败: {}", e)))?,
            metadata: std::collections::HashMap::new(),
        })
    }
}
