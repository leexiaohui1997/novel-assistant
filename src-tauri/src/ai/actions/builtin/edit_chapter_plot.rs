// AI 编辑章节剧情 Action 实现

use async_trait::async_trait;
use serde::Deserialize;
use uuid::Uuid;

use crate::ai::actions::context_helpers;
use crate::ai::actions::{ActionContext, ActionError, ActionHandler, ActionResponse};
use crate::ai::prompts::{EditChapterPlotContext, PromptTemplates};
use crate::ai::types::{AiRequestData, Message, MessageRole};

/// 编辑章节剧情输入参数
#[derive(Debug, Deserialize)]
pub struct EditChapterPlotInput {
    /// 小说 ID（必填）
    pub novel_id: Uuid,

    /// 章节 ID（可选）
    #[serde(default)]
    pub chapter_id: Option<Uuid>,

    /// 标题（可选，覆盖数据库中的标题）
    #[serde(default)]
    pub title: Option<String>,

    /// 正文（可选，覆盖数据库中的正文）
    #[serde(default)]
    pub content: Option<String>,

    /// 用户意见（可选）
    #[serde(default)]
    pub user_feedback: Option<String>,
}

/// AI 编辑章节剧情 Action
///
/// 根据小说上下文、大纲、角色、正文和前情介绍，
/// AI 从正文中提炼剧情概要，或（无正文时）根据上下文编写剧情概要
pub struct EditChapterPlotAction;

#[async_trait]
impl ActionHandler for EditChapterPlotAction {
    fn name(&self) -> &str {
        "edit_chapter_plot"
    }

    fn description(&self) -> &str {
        "根据小说上下文信息生成或提炼章节剧情概要"
    }

    async fn handle(&self, ctx: ActionContext) -> Result<ActionResponse, ActionError> {
        let input = parse_input(&ctx)?;

        let novel_info = fetch_novel_info(&ctx, &input).await?;
        let outline_info = fetch_outline_info(&ctx, &input).await?;
        let characters = fetch_character_list(&ctx, &input).await?;
        let chapter_content = fetch_content(&ctx, &input).await?;
        let chapter_location = fetch_location(&ctx, &input).await?;
        let previous_plots = fetch_prev_plots(&ctx, &input).await?;

        let prompt = render_prompt(
            &novel_info,
            &outline_info,
            &characters,
            &chapter_content,
            &chapter_location,
            &previous_plots,
            &input.user_feedback,
        )?;

        let plot = call_ai(ctx, prompt).await?;

        Ok(ActionResponse {
            data: serde_json::json!({ "plot": plot }),
            metadata: std::collections::HashMap::new(),
        })
    }
}

/// 反序列化入参
fn parse_input(ctx: &ActionContext) -> Result<EditChapterPlotInput, ActionError> {
    serde_json::from_value(ctx.input.clone())
        .map_err(|e| ActionError::InvalidInput(format!("参数格式错误: {}", e)))
}

/// 查询小说基础信息
async fn fetch_novel_info(
    ctx: &ActionContext,
    input: &EditChapterPlotInput,
) -> Result<context_helpers::NovelInfo, ActionError> {
    context_helpers::fetch_novel_info(&ctx.novel_repo, &ctx.tag_repo, input.novel_id)
        .await
        .map_err(|e| ActionError::ExecutionFailed(format!("查询小说信息失败: {}", e)))
}

/// 查询章节大纲信息
async fn fetch_outline_info(
    ctx: &ActionContext,
    input: &EditChapterPlotInput,
) -> Result<Option<context_helpers::ChapterOutlineInfo>, ActionError> {
    context_helpers::fetch_chapter_outline(
        &ctx.chapter_outline_repo,
        input.novel_id,
        input.chapter_id,
    )
    .await
    .map_err(|e| ActionError::ExecutionFailed(format!("查询大纲信息失败: {}", e)))
}

/// 查询角色列表
async fn fetch_character_list(
    ctx: &ActionContext,
    input: &EditChapterPlotInput,
) -> Result<Vec<crate::ai::prompts::CharacterInfo>, ActionError> {
    context_helpers::fetch_characters(&ctx.character_repo, input.novel_id)
        .await
        .map_err(|e| ActionError::ExecutionFailed(format!("查询角色列表失败: {}", e)))
}

/// 查询章节内容（标题+正文）
async fn fetch_content(
    ctx: &ActionContext,
    input: &EditChapterPlotInput,
) -> Result<Option<context_helpers::ChapterContentInfo>, ActionError> {
    context_helpers::fetch_chapter_content(
        &ctx.chapter_repo,
        input.novel_id,
        input.chapter_id,
        input.title.clone(),
        input.content.clone(),
    )
    .await
    .map_err(|e| ActionError::ExecutionFailed(format!("查询章节内容失败: {}", e)))
}

/// 查询前情介绍
async fn fetch_prev_plots(
    ctx: &ActionContext,
    input: &EditChapterPlotInput,
) -> Result<String, ActionError> {
    context_helpers::fetch_previous_outlines(
        &ctx.chapter_repo,
        &ctx.chapter_outline_repo,
        input.novel_id,
        input.chapter_id,
    )
    .await
    .map_err(|e| ActionError::ExecutionFailed(format!("查询前情介绍失败: {}", e)))
}

/// 查询章节位置信息（卷名、卷序号、章节序号）
async fn fetch_location(
    ctx: &ActionContext,
    input: &EditChapterPlotInput,
) -> Result<context_helpers::ChapterLocationInfo, ActionError> {
    context_helpers::fetch_chapter_location(&ctx.chapter_repo, input.novel_id, input.chapter_id)
        .await
        .map_err(|e| ActionError::ExecutionFailed(format!("查询章节位置失败: {}", e)))
}

/// 渲染提示词
fn render_prompt(
    novel_info: &context_helpers::NovelInfo,
    outline_info: &Option<context_helpers::ChapterOutlineInfo>,
    characters: &[crate::ai::prompts::CharacterInfo],
    chapter_content: &Option<context_helpers::ChapterContentInfo>,
    chapter_location: &context_helpers::ChapterLocationInfo,
    previous_plots: &str,
    user_feedback: &Option<String>,
) -> Result<String, ActionError> {
    let templates = PromptTemplates::new()
        .map_err(|e| ActionError::ExecutionFailed(format!("加载模板失败: {}", e)))?;

    let prompt_context = EditChapterPlotContext {
        title: novel_info.title.clone(),
        channel_name: Some(novel_info.channel_name.clone()),
        tags: if novel_info.tags.is_empty() {
            None
        } else {
            Some(novel_info.tags.clone())
        },
        description: if novel_info.description.is_empty() {
            None
        } else {
            Some(novel_info.description.clone())
        },
        chapter_sequence: Some(chapter_location.chapter_sequence),
        volume_sequence: chapter_location.volume.as_ref().map(|v| v.sequence),
        volume_name: chapter_location.volume.as_ref().map(|v| v.name.clone()),
        outline_positioning: outline_info.as_ref().and_then(|o| o.positioning.clone()),
        outline_plot: outline_info.as_ref().and_then(|o| o.plot.clone()),
        outline_characters: match outline_info {
            Some(info) if !info.character_ids.is_empty() && !characters.is_empty() => {
                let ids = &info.character_ids;
                let filtered: Vec<_> = characters
                    .iter()
                    .filter(|c| ids.contains(&c.id))
                    .cloned()
                    .collect();
                if filtered.is_empty() {
                    None
                } else {
                    Some(filtered)
                }
            }
            _ => None,
        },
        characters: if characters.is_empty() {
            None
        } else {
            Some(characters.to_vec())
        },
        chapter_title: chapter_content.as_ref().map(|c| c.title.clone()),
        chapter_content: chapter_content.as_ref().map(|c| c.content.clone()),
        previous_plots: if previous_plots.is_empty() {
            None
        } else {
            Some(previous_plots.to_string())
        },
        user_feedback: user_feedback.clone(),
    };

    templates
        .render_edit_chapter_plot(&prompt_context)
        .map_err(|e| ActionError::ExecutionFailed(format!("渲染提示词失败: {}", e)))
}

/// 调用 AI 服务
async fn call_ai(ctx: ActionContext, prompt: String) -> Result<String, ActionError> {
    let parsed_model_id = ctx.model_id.and_then(|id| Uuid::parse_str(&id).ok());

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

    Ok(ai_response.content.trim().to_string())
}
