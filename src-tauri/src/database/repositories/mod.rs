pub mod novel_repo;
pub mod tag_repo;

pub use novel_repo::{NovelRepository, SqliteNovelRepository};
pub use tag_repo::{SqliteTagRepository, TagRepository};
