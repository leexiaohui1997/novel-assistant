// 内置 Action 实现模块
//
// 包含系统预定义的 AI Actions，用户也可以注册自定义 Action。
//
// # 添加新 Action
//
// 1. 在 `builtin/` 目录下创建新文件，如 `my_action.rs`
// 2. 实现 `ActionHandler` trait
// 3. 在本文件中导出并注册到 Router

pub mod generate_introduction;
pub mod recommend_tags;

// 重新导出所有内置 Action
pub use generate_introduction::GenerateIntroductionAction;
pub use recommend_tags::RecommendTagsAction;
