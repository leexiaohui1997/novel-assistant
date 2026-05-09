//! AI v2 工具函数集合
//!
//! 跨仓储 / 跨模块的辅助逻辑（不属于任何具体 service 的通用推导）。

pub mod recommended_model;

pub use recommended_model::find_recommended_model;
