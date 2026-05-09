//! 「小说基础信息」提示模板实例
//!
//! 渲染当前小说的基础信息（书名、频道、标签、简介）作为对话上下文。
//! 由于需要按标签 ID 查询标签实体，本实例持有 `TagRepository`，并覆写
//! `TemplateInstance::render` 在校验后注入查库与格式化逻辑。

use std::sync::Arc;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use tokio::sync::RwLock;
use validator::Validate;

use crate::ai_v2::template::{TemplateError, TemplateInstance, TemplateManager};
use crate::database::repositories::TagRepository;
use crate::utils::formatters::{format_channel, format_tags};

/// 小说基础信息模板入参
#[derive(Debug, Clone, Deserialize, Serialize, Validate)]
pub struct NovelBasicParams {
    /// 书名
    #[serde(default)]
    pub title: Option<String>,

    /// 频道：male / female / both
    #[validate(custom = "validate_channel")]
    #[serde(default)]
    pub channel: Option<String>,

    /// 标签 ID 列表
    #[serde(default)]
    pub tag_ids: Option<Vec<i64>>,

    /// 简介
    #[serde(default)]
    pub introduction: Option<String>,
}

/// 频道值校验：仅允许 male/female/both
fn validate_channel(value: &str) -> Result<(), validator::ValidationError> {
    match value {
        "male" | "female" | "both" => Ok(()),
        _ => Err(validator::ValidationError::new("invalid_channel")),
    }
}

/// 标签仓储别名（与 AppState 保持一致）
type TagRepo = Arc<RwLock<Box<dyn TagRepository + Send + Sync>>>;

/// 「小说基础信息」模板实例
pub struct NovelBasicInstance {
    tag_repo: TagRepo,
}

impl NovelBasicInstance {
    pub fn new(tag_repo: TagRepo) -> Self {
        Self { tag_repo }
    }

    /// 按标签 ID 列表查询并格式化为标签文案
    async fn build_tags_label(
        &self,
        tag_ids: &Option<Vec<i64>>,
    ) -> Result<Option<String>, TemplateError> {
        let ids = match tag_ids {
            Some(v) if !v.is_empty() => v,
            _ => return Ok(None),
        };
        let repo = self.tag_repo.read().await;
        let tags = repo
            .find_by_ids(ids)
            .await
            .map_err(|e| TemplateError::InvalidContext(format!("查询标签失败: {}", e)))?;
        if tags.is_empty() {
            Ok(None)
        } else {
            Ok(Some(format_tags(&tags)))
        }
    }
}

#[async_trait]
impl TemplateInstance for NovelBasicInstance {
    const TEMPLATE_ID: &'static str = "prompts/novel_basic";
    type Params = NovelBasicParams;

    async fn render(
        &self,
        manager: &TemplateManager,
        data: Value,
    ) -> Result<String, TemplateError> {
        let params = self.validate(&data)?;

        let channel_label = params.channel.as_deref().map(format_channel);
        let tags_label = self.build_tags_label(&params.tag_ids).await?;

        let ctx = json!({
            "title": params.title,
            "channel_label": channel_label,
            "tags_label": tags_label,
            "introduction": params.introduction,
        });

        manager.render(Self::TEMPLATE_ID, &ctx).await
    }
}
