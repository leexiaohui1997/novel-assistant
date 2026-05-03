// 生成作品简介 Action 实现

use async_trait::async_trait;
use serde::Deserialize;

use crate::ai::actions::{ActionContext, ActionError, ActionHandler, ActionResponse};

/// 生成简介输入参数
#[derive(Debug, Deserialize)]
pub struct GenerateIntroductionInput {
    /// 标题（可选）
    pub title: Option<String>,

    /// 频道（可选，male 或 female）
    pub channel: Option<String>,

    /// 已选择的标签 ID 列表（可选）
    pub tag_ids: Option<Vec<i64>>,

    /// 用户意见（可选）
    #[serde(default)]
    pub user_feedback: Option<String>,
}

/// 生成作品简介 Action
///
/// 根据小说上下文信息，AI 生成合适的作品简介
pub struct GenerateIntroductionAction;

#[async_trait]
impl ActionHandler for GenerateIntroductionAction {
    fn name(&self) -> &str {
        "generate_introduction"
    }

    fn description(&self) -> &str {
        "根据小说上下文信息生成作品简介"
    }

    async fn handle(&self, ctx: ActionContext) -> Result<ActionResponse, ActionError> {
        // 1. 反序列化输入参数
        let input: GenerateIntroductionInput = serde_json::from_value(ctx.input)
            .map_err(|e| ActionError::InvalidInput(format!("参数格式错误: {}", e)))?;

        // 2. 查询已选标签信息（如果提供了 tag_ids）
        let selected_tags_info = if let Some(tag_ids) = &input.tag_ids {
            if !tag_ids.is_empty() {
                let tag_repo = ctx.tag_repo.read().await;
                let selected_tags = tag_repo.find_by_ids(tag_ids).await.map_err(|e| {
                    ActionError::ExecutionFailed(format!("查询已选标签失败: {}", e))
                })?;
                drop(tag_repo);

                if !selected_tags.is_empty() {
                    let tags_detail = selected_tags
                        .iter()
                        .map(|t| t.name.as_str())
                        .collect::<Vec<_>>()
                        .join("、");
                    Some(tags_detail)
                } else {
                    None
                }
            } else {
                None
            }
        } else {
            None
        };

        // 3. 使用 Tera 模板渲染提示词
        use crate::ai::prompts::{GenerateIntroductionContext, PromptTemplates};

        let templates = PromptTemplates::new()
            .map_err(|e| ActionError::ExecutionFailed(format!("加载模板失败: {}", e)))?;

        let channel_name = input.channel.as_ref().map(|c| {
            if c == "male" {
                "男频"
            } else if c == "female" {
                "女频"
            } else {
                "未知"
            }
        });

        let prompt_context = GenerateIntroductionContext {
            title: input.title.clone(),
            channel_name: channel_name.map(|s| s.to_string()),
            selected_tags: selected_tags_info,
            user_feedback: input.user_feedback,
        };

        let prompt = templates
            .render_generate_introduction(&prompt_context)
            .map_err(|e| ActionError::ExecutionFailed(format!("渲染提示词失败: {}", e)))?;

        // 4. 调用 AI 服务生成简介
        use crate::ai::types::{AiRequestData, Message, MessageRole};

        // 解析 model_id（如果提供了的话）
        let parsed_model_id = ctx.model_id.and_then(|id| uuid::Uuid::parse_str(&id).ok());

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

        // 5. 返回生成的简介文本（去除前后空格）
        Ok(ActionResponse {
            data: serde_json::json!({
                "introduction": ai_response.content.trim()
            }),
            metadata: std::collections::HashMap::new(),
        })
    }
}
