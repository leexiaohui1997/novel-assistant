//! AI 技能管理模块
//!
//! 提供 AI 技能的抽象接口与注册中心。技能以"按需加载的提示词模板"形式存在，
//! 首轮仅将 `name + description + params_schema` 暴露给 AI，具体模板内容在后续
//! 阶段按需渲染。

mod instances;
mod registry;
mod service;
mod traits;

pub use instances::RefineNovelBasicSkill;
pub use registry::SkillRegistry;
pub use service::generate_skill_prompt;
pub use traits::AiSkill;
