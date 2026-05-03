// 生成角色建议 Action 实现

use async_trait::async_trait;
use serde::Deserialize;
use uuid::Uuid;

use crate::ai::actions::{ActionContext, ActionError, ActionHandler, ActionResponse};

/// 生成角色输入参数
#[derive(Debug, Deserialize)]
pub struct GenerateCharacterInput {
    /// 小说 ID
    pub novel_id: String,
}

/// AI 返回的角色数据结构
#[derive(Debug, Deserialize, serde::Serialize)]
pub struct GeneratedCharacter {
    /// 角色名称
    pub name: String,

    /// 性别：male, female, other, unknown
    pub gender: String,

    /// 角色背景
    pub background: String,

    /// 外貌描写（可选）
    #[serde(default)]
    pub appearance: Option<String>,

    /// 性格特征（可选）
    #[serde(default)]
    pub personality: Option<String>,

    /// 其他描述（可选）
    #[serde(default)]
    pub additional_info: Option<String>,
}

/// 生成角色建议 Action
///
/// 根据小说信息和已有角色，AI 生成新角色建议
pub struct GenerateCharacterAction;

#[async_trait]
impl ActionHandler for GenerateCharacterAction {
    fn name(&self) -> &str {
        "generate_character"
    }

    fn description(&self) -> &str {
        "根据小说上下文信息生成新角色建议"
    }

    async fn handle(&self, ctx: ActionContext) -> Result<ActionResponse, ActionError> {
        // 1. 反序列化输入参数
        let input: GenerateCharacterInput = serde_json::from_value(ctx.input)
            .map_err(|e| ActionError::InvalidInput(format!("参数格式错误: {}", e)))?;

        // 2. 解析小说 ID
        let novel_uuid = Uuid::parse_str(&input.novel_id)
            .map_err(|e| ActionError::InvalidInput(format!("小说 ID 格式错误: {}", e)))?;

        // 3. 查询小说基本信息
        let novel_repo = ctx.novel_repo.read().await;
        let novel = novel_repo
            .find_by_id(
                novel_uuid,
                &crate::database::repositories::QueryOptions { with_tags: true },
            )
            .await
            .map_err(|e| ActionError::ExecutionFailed(format!("查询小说信息失败: {}", e)))?;
        drop(novel_repo);

        // 4. 查询已有角色列表
        let character_repo = ctx.character_repo.read().await;
        let characters = character_repo
            .find_by_novel_id(&novel_uuid)
            .await
            .map_err(|e| ActionError::ExecutionFailed(format!("查询角色列表失败: {}", e)))?;
        drop(character_repo);

        // 5. 构建提示词上下文
        use crate::ai::prompts::{CharacterInfo, GenerateCharacterContext, PromptTemplates};

        // 转换频道名称
        let channel_name = if novel.novel.target_reader == "male" {
            "男频".to_string()
        } else if novel.novel.target_reader == "female" {
            "女频".to_string()
        } else {
            "未知".to_string()
        };

        // 构建标签字符串
        let tags = if !novel.tags.is_empty() {
            Some(
                novel
                    .tags
                    .iter()
                    .map(|t| t.name.as_str())
                    .collect::<Vec<_>>()
                    .join("、"),
            )
        } else {
            None
        };

        // 构建已有角色列表
        let existing_characters = if !characters.is_empty() {
            Some(
                characters
                    .iter()
                    .map(|c| CharacterInfo {
                        name: c.name.clone(),
                        gender: format!("{:?}", c.gender).to_lowercase(),
                        background: c.background.clone(),
                    })
                    .collect(),
            )
        } else {
            None
        };

        let prompt_context = GenerateCharacterContext {
            title: novel.novel.title,
            channel_name,
            tags,
            introduction: Some(novel.novel.description),
            existing_characters,
        };

        // 6. 使用 Tera 模板渲染提示词
        let templates = PromptTemplates::new()
            .map_err(|e| ActionError::ExecutionFailed(format!("加载模板失败: {}", e)))?;

        let prompt = templates
            .render_generate_character(&prompt_context)
            .map_err(|e| ActionError::ExecutionFailed(format!("渲染提示词失败: {}", e)))?;

        // 7. 调用 AI 服务生成角色
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

        // 8. 解析 AI 返回的 JSON
        let content = ai_response.content.trim();

        // 尝试去除可能的 Markdown 代码块标记
        let json_str = content
            .trim_start_matches("```json")
            .trim_start_matches("```")
            .trim_end_matches("```")
            .trim();

        let generated_character: GeneratedCharacter =
            serde_json::from_str(json_str).map_err(|e| {
                ActionError::ExecutionFailed(format!(
                    "解析 AI 返回的 JSON 失败: {}。原始内容: {}",
                    e, content
                ))
            })?;

        // 9. 验证性别字段
        match generated_character.gender.as_str() {
            "male" | "female" | "other" | "unknown" => {}
            _ => {
                return Err(ActionError::ExecutionFailed(format!(
                    "无效的性别值: {}。必须是 male、female、other 或 unknown",
                    generated_character.gender
                )))
            }
        }

        // 10. 返回生成的角色数据
        Ok(ActionResponse {
            data: serde_json::to_value(generated_character)
                .map_err(|e| ActionError::ExecutionFailed(format!("序列化角色数据失败: {}", e)))?,
            metadata: std::collections::HashMap::new(),
        })
    }
}
