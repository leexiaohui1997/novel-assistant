use async_trait::async_trait;
use uuid::Uuid;

use crate::database::error::DbError;
use crate::database::models::chapter_outline::ChapterOutline;

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
}
