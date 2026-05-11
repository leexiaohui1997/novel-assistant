use async_trait::async_trait;
use uuid::Uuid;

use crate::database::error::DbError;
use crate::database::models::chapter_term_relation::{
    ChapterTermRelation, ChapterTermRelationQuery, NewChapterTermRelation,
    UpdateChapterTermRelation,
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
}
