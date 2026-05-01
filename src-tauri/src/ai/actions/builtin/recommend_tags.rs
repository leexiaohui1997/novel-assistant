// 推荐标签 Action 实现

use async_trait::async_trait;
use lazy_static::lazy_static;
use regex::Regex;
use serde::Deserialize;
use validator::Validate;

use crate::ai::actions::{ActionContext, ActionError, ActionHandler, ActionResponse};
use crate::database::models::tag::Tag;

// 频道正则表达式
lazy_static! {
    static ref CHANNEL_REGEX: Regex = Regex::new(r"^(male|female)$").unwrap();
}

/// 将标签列表转换为名称字符串（用逗号分隔）
fn tags_to_names(tags: &[Tag]) -> String {
    if tags.is_empty() {
        return "无".to_string();
    }
    tags.iter()
        .map(|t| t.name.as_str())
        .collect::<Vec<_>>()
        .join("、")
}

/// 将标签类型转换为中文
fn tag_type_to_chinese(tag_type: &crate::database::models::tag::TagType) -> &'static str {
    use crate::database::models::tag::TagType;
    match tag_type {
        TagType::MainCategory => "主分类",
        TagType::Theme => "主题",
        TagType::Character => "角色",
        TagType::Plot => "情节",
    }
}

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

/// 推荐标签输入参数
#[derive(Debug, Deserialize, Validate)]
pub struct RecommendTagsInput {
    /// 标题（可选）
    pub title: Option<String>,

    /// 频道（必填，必须是 male 或 female）
    #[validate(regex(path = "CHANNEL_REGEX", message = "频道必须是 'male' 或 'female'"))]
    pub channel: String,

    /// 简介（可选）
    pub introduction: Option<String>,

    /// 已选择的标签 ID 列表（可选）
    pub tag_ids: Option<Vec<i64>>,
}

/// 推荐标签 Action
///
/// 根据小说上下文信息，AI 推荐合适的标签
pub struct RecommendTagsAction;

#[async_trait]
impl ActionHandler for RecommendTagsAction {
    fn name(&self) -> &str {
        "recommend_tags"
    }

    fn description(&self) -> &str {
        "根据小说上下文信息推荐合适的标签"
    }

    async fn handle(&self, ctx: ActionContext) -> Result<ActionResponse, ActionError> {
        // 1. 反序列化并验证输入参数
        let input: RecommendTagsInput = serde_json::from_value(ctx.input)
            .map_err(|e| ActionError::InvalidInput(format!("参数格式错误: {}", e)))?;

        input
            .validate()
            .map_err(|e| ActionError::InvalidInput(format!("参数验证失败: {}", e)))?;

        // 2. 查询对应频道的标签列表（按类型分组）
        let tag_repo = ctx.tag_repo.read().await;

        let main_categories = tag_repo
            .find_by_audience_and_type(&input.channel, "main_category")
            .await
            .map_err(|e| ActionError::ExecutionFailed(format!("查询主分类标签失败: {}", e)))?;

        let themes = tag_repo
            .find_by_audience_and_type(&input.channel, "theme")
            .await
            .map_err(|e| ActionError::ExecutionFailed(format!("查询主题标签失败: {}", e)))?;

        let characters = tag_repo
            .find_by_audience_and_type(&input.channel, "character")
            .await
            .map_err(|e| ActionError::ExecutionFailed(format!("查询角色标签失败: {}", e)))?;

        let plots = tag_repo
            .find_by_audience_and_type(&input.channel, "plot")
            .await
            .map_err(|e| ActionError::ExecutionFailed(format!("查询情节标签失败: {}", e)))?;

        drop(tag_repo); // 释放读锁

        // 2.5. 查询已选标签信息（如果提供了 tag_ids）
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
                        .map(|t| format!("- {} ({})", t.name, tag_type_to_chinese(&t.tag_type)))
                        .collect::<Vec<_>>()
                        .join("\n");

                    // 统计每种类型已选的数量
                    use crate::database::models::tag::TagType;
                    let mut main_count = 0;
                    let mut theme_count = 0;
                    let mut character_count = 0;
                    let mut plot_count = 0;

                    for tag in &selected_tags {
                        match tag.tag_type {
                            TagType::MainCategory => main_count += 1,
                            TagType::Theme => theme_count += 1,
                            TagType::Character => character_count += 1,
                            TagType::Plot => plot_count += 1,
                        }
                    }

                    Some((
                        tags_detail,
                        main_count,
                        theme_count,
                        character_count,
                        plot_count,
                    ))
                } else {
                    None
                }
            } else {
                None
            }
        } else {
            None
        };

        // 3. 计算剩余可推荐数量
        let (main_remaining, theme_remaining, character_remaining, plot_remaining) =
            if let Some((_, main_count, theme_count, character_count, plot_count)) =
                &selected_tags_info
            {
                (
                    1usize.saturating_sub(*main_count as usize),
                    2usize.saturating_sub(*theme_count as usize),
                    2usize.saturating_sub(*character_count as usize),
                    2usize.saturating_sub(*plot_count as usize),
                )
            } else {
                (1, 2, 2, 2)
            };

        // 4. 检查是否还需要推荐标签
        if main_remaining == 0
            && theme_remaining == 0
            && character_remaining == 0
            && plot_remaining == 0
        {
            // 所有类型都已选满，直接返回空数组
            return Ok(ActionResponse {
                data: serde_json::json!({
                    "tags": []
                }),
                metadata: std::collections::HashMap::new(),
            });
        }

        // 5. 使用 Tera 模板渲染提示词
        use crate::ai::prompts::{PromptTemplates, RecommendTagsContext};

        let templates = PromptTemplates::new()
            .map_err(|e| ActionError::ExecutionFailed(format!("加载模板失败: {}", e)))?;

        let selected_info = selected_tags_info
            .as_ref()
            .map(|(info, _, _, _, _)| info.clone());

        let prompt_context = RecommendTagsContext {
            channel_name: if input.channel == "male" {
                "男频"
            } else {
                "女频"
            }
            .to_string(),
            title: input.title.clone(),
            introduction: input.introduction.clone(),
            selected_tags_info: selected_info,
            main_categories: tags_to_names(&main_categories),
            themes: tags_to_names(&themes),
            characters: tags_to_names(&characters),
            plots: tags_to_names(&plots),
            main_limit: 1,
            theme_limit: 2,
            character_limit: 2,
            plot_limit: 2,
            main_remaining,
            theme_remaining,
            character_remaining,
            plot_remaining,
        };

        let prompt = templates
            .render_recommend_tags(&prompt_context)
            .map_err(|e| ActionError::ExecutionFailed(format!("渲染提示词失败: {}", e)))?;

        // 6. 调用 AI 服务生成推荐
        use crate::ai::types::{AiRequestData, Message, MessageRole};

        let request_data = AiRequestData {
            model_id: None, // 使用默认模型
            messages: vec![Message {
                role: MessageRole::User,
                content: prompt.clone(), // 克隆一份给 AI
            }],
        };

        let ai_response = ctx
            .ai_service
            .chat(request_data)
            .await
            .map_err(|e| ActionError::ExecutionFailed(format!("AI 调用失败: {}", e)))?;

        // 7. 解析 AI 返回的 JSON
        let ai_content = &ai_response.content;

        // 清理可能的代码块标记
        let cleaned_content = clean_json_output(ai_content);

        // 尝试解析 JSON
        let tags_result: serde_json::Value = serde_json::from_str(&cleaned_content)
            .map_err(|e| ActionError::ExecutionFailed(format!("解析 AI 返回结果失败: {}", e)))?;

        // 提取各类标签名称
        let main_category_names: Vec<String> = tags_result
            .get("main_category")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str().map(|s| s.to_string()))
                    .collect()
            })
            .unwrap_or_else(|| vec![]);

        let theme_names: Vec<String> = tags_result
            .get("theme")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str().map(|s| s.to_string()))
                    .collect()
            })
            .unwrap_or_else(|| vec![]);

        let character_names: Vec<String> = tags_result
            .get("character")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str().map(|s| s.to_string()))
                    .collect()
            })
            .unwrap_or_else(|| vec![]);

        let plot_names: Vec<String> = tags_result
            .get("plot")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str().map(|s| s.to_string()))
                    .collect()
            })
            .unwrap_or_else(|| vec![]);

        // 8. 收集所有需要查询的标签名称
        let all_names: Vec<&str> = main_category_names
            .iter()
            .map(|s| s.as_str())
            .chain(theme_names.iter().map(|s| s.as_str()))
            .chain(character_names.iter().map(|s| s.as_str()))
            .chain(plot_names.iter().map(|s| s.as_str()))
            .collect();

        // 9. 批量查询标签 ID
        let tag_ids = if all_names.is_empty() {
            vec![]
        } else {
            let tag_repo = ctx.tag_repo.read().await;
            let tags = tag_repo
                .find_by_names(&all_names)
                .await
                .map_err(|e| ActionError::ExecutionFailed(format!("查询标签失败: {}", e)))?;
            drop(tag_repo);

            tags.iter().map(|t| t.id).collect::<Vec<i64>>()
        };

        // 10. 返回结果
        Ok(ActionResponse {
            data: serde_json::json!({
                "tags": tag_ids
            }),
            metadata: std::collections::HashMap::new(),
        })
    }
}
