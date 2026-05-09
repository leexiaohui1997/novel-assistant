//! `prompts` 类目下的模板实例集合
//!
//! 与 `templates/v2/prompts/` 物理目录一一对应，每个 `.tera` 模板对应
//! 此处一个模块。
//!
//! 当前包含：
//! - [`novel_basic`]：渲染当前小说基础信息（书名 / 频道 / 标签 / 简介）

pub mod novel_basic;

pub use novel_basic::{NovelBasicInstance, NovelBasicParams};
