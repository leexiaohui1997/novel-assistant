// AI 识别章节出场角色 Action 实现

use async_trait::async_trait;
use serde::Deserialize;
use uuid::Uuid;

use crate::ai::actions::context_helpers;
use crate::ai::actions::{ActionContext, ActionError, ActionHandler, ActionResponse};
use crate::ai::prompts::{CharacterWithIdInfo, EditChapterCharactersContext, PromptTemplates};
use crate::ai::types::{AiRequestData, Message, MessageRole};

/// 识别章节出场角色输入参数
#[derive(Debug, Deserialize)]
pub struct EditChapterCharactersInput {
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

/// AI 返回的角色识别结果
#[derive(Debug, Deserialize)]
struct CharactersResult {
    /// 已创建角色的 ID 数组
    existing_character_ids: Vec<String>,

    /// 待创建角色的描写片段数组（可能不含角色名，仅相关描写）
    new_character_descriptions: Vec<String>,
}

/// AI 识别章节出场角色 Action
///
/// 根据小说上下文、大纲、正文和前情介绍，
/// AI 识别本章应出场的角色，返回已有角色 ID 和待创建角色名称
pub struct EditChapterCharactersAction;

#[async_trait]
impl ActionHandler for EditChapterCharactersAction {
    fn name(&self) -> &str {
        "edit_chapter_characters"
    }

    fn description(&self) -> &str {
        "根据小说上下文信息识别章节出场角色"
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

        let ai_response = call_ai(ctx, prompt).await?;
        let result = parse_ai_response(&ai_response)?;

        Ok(ActionResponse {
            data: serde_json::json!({
                "existing_character_ids": result.existing_character_ids,
                "new_character_descriptions": result.new_character_descriptions,
            }),
            metadata: std::collections::HashMap::new(),
        })
    }
}

/// 反序列化入参
fn parse_input(ctx: &ActionContext) -> Result<EditChapterCharactersInput, ActionError> {
    serde_json::from_value(ctx.input.clone())
        .map_err(|e| ActionError::InvalidInput(format!("参数格式错误: {}", e)))
}

/// 查询小说基础信息
async fn fetch_novel_info(
    ctx: &ActionContext,
    input: &EditChapterCharactersInput,
) -> Result<context_helpers::NovelInfo, ActionError> {
    context_helpers::fetch_novel_info(&ctx.novel_repo, &ctx.tag_repo, input.novel_id)
        .await
        .map_err(|e| ActionError::ExecutionFailed(format!("查询小说信息失败: {}", e)))
}

/// 查询章节大纲信息
async fn fetch_outline_info(
    ctx: &ActionContext,
    input: &EditChapterCharactersInput,
) -> Result<Option<context_helpers::ChapterOutlineInfo>, ActionError> {
    context_helpers::fetch_chapter_outline(
        &ctx.chapter_outline_repo,
        input.novel_id,
        input.chapter_id,
    )
    .await
    .map_err(|e| ActionError::ExecutionFailed(format!("查询大纲信息失败: {}", e)))
}

/// 查询角色列表（含 ID，供模板输出和 AI 识别）
async fn fetch_character_list(
    ctx: &ActionContext,
    input: &EditChapterCharactersInput,
) -> Result<Vec<CharacterWithIdInfo>, ActionError> {
    let characters = context_helpers::fetch_characters(&ctx.character_repo, input.novel_id)
        .await
        .map_err(|e| ActionError::ExecutionFailed(format!("查询角色列表失败: {}", e)))?;

    // 将 CharacterInfo 转换为 CharacterWithIdInfo（包含 ID）
    Ok(characters
        .into_iter()
        .map(|c| CharacterWithIdInfo {
            id: c.id.to_string(),
            name: c.name,
            gender: c.gender,
            background: c.background,
            appearance: c.appearance,
            personality: c.personality,
            additional_info: c.additional_info,
        })
        .collect())
}

/// 查询章节内容（标题+正文）
async fn fetch_content(
    ctx: &ActionContext,
    input: &EditChapterCharactersInput,
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
    input: &EditChapterCharactersInput,
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

/// 查询章节位置信息
async fn fetch_location(
    ctx: &ActionContext,
    input: &EditChapterCharactersInput,
) -> Result<context_helpers::ChapterLocationInfo, ActionError> {
    context_helpers::fetch_chapter_location(&ctx.chapter_repo, input.novel_id, input.chapter_id)
        .await
        .map_err(|e| ActionError::ExecutionFailed(format!("查询章节位置失败: {}", e)))
}

/// 渲染提示词
fn render_prompt(
    novel_info: &context_helpers::NovelInfo,
    outline_info: &Option<context_helpers::ChapterOutlineInfo>,
    characters: &[CharacterWithIdInfo],
    chapter_content: &Option<context_helpers::ChapterContentInfo>,
    chapter_location: &context_helpers::ChapterLocationInfo,
    previous_plots: &str,
    user_feedback: &Option<String>,
) -> Result<String, ActionError> {
    let templates = PromptTemplates::new()
        .map_err(|e| ActionError::ExecutionFailed(format!("加载模板失败: {}", e)))?;

    // 从 characters 中筛选大纲关联角色（需用 CharacterWithIdInfo 的 id 匹配）
    let outline_characters = build_outline_characters(outline_info, characters);

    let prompt_context = EditChapterCharactersContext {
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
        outline_characters,
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
        .render_edit_chapter_characters(&prompt_context)
        .map_err(|e| ActionError::ExecutionFailed(format!("渲染提示词失败: {}", e)))
}

/// 构建大纲关联角色列表
fn build_outline_characters(
    outline_info: &Option<context_helpers::ChapterOutlineInfo>,
    characters: &[CharacterWithIdInfo],
) -> Option<Vec<CharacterWithIdInfo>> {
    match outline_info {
        Some(info) if !info.character_ids.is_empty() && !characters.is_empty() => {
            let filtered: Vec<_> = characters
                .iter()
                .filter(|c| info.character_ids.iter().any(|id| c.id == id.to_string()))
                .cloned()
                .collect();
            if filtered.is_empty() {
                None
            } else {
                Some(filtered)
            }
        }
        _ => None,
    }
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

/// 解析 AI 返回的 JSON
fn parse_ai_response(raw: &str) -> Result<CharactersResult, ActionError> {
    // 尝试提取 JSON 内容（AI 可能包裹在 markdown 代码块中）
    let json_str = extract_json(raw);

    serde_json::from_str::<CharactersResult>(&json_str)
        .map_err(|e| ActionError::ExecutionFailed(format!("解析 AI 返回结果失败: {}", e)))
}

/// 从 AI 响应中提取 JSON（处理 markdown 代码块包裹的情况）
fn extract_json(raw: &str) -> String {
    // 尝试提取 ```json ... ``` 代码块
    if let Some(content) = extract_code_block(raw) {
        return content;
    }

    // 尝试提取 { ... } JSON 对象
    if let Some(content) = extract_json_object(raw) {
        return content;
    }

    raw.to_string()
}

/// 提取 markdown 代码块中的内容
fn extract_code_block(raw: &str) -> Option<String> {
    let start_marker = "```json";
    let end_marker = "```";

    let start = raw.find(start_marker)?;
    let content_start = start + start_marker.len();
    let end = raw[content_start..].find(end_marker)?;
    Some(raw[content_start..content_start + end].trim().to_string())
}

/// 提取第一个 { } 包裹的 JSON 对象
fn extract_json_object(raw: &str) -> Option<String> {
    let start = raw.find('{')?;
    let end = raw.rfind('}')?;
    if end > start {
        Some(raw[start..=end].to_string())
    } else {
        None
    }
}
