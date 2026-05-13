//! 提示词片段工具函数集合
//!
//! 该模块承载"装配型"的提示词片段：输入业务实体（如小说 ID），
//! 内部完成"数据库查询 → 数据整理 → Tera 模板渲染"的完整流程，
//! 输出可直接拼接到上层大提示词中的 Markdown 文本。
//!
//! 当前提供的片段：
//! - [`render_novel_terms_fragment`]：渲染"小说相关名词"片段。

use std::collections::HashMap;
use std::path::Path;

use thiserror::Error;
use tracing::error;
use uuid::Uuid;

use crate::database::error::DbError;
use crate::database::models::chapter_term_relation::ChapterTermRelationWithChapter;
use crate::database::models::novel_term::{NovelTerm, TermType, ALL_TERM_TYPES};
use crate::database::repositories::chapter_term_relation_repo::ChapterTermRelationRepository;
use crate::database::repositories::novel_term_repo::{NovelTermQuery, NovelTermRepository};

use super::{NovelTermGroup, NovelTermItem, NovelTermsContext, PromptTemplates};

/// 提示词片段装配错误
///
/// 将"入参校验 / 数据库 / 模板渲染"三类错误统一对上抛出，
/// 调用方可按需拍平为自身的错误类型。
#[derive(Debug, Error)]
pub enum PromptFragmentError {
    /// 入参非法（调用方 bug，不应发生在业务运行时）
    #[error("提示词片段入参非法：{0}")]
    InvalidInput(&'static str),

    /// 数据库查询失败
    #[error("提示词片段查询数据失败")]
    Db(#[source] DbError),

    /// Tera 模板渲染失败
    #[error("提示词片段模板渲染失败")]
    Render(#[source] tera::Error),
}

/// 渲染"小说相关名词"提示词片段
///
/// 该函数为装配型工具：
/// 1. 校验 `novel_id` 非空；
/// 2. 通过 [`NovelTermRepository::find_with_query`] 全量查询该小说下的名词
///    （`page_size = 0`，不做任何类型/名称/描述过滤）；
/// 3. 按 [`ALL_TERM_TYPES`] 顺序分组，命中为空的类型不进入结果；
/// 4. 当 `with_full_description = true` 时，再调用
///    [`ChapterTermRelationRepository::find_term_chapter_relations_by_novel`] 一次性
///    拉取所有名词的章节关联记录，并按 `term_id` 分组拼接为"完整描述"覆盖每条名词的描述字段；
/// 5. 使用 `fragments/novel_terms.tera` 模板渲染为 Markdown 文本。
///
/// # 参数
/// - `novel_id`: 小说 ID；为 `Uuid::nil()` 时直接返回 [`PromptFragmentError::InvalidInput`]。
/// - `show_id`: 是否在每条名词后显示 `[ID: xxx]` 片段。
/// - `with_full_description`: 是否用"完整描述"（该名词在所有关联章节中的
///   描述按卷序、章序拼接）覆盖名词主表的简单描述；`false` 时沿用主表 `description`。
/// - `repo`: 小说名词仓储 trait 对象。
/// - `chapter_term_relation_repo`: 名词-章节关联仓储 trait 对象；仅当
///   `with_full_description = true` 时被使用。
/// - `templates_root`: `templates/` 根目录；内部将调用 [`PromptTemplates::new`]
///   现场实例化模板引擎，与 `ai/actions/builtin/*` 风格一致。
///
/// # 返回值
/// 成功返回完整渲染后的字符串；小说下无名词时同样返回成功（模板输出"暂无名词"占位）。
pub async fn render_novel_terms_fragment(
    novel_id: Uuid,
    show_id: bool,
    with_full_description: bool,
    repo: &(dyn NovelTermRepository + Send + Sync),
    chapter_term_relation_repo: &(dyn ChapterTermRelationRepository + Send + Sync),
    templates_root: &Path,
) -> Result<String, PromptFragmentError> {
    if novel_id.is_nil() {
        return Err(PromptFragmentError::InvalidInput("novel_id 不能为空"));
    }

    let terms = fetch_all_terms(novel_id, repo).await?;
    let grouped = group_terms_by_type(terms);

    let full_desc_map = if with_full_description && !grouped.is_empty() {
        build_full_descriptions_map(novel_id, chapter_term_relation_repo).await?
    } else {
        HashMap::new()
    };

    let context = build_template_context(grouped, show_id, &full_desc_map);

    render_with_templates(templates_root, &context)
}

/// 以"全量查询"的方式拉取指定小说的所有名词
///
/// 固定 `page = 1, page_size = 0`（page_size=0 表示不分页，Repo 会返回全部）。
async fn fetch_all_terms(
    novel_id: Uuid,
    repo: &(dyn NovelTermRepository + Send + Sync),
) -> Result<Vec<NovelTerm>, PromptFragmentError> {
    let query = NovelTermQuery {
        novel_id,
        term_type: None,
        name: None,
        description_keyword: None,
        page: 1,
        page_size: 0,
    };

    match repo.find_with_query(&query).await {
        Ok(result) => Ok(result.data),
        Err(err) => {
            error!(
                op = "render_novel_terms_fragment",
                stage = "fetch_all_terms",
                novel_id = %novel_id,
                error = %err,
                "查询小说名词失败"
            );
            Err(PromptFragmentError::Db(err))
        }
    }
}

/// 按 [`ALL_TERM_TYPES`] 声明顺序将名词分组
///
/// - 每个分组内部保持输入顺序（Repo 已按 `created_at ASC, id ASC` 稳定排序）；
/// - 命中为空的类型不会出现在结果中。
fn group_terms_by_type(terms: Vec<NovelTerm>) -> Vec<(TermType, Vec<NovelTerm>)> {
    let mut buckets: Vec<(TermType, Vec<NovelTerm>)> =
        ALL_TERM_TYPES.iter().map(|ty| (*ty, Vec::new())).collect();

    for term in terms {
        if let Some(bucket) = buckets.iter_mut().find(|(ty, _)| *ty == term.term_type) {
            bucket.1.push(term);
        }
    }

    buckets
        .into_iter()
        .filter(|(_, list)| !list.is_empty())
        .collect()
}

/// 由分组结果构造 Tera 模板上下文
///
/// - 使用 [`TermType::label`] 作为分组标题；
/// - 当 `full_desc_map` 中存在某名词 ID 对应的非空字符串时，使用该"完整描述"
///   覆盖名词主表 `description`；否则按既有逻辑取主表 `description`；
/// - `description` 为 `None` 或 trim 后为空时归一化为空字符串，
///   由模板判断是否省略末尾描述段。
fn build_template_context(
    grouped: Vec<(TermType, Vec<NovelTerm>)>,
    show_id: bool,
    full_desc_map: &HashMap<Uuid, String>,
) -> NovelTermsContext {
    let groups = grouped
        .into_iter()
        .map(|(ty, items)| NovelTermGroup {
            type_label: ty.label().to_string(),
            items: items
                .into_iter()
                .map(|term| to_template_item(term, full_desc_map))
                .collect(),
        })
        .collect();

    NovelTermsContext { groups, show_id }
}

/// 单条名词转换为模板条目（归一化描述字段）
///
/// `full_desc_map` 中存在且非空的条目优先使用"完整描述"覆盖。
fn to_template_item(term: NovelTerm, full_desc_map: &HashMap<Uuid, String>) -> NovelTermItem {
    let full = full_desc_map
        .get(&term.id)
        .map(String::as_str)
        .unwrap_or("");
    let description = if !full.is_empty() {
        full.to_string()
    } else {
        term.description
            .as_deref()
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .unwrap_or("")
            .to_string()
    };

    NovelTermItem {
        id: term.id.to_string(),
        name: term.name,
        description,
    }
}

/// 现场实例化 `PromptTemplates` 并执行渲染，错误同样写入 tracing 日志
fn render_with_templates(
    templates_root: &Path,
    context: &NovelTermsContext,
) -> Result<String, PromptFragmentError> {
    let templates = PromptTemplates::new(templates_root).map_err(|err| {
        error!(
            op = "render_novel_terms_fragment",
            stage = "load_templates",
            error = %err,
            "加载 Tera 模板失败"
        );
        PromptFragmentError::Render(err)
    })?;

    templates.render_novel_terms(context).map_err(|err| {
        error!(
            op = "render_novel_terms_fragment",
            stage = "render",
            error = %err,
            "渲染 novel_terms 模板失败"
        );
        PromptFragmentError::Render(err)
    })
}

/// 按小说 ID 一次性反查所有名词的章节关联记录，按 `term_id` 分组拼接为"完整描述"
///
/// - 单条关联描述为 `None` 或 trim 后为空时跳过，不贡献空段；
/// - 多条按 SQL 排序（卷序 ASC, 章序 ASC, created_at ASC）后用中文句号 `。` join；
/// - 某名词若全部关联均为空，map 中**不**写入对应 key，由调用方走"主表 description"兜底；
/// - 仅当主流程判断需要才会被调用，外部为 `false` 时零 SQL 开销。
async fn build_full_descriptions_map(
    novel_id: Uuid,
    repo: &(dyn ChapterTermRelationRepository + Send + Sync),
) -> Result<HashMap<Uuid, String>, PromptFragmentError> {
    let relations = fetch_term_chapter_relations(novel_id, repo).await?;
    Ok(group_relations_to_full_descriptions(relations))
}

/// 调用 repo 拉取关联记录，错误统一映射为 `PromptFragmentError::Db`
async fn fetch_term_chapter_relations(
    novel_id: Uuid,
    repo: &(dyn ChapterTermRelationRepository + Send + Sync),
) -> Result<Vec<ChapterTermRelationWithChapter>, PromptFragmentError> {
    repo.find_term_chapter_relations_by_novel(&novel_id)
        .await
        .map_err(|err| {
            error!(
                op = "render_novel_terms_fragment",
                stage = "fetch_chapter_relations",
                novel_id = %novel_id,
                error = %err,
                "查询小说名词章节关联失败"
            );
            PromptFragmentError::Db(err)
        })
}

/// 按 `term_id` 分组拼接非空描述；最终为空字符串的 key 不会出现在结果中
fn group_relations_to_full_descriptions(
    relations: Vec<ChapterTermRelationWithChapter>,
) -> HashMap<Uuid, String> {
    let mut buckets: HashMap<Uuid, Vec<String>> = HashMap::new();
    for rel in relations {
        let desc = rel
            .description
            .as_deref()
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .map(str::to_string);
        if let Some(text) = desc {
            buckets.entry(rel.term_id).or_default().push(text);
        }
    }
    buckets
        .into_iter()
        .map(|(term_id, parts)| (term_id, parts.join("。")))
        .collect()
}
