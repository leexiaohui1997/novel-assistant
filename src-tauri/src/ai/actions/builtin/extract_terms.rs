// 提取名词 Action 实现

use async_trait::async_trait;
use serde::Deserialize;
use uuid::Uuid;

use crate::ai::actions::{ActionContext, ActionError, ActionHandler, ActionResponse};

/// 清理 AI 返回的 JSON 字符串，移除可能的代码块标记
fn clean_json_output(content: &str) -> String {
    let trimmed = content.trim();

    // 移除开头的 ```json 或 ```
    let cleaned = if trimmed.starts_with("```") {
        // 找到第一个换行符后的内容
        if let Some(pos) = trimmed.find('\n') {
            &trimmed[pos + 1..]
        } else {
            trimmed
        }
    } else {
        trimmed
    };

    // 移除结尾的 ```
    let cleaned = cleaned.trim_end();
    if cleaned.ends_with("```") {
        cleaned[..cleaned.len() - 3].trim().to_string()
    } else {
        cleaned.to_string()
    }
}

/// 提取名词输入参数
#[derive(Debug, Deserialize)]
pub struct ExtractTermsInput {
    /// 小说ID
    pub novel_id: String,

    /// 正文内容
    pub content: String,
}

/// 提取名词 Action
///
/// 从小说正文中提取所有出现的名词，并按类型分类
pub struct ExtractTermsAction;

#[async_trait]
impl ActionHandler for ExtractTermsAction {
    fn name(&self) -> &str {
        "extract_terms"
    }

    fn description(&self) -> &str {
        "从小说正文中提取所有出现的名词，并按类型分类"
    }

    async fn handle(&self, ctx: ActionContext) -> Result<ActionResponse, ActionError> {
        // 1. 解析输入参数
        let input: ExtractTermsInput = serde_json::from_value(ctx.input)
            .map_err(|e| ActionError::InvalidInput(format!("参数格式错误: {}", e)))?;

        // 2. 验证参数
        if input.novel_id.trim().is_empty() {
            return Err(ActionError::InvalidInput("小说ID不能为空".to_string()));
        }
        if input.content.trim().is_empty() {
            return Err(ActionError::InvalidInput("正文内容不能为空".to_string()));
        }

        // 2. 解析小说ID
        let novel_uuid = Uuid::parse_str(&input.novel_id)
            .map_err(|e| ActionError::InvalidInput(format!("小说ID格式错误: {}", e)))?;

        // 3. 构建现有名词的 Markdown 表示（复用 fragments 函数）
        use crate::ai::prompts::fragments::render_novel_terms_fragment;

        let term_repo = ctx.novel_term_repo.read().await;
        let existing_terms_md = render_novel_terms_fragment(
            novel_uuid,
            true, // 显示 ID，方便 AI 匹配已有名词
            term_repo.as_ref(),
            ctx.templates_root.as_path(),
        )
        .await
        .map_err(|e| ActionError::ExecutionFailed(format!("生成现有名词列表失败: {}", e)))?;
        drop(term_repo);

        // 4. 构建可用名词类型列表
        use crate::database::models::novel_term::ALL_TERM_TYPES;

        let types_list = ALL_TERM_TYPES
            .iter()
            .map(|t| format!("- {} ({})", t.label(), t))
            .collect::<Vec<_>>()
            .join("\n");

        // 5. 渲染提示词模板
        use crate::ai::prompts::{ExtractTermsContext, PromptTemplates};

        let prompt_context = ExtractTermsContext {
            existing_terms_md,
            available_types: types_list,
            content: input.content.clone(),
        };

        let templates = PromptTemplates::new(ctx.templates_root.as_path())
            .map_err(|e| ActionError::ExecutionFailed(format!("加载模板失败: {}", e)))?;

        let prompt = templates
            .render_extract_terms(&prompt_context)
            .map_err(|e| ActionError::ExecutionFailed(format!("渲染提示词失败: {}", e)))?;

        // 6. 调用 AI 服务
        use crate::ai::types::{AiRequestData, Message, MessageRole};

        let parsed_model_id = ctx.model_id.and_then(|id| uuid::Uuid::parse_str(&id).ok());

        let request_data = AiRequestData {
            model_id: parsed_model_id,
            messages: vec![Message {
                role: MessageRole::User,
                content: prompt,
            }],
        };

        let ai_response = ctx
            .ai_service
            .chat(request_data)
            .await
            .map_err(|e| ActionError::ExecutionFailed(format!("AI 调用失败: {}", e)))?;

        // 7. 清理并解析 AI 返回的 JSON
        let ai_content = &ai_response.content;
        let cleaned_content = clean_json_output(ai_content);

        let terms_result: serde_json::Value = serde_json::from_str(&cleaned_content)
            .map_err(|e| ActionError::ExecutionFailed(format!("解析 AI 返回结果失败: {}", e)))?;

        // 验证 JSON 结构
        if !terms_result.is_object() {
            return Err(ActionError::ExecutionFailed(
                "AI 返回结果不是有效的 JSON 对象".to_string(),
            ));
        }

        if let Some(terms) = terms_result.get("terms") {
            if !terms.is_array() {
                return Err(ActionError::ExecutionFailed(
                    "AI 返回结果中 terms 字段不是数组".to_string(),
                ));
            }
        } else {
            return Err(ActionError::ExecutionFailed(
                "AI 返回结果中缺少 terms 字段".to_string(),
            ));
        }

        // 8. 返回结果
        Ok(ActionResponse {
            data: terms_result,
            metadata: std::collections::HashMap::new(),
        })
    }
}
