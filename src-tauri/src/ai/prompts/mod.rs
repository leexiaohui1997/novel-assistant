// 提示词模板管理器
//
// 使用 Tera 模板引擎管理 AI 提示词模板

use serde::Serialize;
use tera::{Context, Tera};

/// 提示词模板管理器
pub struct PromptTemplates {
    tera: Tera,
}

impl PromptTemplates {
    /// 创建新的提示词模板管理器
    pub fn new() -> Result<Self, tera::Error> {
        // 从 templates 目录自动加载所有 .tera 模板文件
        let mut tera = Tera::default();

        // 使用 glob 模式自动发现 templates 目录下的所有 .tera 文件
        // 模板名称会自动根据文件名生成（不含扩展名）
        // 例如: recommend_tags.tera -> "recommend_tags"
        let template_dir = concat!(env!("CARGO_MANIFEST_DIR"), "/templates/**/*.tera");

        for entry in glob::glob(template_dir)
            .map_err(|e| tera::Error::msg(format!("Failed to glob templates: {}", e)))?
        {
            match entry {
                Ok(path) => {
                    // 从文件路径提取模板名称（去除扩展名和路径）
                    if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
                        let name: Option<&str> = Some(stem);
                        tera.add_template_file(&path, name)?;
                    }
                }
                Err(e) => {
                    return Err(tera::Error::msg(format!("Failed to read template: {}", e)));
                }
            }
        }

        Ok(Self { tera })
    }

    /// 渲染 recommend_tags 提示词
    pub fn render_recommend_tags(
        &self,
        context: &RecommendTagsContext,
    ) -> Result<String, tera::Error> {
        // 使用 Serialize trait 自动转换，无需手动 insert
        let tera_context = Context::from_serialize(context)?;

        self.tera.render("recommend_tags", &tera_context)
    }

    /// 渲染 generate_introduction 提示词
    pub fn render_generate_introduction(
        &self,
        context: &GenerateIntroductionContext,
    ) -> Result<String, tera::Error> {
        // 使用 Serialize trait 自动转换，无需手动 insert
        let tera_context = Context::from_serialize(context)?;

        self.tera.render("generate_introduction", &tera_context)
    }

    /// 渲染 generate_title 提示词
    pub fn render_generate_title(
        &self,
        context: &GenerateTitleContext,
    ) -> Result<String, tera::Error> {
        // 使用 Serialize trait 自动转换，无需手动 insert
        let tera_context = Context::from_serialize(context)?;

        self.tera.render("generate_title", &tera_context)
    }

    /// 渲染 generate_character 提示词
    pub fn render_generate_character(
        &self,
        context: &GenerateCharacterContext,
    ) -> Result<String, tera::Error> {
        // 使用 Serialize trait 自动转换，无需手动 insert
        let tera_context = Context::from_serialize(context)?;

        self.tera.render("generate_character", &tera_context)
    }
}

/// recommend_tags 模板的上下文数据
#[derive(Debug, Serialize)]
pub struct RecommendTagsContext {
    /// 频道名称（男频/女频）
    pub channel_name: String,

    /// 标题（可选）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,

    /// 简介（可选）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub introduction: Option<String>,

    /// 已选标签信息（可选）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub selected_tags_info: Option<String>,

    /// 可用标签列表
    pub main_categories: String,
    pub themes: String,
    pub characters: String,
    pub plots: String,

    /// 限制数量
    pub main_limit: usize,
    pub theme_limit: usize,
    pub character_limit: usize,
    pub plot_limit: usize,

    /// 剩余可推荐数量
    pub main_remaining: usize,
    pub theme_remaining: usize,
    pub character_remaining: usize,
    pub plot_remaining: usize,
}

/// generate_introduction 模板的上下文数据
#[derive(Debug, Serialize)]
pub struct GenerateIntroductionContext {
    /// 标题（可选）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,

    /// 频道名称（男频/女频，可选）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub channel_name: Option<String>,

    /// 已选标签信息（可选）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub selected_tags: Option<String>,
}

/// generate_title 模板的上下文数据
#[derive(Debug, Serialize)]
pub struct GenerateTitleContext {
    /// 频道名称（男频/女频，可选）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub channel_name: Option<String>,

    /// 标签信息（可选）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tags: Option<String>,

    /// 作品简介（可选）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub introduction: Option<String>,
}

/// generate_character 模板的上下文数据
#[derive(Debug, Serialize)]
pub struct GenerateCharacterContext {
    /// 小说标题
    pub title: String,

    /// 频道名称（男频/女频）
    pub channel_name: String,

    /// 标签信息（可选）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tags: Option<String>,

    /// 作品简介（可选）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub introduction: Option<String>,

    /// 已有角色列表
    #[serde(skip_serializing_if = "Option::is_none")]
    pub existing_characters: Option<Vec<CharacterInfo>>,
}

/// 角色信息（用于提示词上下文）
#[derive(Debug, Serialize)]
pub struct CharacterInfo {
    /// 角色名称
    pub name: String,

    /// 性别
    pub gender: String,

    /// 背景（可选）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub background: Option<String>,
}
