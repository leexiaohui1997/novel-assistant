// 可复用上下文查询函数模块
//
// 封装 AI Action 中常用的数据查询逻辑，供多个 Action 复用。
// 每个函数的圈复杂度 < 5。

use std::sync::Arc;
use tokio::sync::RwLock;

use crate::ai::prompts::{ChapterOutlineItem, CharacterInfo};
use crate::database::error::DbError;
use crate::database::models::chapter::{Chapter, ChapterOutlineRow};
use crate::database::models::chapter_outline::ChapterOutlineWithCharacters;
use crate::database::models::character::CharacterType;
use crate::database::models::tag::{Tag, TagType};
use crate::database::repositories::{
    ChapterOutlineRepository, ChapterRepository, CharacterRepository, NovelRepository,
    QueryOptions, TagRepository,
};
use uuid::Uuid;

/// 小说基础信息（用于 AI 提示词上下文）
#[derive(Debug, Clone)]
pub struct NovelInfo {
    pub title: String,
    pub channel_name: String,
    pub tags: String,
    pub description: String,
}

/// 章节大纲信息（用于 AI 提示词上下文）
#[derive(Debug, Clone)]
pub struct ChapterOutlineInfo {
    pub positioning: Option<String>,
    pub plot: Option<String>,
    pub character_ids: Vec<Uuid>,
}

/// 分卷信息（用于 AI 提示词上下文）
#[derive(Debug, Clone)]
pub struct VolumeInfo {
    pub sequence: i64,
    pub name: String,
}

/// 章节位置信息（卷名、章节序号）
#[derive(Debug, Clone)]
pub struct ChapterLocationInfo {
    pub chapter_sequence: i64,
    pub volume: Option<VolumeInfo>,
}

/// 章节内容信息（标题 + 正文）
#[derive(Debug, Clone)]
pub struct ChapterContentInfo {
    pub title: String,
    pub content: String,
}

/// 将 TagType 映射为人类可读的中文分类名称
///
/// # 示例
/// ```ignore
/// assert_eq!(tag_type_cn(&TagType::MainCategory), "主分类");
/// ```
fn tag_type_cn(tag_type: &TagType) -> &'static str {
    match tag_type {
        TagType::MainCategory => "主分类",
        TagType::Theme => "主题",
        TagType::Character => "角色",
        TagType::Plot => "情节",
    }
}

/// 将标签格式化为「标签名（分类）」形式，供 AI 提示词构建上下文统一使用。
///
/// 此函数为全局唯一的标签命名格式记录点。调用方在拼接任何待送入 AI 上下文的标签名称时都应使用此函数，
/// 以封装分类后缀与括号风格（中文全角括号）。
///
/// # 示例
/// ```ignore
/// let tag = Tag { name: "现代都市".into(), tag_type: TagType::MainCategory, ... };
/// assert_eq!(format_tag_name(&tag), "现代都市（主分类）");
/// ```
pub fn format_tag_name(tag: &Tag) -> String {
    format!("{}（{}）", tag.name, tag_type_cn(&tag.tag_type))
}

/// 根据 novel_id 查询小说基础信息（书名、频道、标签、简介）
pub async fn fetch_novel_info(
    novel_repo: &Arc<RwLock<Box<dyn NovelRepository + Send + Sync>>>,
    _tag_repo: &Arc<RwLock<Box<dyn TagRepository + Send + Sync>>>,
    novel_id: Uuid,
) -> Result<NovelInfo, DbError> {
    let novel_repo_guard = novel_repo.read().await;
    let novel_with_tags = novel_repo_guard
        .find_by_id(
            novel_id,
            &QueryOptions {
                with_tags: true,
                ..Default::default()
            },
        )
        .await?;

    let channel_name = match novel_with_tags.novel.target_reader.as_str() {
        "male" => "男频",
        "female" => "女频",
        _ => "通用",
    }
    .to_string();

    let tags = novel_with_tags
        .tags
        .iter()
        .map(format_tag_name)
        .collect::<Vec<_>>()
        .join("、");

    Ok(NovelInfo {
        title: novel_with_tags.novel.title,
        channel_name,
        tags,
        description: novel_with_tags.novel.description,
    })
}

/// 根据 novel_id + chapter_id 查询章节大纲信息（定位、剧情、出场角色）
pub async fn fetch_chapter_outline(
    chapter_outline_repo: &Arc<RwLock<Box<dyn ChapterOutlineRepository + Send + Sync>>>,
    novel_id: Uuid,
    chapter_id: Option<Uuid>,
) -> Result<Option<ChapterOutlineInfo>, DbError> {
    let repo = chapter_outline_repo.read().await;
    let result = repo
        .find_with_characters(&novel_id, chapter_id.as_ref())
        .await?;

    Ok(result.map(|o| ChapterOutlineInfo {
        positioning: o.outline.positioning,
        plot: o.outline.plot,
        character_ids: o.character_ids,
    }))
}

/// 根据 novel_id 查询所有角色列表（姓名、性别、角色类型、背景、外貌、性格、其它）
pub async fn fetch_characters(
    character_repo: &Arc<RwLock<Box<dyn CharacterRepository + Send + Sync>>>,
    novel_id: Uuid,
) -> Result<Vec<CharacterInfo>, DbError> {
    let repo = character_repo.read().await;
    let characters = repo.find_by_novel_id(&novel_id).await?;

    Ok(characters
        .into_iter()
        .map(|c| CharacterInfo {
            id: c.id,
            name: c.name,
            gender: format!("{:?}", c.gender),
            character_type: c.character_type.map(character_type_label),
            background: c.background,
            appearance: c.appearance,
            personality: c.personality,
            additional_info: c.additional_info,
        })
        .collect())
}

/// 将角色类型枚举转为中文标签，便于 AI 理解
pub(crate) fn character_type_label(ct: CharacterType) -> String {
    match ct {
        CharacterType::Protagonist => "主角",
        CharacterType::SecondProtagonist => "二号主角",
        CharacterType::ThirdProtagonist => "三号主角",
        CharacterType::Supporting => "配角",
        CharacterType::MinorSupporting => "次要配角",
        CharacterType::MajorAntagonist => "重要敌对",
        CharacterType::MinorAntagonist => "次要敌对",
        CharacterType::AntagonistFaction => "敌对阵营",
    }
    .to_string()
}

/// 获取所有角色类型选项（用于动态注入模板）
pub(crate) fn get_character_type_options() -> Vec<crate::ai::prompts::CharacterTypeOption> {
    use crate::database::models::character::ALL_CHARACTER_TYPES;
    ALL_CHARACTER_TYPES
        .iter()
        .map(|&ct| {
            // 使用 serde 序列化获取正确的 snake_case 值
            let value = serde_json::to_value(ct)
                .ok()
                .and_then(|v| v.as_str().map(|s| s.to_string()))
                .unwrap_or_else(|| format!("{:?}", ct).to_lowercase());

            crate::ai::prompts::CharacterTypeOption {
                value,
                label: character_type_label(ct),
            }
        })
        .collect()
}

/// 根据 novel_id + chapter_id 查询章节标题和正文；入参优先覆盖
pub async fn fetch_chapter_content(
    chapter_repo: &Arc<RwLock<Box<dyn ChapterRepository + Send + Sync>>>,
    _novel_id: Uuid,
    chapter_id: Option<Uuid>,
    input_title: Option<String>,
    input_content: Option<String>,
) -> Result<Option<ChapterContentInfo>, DbError> {
    let chapter_id = match chapter_id {
        Some(id) => id,
        None => {
            // 没有章节 ID 时，仍返回输入参数中的 title/content
            if input_title.is_some() || input_content.is_some() {
                return Ok(Some(ChapterContentInfo {
                    title: input_title.unwrap_or_default(),
                    content: input_content.unwrap_or_default(),
                }));
            }
            return Ok(None);
        }
    };

    let repo = chapter_repo.read().await;
    let chapter = repo.find_by_id(chapter_id).await?;

    Ok(chapter.map(|c| ChapterContentInfo {
        title: input_title.unwrap_or(c.title),
        content: input_content.unwrap_or(c.content),
    }))
}

/// 根据 chapter_id 查询章节位置信息（卷名、卷序号、章节序号）
///
/// - 有 chapter_id 时：查询该章节所属卷和序号；
///   若章节未关联任何卷，则默认归属第一卷（默认卷）。
/// - 无 chapter_id（新建章节场景）：卷取最后一卷，章节序号 = 最后一卷最大序号 + 1；
///   若小说尚无分卷，则使用默认的第一卷。
pub async fn fetch_chapter_location(
    chapter_repo: &Arc<RwLock<Box<dyn ChapterRepository + Send + Sync>>>,
    novel_id: Uuid,
    chapter_id: Option<Uuid>,
) -> Result<ChapterLocationInfo, DbError> {
    let repo = chapter_repo.read().await;

    if let Some(id) = chapter_id {
        return fetch_existing_chapter_location(&repo, id).await;
    }

    // 新建章节场景：取最后一卷，章节序号 = 最后一卷最大序号 + 1
    fetch_new_chapter_location(&repo, novel_id).await
}

/// 已有章节的位置查询
async fn fetch_existing_chapter_location(
    repo: &Box<dyn ChapterRepository + Send + Sync>,
    chapter_id: Uuid,
) -> Result<ChapterLocationInfo, DbError> {
    let chapter = repo.find_by_id(chapter_id).await?;
    let chapter = match chapter {
        Some(c) => c,
        None => {
            return Ok(ChapterLocationInfo {
                chapter_sequence: 1,
                volume: Some(VolumeInfo {
                    sequence: 1,
                    name: "默认".to_string(),
                }),
            })
        }
    };

    let volume = repo.find_volume_by_chapter(chapter_id).await?;

    // 若章节未关联任何卷，默认归属第一卷
    let volume_info = volume
        .map(|v| VolumeInfo {
            sequence: v.sequence,
            name: v.name,
        })
        .or_else(|| {
            Some(VolumeInfo {
                sequence: 1,
                name: "默认".to_string(),
            })
        });

    Ok(ChapterLocationInfo {
        chapter_sequence: chapter.sequence,
        volume: volume_info,
    })
}

/// 新建章节的位置推断
async fn fetch_new_chapter_location(
    repo: &Box<dyn ChapterRepository + Send + Sync>,
    novel_id: Uuid,
) -> Result<ChapterLocationInfo, DbError> {
    let volumes = repo.get_volumes(novel_id).await?;

    // 若小说尚无分卷，使用默认第一卷
    let last_volume = volumes.last();

    let volume_info = last_volume
        .map(|v| VolumeInfo {
            sequence: v.sequence,
            name: v.name.clone(),
        })
        .or_else(|| {
            Some(VolumeInfo {
                sequence: 1,
                name: "默认".to_string(),
            })
        });

    let volume_sequence = volume_info.as_ref().map(|v| v.sequence).unwrap_or(1);

    // 取最后一卷下最大章节序号
    let max_seq = repo
        .get_max_sequence_in_volume(novel_id, volume_sequence)
        .await?;

    Ok(ChapterLocationInfo {
        chapter_sequence: max_seq.unwrap_or(0) + 1,
        volume: volume_info,
    })
}

/// 查询全书章节标题大纲（按卷序、章序升序）
///
/// 供 AI 提示词上下文渲染“全书章节标题”小节。
/// 仅返回非草稿章节；孤儿章节归首卷=1。
pub async fn fetch_chapter_outline_list(
    chapter_repo: &Arc<RwLock<Box<dyn ChapterRepository + Send + Sync>>>,
    novel_id: Uuid,
) -> Result<Vec<ChapterOutlineItem>, DbError> {
    let repo = chapter_repo.read().await;
    let rows = repo.list_chapter_outline(novel_id).await?;
    Ok(rows.into_iter().map(outline_row_to_item).collect())
}

/// 仓储层 `ChapterOutlineRow` 转换为提示词层 `ChapterOutlineItem`
fn outline_row_to_item(row: ChapterOutlineRow) -> ChapterOutlineItem {
    ChapterOutlineItem {
        volume_sequence: row.volume_sequence,
        chapter_sequence: row.chapter_sequence,
        title: row.title,
    }
}

/// 根据 novel_id + chapter_id 查询前情介绍（此章节之前所有大纲的剧情汇总）
pub async fn fetch_previous_outlines(
    chapter_repo: &Arc<RwLock<Box<dyn ChapterRepository + Send + Sync>>>,
    chapter_outline_repo: &Arc<RwLock<Box<dyn ChapterOutlineRepository + Send + Sync>>>,
    novel_id: Uuid,
    chapter_id: Option<Uuid>,
) -> Result<String, DbError> {
    let (chapters, outlines) =
        fetch_chapters_and_outlines(chapter_repo, chapter_outline_repo, novel_id).await?;

    let target_seq = resolve_target_sequence(&chapters, chapter_id);
    let previous_plots = collect_previous_plots(&chapters, &outlines, target_seq);

    Ok(previous_plots.join("\n"))
}

/// 查询小说所有非草稿章节和大纲
async fn fetch_chapters_and_outlines(
    chapter_repo: &Arc<RwLock<Box<dyn ChapterRepository + Send + Sync>>>,
    chapter_outline_repo: &Arc<RwLock<Box<dyn ChapterOutlineRepository + Send + Sync>>>,
    novel_id: Uuid,
) -> Result<(Vec<Chapter>, Vec<ChapterOutlineWithCharacters>), DbError> {
    let chapters = chapter_repo
        .read()
        .await
        .find_all_non_draft(novel_id)
        .await?;
    let outlines = chapter_outline_repo
        .read()
        .await
        .find_all_by_novel(&novel_id)
        .await?;
    Ok((chapters, outlines))
}

/// 解析目标章节的 sequence
fn resolve_target_sequence(chapters: &[Chapter], chapter_id: Option<Uuid>) -> Option<i64> {
    chapter_id.and_then(|id| chapters.iter().find(|c| c.id == id).map(|c| c.sequence))
}

/// 收集目标章节之前所有大纲的剧情
fn collect_previous_plots(
    chapters: &[Chapter],
    outlines: &[ChapterOutlineWithCharacters],
    target_seq: Option<i64>,
) -> Vec<String> {
    // 建立 chapter_id -> sequence 映射
    let seq_map: std::collections::HashMap<Uuid, i64> =
        chapters.iter().map(|c| (c.id, c.sequence)).collect();

    let mut plots: Vec<(i64, String)> = outlines
        .iter()
        .filter_map(|o| {
            let cid = o.outline.chapter_id?;
            let seq = seq_map.get(&cid).copied()?;
            let plot = o.outline.plot.clone()?;
            Some((seq, plot))
        })
        .collect();

    plots.sort_by_key(|(seq, _)| *seq);

    match target_seq {
        Some(ts) => plots
            .into_iter()
            .filter(|(seq, _)| *seq < ts)
            .map(|(_, plot)| plot)
            .collect(),
        None => plots.into_iter().map(|(_, plot)| plot).collect(),
    }
}
