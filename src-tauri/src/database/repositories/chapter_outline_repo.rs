use async_trait::async_trait;
use uuid::Uuid;

use crate::database::error::DbError;
use crate::database::models::chapter_outline::ChapterOutline;
use crate::database::models::chapter_outline::ChapterOutlineWithCharacters;

/// 章节大纲仓储 trait
#[async_trait]
pub trait ChapterOutlineRepository {
    /// 根据 novel_id 和 chapter_id 查询大纲（chapter_id 可为空）
    async fn find_by_novel_and_chapter(
        &self,
        novel_id: &Uuid,
        chapter_id: Option<&Uuid>,
    ) -> Result<Option<ChapterOutline>, DbError>;

    /// 创建或更新大纲
    async fn upsert(&self, outline: &ChapterOutline) -> Result<ChapterOutline, DbError>;

    /// 查询大纲关联的角色 ID 列表
    async fn find_character_ids(&self, outline_id: i64) -> Result<Vec<Uuid>, DbError>;

    /// 全量同步大纲关联的角色（事务内先删后插）
    async fn sync_characters(&self, outline_id: i64, character_ids: &[Uuid])
        -> Result<(), DbError>;

    /// 查询大纲（含关联角色ID列表）
    async fn find_with_characters(
        &self,
        novel_id: &Uuid,
        chapter_id: Option<&Uuid>,
    ) -> Result<Option<ChapterOutlineWithCharacters>, DbError>;
}
