//! `refine_novel_basic` 技能实现
//!
//! 用于在 AI 多轮会话中识别到「需要提供或修改小说书名、标签或简介」场景时按需装载。
//! 该技能装载时不接收任何参数，模板内部已固化输出约定。

use serde_json::{json, Value};

use crate::ai_v2::skills::AiSkill;
use async_trait::async_trait;

/// 小说基础信息优化技能（无参）
pub struct RefineNovelBasicSkill;

#[async_trait]
impl AiSkill for RefineNovelBasicSkill {
    fn name(&self) -> &str {
        "refine_novel_basic"
    }

    fn description(&self) -> &str {
        "需要提供或修改小说书名、标签、简介信息时调用（优化下文案）"
    }

    fn template_id(&self) -> &str {
        "skills/refine_novel_basic"
    }

    fn params_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {},
            "additionalProperties": false
        })
    }
}
