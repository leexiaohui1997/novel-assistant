//! AI 技能注册中心

use std::collections::HashMap;
use std::sync::Arc;

use super::traits::AiSkill;

/// AI 技能注册中心
///
/// 负责管理所有可用的 AI 技能实例。
pub struct SkillRegistry {
    skills: HashMap<String, Arc<dyn AiSkill>>,
}

impl SkillRegistry {
    /// 创建一个新的空注册中心
    pub fn new() -> Self {
        Self {
            skills: HashMap::new(),
        }
    }

    /// 注册一个技能
    ///
    /// # 参数
    /// - `skill`: 实现了 [`AiSkill`] trait 的技能实例
    pub fn register(&mut self, skill: Arc<dyn AiSkill>) {
        let name = skill.name().to_string();
        self.skills.insert(name, skill);
    }

    /// 根据名称获取技能
    pub fn get_skill(&self, name: &str) -> Option<&Arc<dyn AiSkill>> {
        self.skills.get(name)
    }

    /// 获取所有已注册技能的列表（用于向 AI 模型提供技能元数据）
    pub fn list_skills(&self) -> Vec<Arc<dyn AiSkill>> {
        self.skills.values().cloned().collect()
    }

    /// 迭代器，遍历所有已注册的技能
    pub fn iter(&self) -> impl Iterator<Item = &Arc<dyn AiSkill>> {
        self.skills.values()
    }
}

impl Default for SkillRegistry {
    fn default() -> Self {
        Self::new()
    }
}
