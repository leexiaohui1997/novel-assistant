use async_trait::async_trait;
use uuid::Uuid;

use crate::database::error::DbError;
use crate::database::models::chapter_term_relation::{
    ChapterTermRelation, ChapterTermRelationQuery, ChapterTermRelationWithChapter,
    ChapterTermRelationWithTerm, NewChapterTermRelation, UpdateChapterTermRelation,
};
use crate::utils::pagination::PaginatedResult;

/// 小说名词-章节关联仓储 trait
#[async_trait]
pub trait ChapterTermRelationRepository {
    /// 创建关联
    async fn create(
        &self,
        relation: &NewChapterTermRelation,
    ) -> Result<ChapterTermRelation, DbError>;

    /// 根据ID查询关联
    async fn find_by_id(&self, id: &Uuid) -> Result<Option<ChapterTermRelation>, DbError>;

    /// 更新关联（仅更新description）
    async fn update(
        &self,
        id: &Uuid,
        update: &UpdateChapterTermRelation,
    ) -> Result<ChapterTermRelation, DbError>;

    /// 删除关联
    async fn delete(&self, id: &Uuid) -> Result<(), DbError>;

    /// 根据章节ID和名词ID查询（用于检查是否已存在）
    async fn find_by_chapter_and_term(
        &self,
        chapter_id: &Uuid,
        term_id: &Uuid,
    ) -> Result<Option<ChapterTermRelation>, DbError>;

    /// 组合查询（支持筛选、分页、排序）
    async fn find_with_query(
        &self,
        query: &ChapterTermRelationQuery,
    ) -> Result<PaginatedResult<ChapterTermRelation>, DbError>;

    /// 删除指定章节的所有关联
    async fn delete_by_chapter_id(&self, chapter_id: &Uuid) -> Result<(), DbError>;

    /// 删除指定名词的所有关联
    async fn delete_by_term_id(&self, term_id: &Uuid) -> Result<(), DbError>;

    /// 同步章节ID：将指定小说下 chapter_id 为空的记录更新为新章节ID
    async fn sync_chapter_id(&self, novel_id: &Uuid, new_chapter_id: &Uuid)
        -> Result<u64, DbError>;

    /// 按小说ID和可选章节ID查询名词列表（带关联信息）
    async fn find_terms_by_novel_and_chapter(
        &self,
        novel_id: &Uuid,
        chapter_id: Option<&Uuid>,
    ) -> Result<Vec<ChapterTermRelationWithTerm>, DbError>;

    /// 批量删除指定小说和章节的关联
    async fn delete_by_novel_and_chapter(
        &self,
        novel_id: &Uuid,
        chapter_id: Option<&Uuid>,
    ) -> Result<u64, DbError>;

    /// 按名词 ID 反查其在各章节中的引用列表（带章节标题、章节序号、分卷序号）
    ///
    /// 仅返回 `chapter_id IS NOT NULL` 的关联，按 `volume_sequence ASC, chapter_sequence ASC` 排序。
    async fn find_chapters_by_term(
        &self,
        term_id: &Uuid,
    ) -> Result<Vec<ChapterTermRelationWithChapter>, DbError>;

    /// 按小说 ID 全量反查所有名词的章节关联记录（带章节标题、章节序号、分卷序号）
    ///
    /// 用于"完整描述"装配场景：一次 SQL 拉回该小说下所有名词在所有章节中的描述记录，
    /// 上层在 Rust 侧按 `term_id` 分组拼接，避免逐名词调用 [`Self::find_chapters_by_term`]
    /// 引发的 N+1 查询。
    ///
    /// - 仅返回 `chapter_id IS NOT NULL` 的关联；
    /// - 排序为 `volume_sequence ASC, chapter_sequence ASC, c.created_at ASC`，
    ///   与 [`Self::find_chapters_by_term`] 一致，保证后续拼接顺序稳定。
    async fn find_term_chapter_relations_by_novel(
        &self,
        novel_id: &Uuid,
    ) -> Result<Vec<ChapterTermRelationWithChapter>, DbError>;
}
