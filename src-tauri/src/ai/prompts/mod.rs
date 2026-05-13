// 提示词模板管理器
//
// 使用 Tera 模板引擎管理 AI 提示词模板

mod edit_chapter_characters;
mod edit_chapter_plot;
mod edit_chapter_positioning;
mod edit_chapter_title;
mod extract_terms;
pub mod fragments;
mod generate_chapter_content;
mod generate_character;
mod generate_introduction;
mod generate_title;
mod novel_terms;
mod optimize_character;
mod recommend_tags;
mod types;

use tera::{Context, Tera};

pub use edit_chapter_characters::EditChapterCharactersContext;
pub use edit_chapter_plot::EditChapterPlotContext;
pub use edit_chapter_positioning::EditChapterPositioningContext;
pub use edit_chapter_title::EditChapterTitleContext;
pub use extract_terms::ExtractTermsContext;
pub use fragments::{render_novel_terms_fragment, PromptFragmentError};
pub use generate_chapter_content::GenerateChapterContentContext;
pub use generate_character::CharacterTypeOption;
pub use generate_character::GenerateCharacterContext;
pub use generate_introduction::GenerateIntroductionContext;
pub use generate_title::GenerateTitleContext;
pub use novel_terms::{NovelTermGroup, NovelTermItem, NovelTermsContext};
pub use optimize_character::OptimizeCharacterContext;
pub use recommend_tags::RecommendTagsContext;
pub use types::{ChapterOutlineItem, CharacterDetail, CharacterInfo, CharacterWithIdInfo};

/// 提示词模板管理器
pub struct PromptTemplates {
    tera: Tera,
}

impl PromptTemplates {
    /// 创建新的提示词模板管理器
    ///
    /// # 参数
    /// - `templates_dir`: 模板根目录（如 `templates/`），内部会递归
    ///   扫描所有 `.tera` 文件。dev 态由 `CARGO_MANIFEST_DIR` 定位，
    ///   prod 态由 Tauri `resource_dir()` 定位，详见
    ///   [`crate::config::paths::get_templates_base`]。
    pub fn new(templates_dir: &std::path::Path) -> Result<Self, tera::Error> {
        let mut tera = Tera::default();

        let pattern = templates_dir.join("**/*.tera");
        let pattern_str = pattern
            .to_str()
            .ok_or_else(|| tera::Error::msg("模板根目录包含非 UTF-8 字符"))?;

        for entry in glob::glob(pattern_str)
            .map_err(|e| tera::Error::msg(format!("Failed to glob templates: {}", e)))?
        {
            match entry {
                Ok(path) => {
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
        let tera_context = Context::from_serialize(context)?;
        self.tera.render("recommend_tags", &tera_context)
    }

    /// 渲染 generate_introduction 提示词
    pub fn render_generate_introduction(
        &self,
        context: &GenerateIntroductionContext,
    ) -> Result<String, tera::Error> {
        let tera_context = Context::from_serialize(context)?;
        self.tera.render("generate_introduction", &tera_context)
    }

    /// 渲染 generate_title 提示词
    pub fn render_generate_title(
        &self,
        context: &GenerateTitleContext,
    ) -> Result<String, tera::Error> {
        let tera_context = Context::from_serialize(context)?;
        self.tera.render("generate_title", &tera_context)
    }

    /// 渲染 generate_character 提示词
    pub fn render_generate_character(
        &self,
        context: &GenerateCharacterContext,
    ) -> Result<String, tera::Error> {
        let tera_context = Context::from_serialize(context)?;
        self.tera.render("generate_character", &tera_context)
    }

    /// 渲染 optimize_character 提示词
    pub fn render_optimize_character(
        &self,
        context: &OptimizeCharacterContext,
    ) -> Result<String, tera::Error> {
        let tera_context = Context::from_serialize(context)?;
        self.tera.render("optimize_character", &tera_context)
    }

    /// 渲染 edit_chapter_positioning 提示词
    pub fn render_edit_chapter_positioning(
        &self,
        context: &EditChapterPositioningContext,
    ) -> Result<String, tera::Error> {
        let tera_context = Context::from_serialize(context)?;
        self.tera.render("edit_chapter_positioning", &tera_context)
    }

    /// 渲染 edit_chapter_plot 提示词
    pub fn render_edit_chapter_plot(
        &self,
        context: &EditChapterPlotContext,
    ) -> Result<String, tera::Error> {
        let tera_context = Context::from_serialize(context)?;
        self.tera.render("edit_chapter_plot", &tera_context)
    }

    /// 渲染 edit_chapter_characters 提示词
    pub fn render_edit_chapter_characters(
        &self,
        context: &EditChapterCharactersContext,
    ) -> Result<String, tera::Error> {
        let tera_context = Context::from_serialize(context)?;
        self.tera.render("edit_chapter_characters", &tera_context)
    }

    /// 渲染 edit_chapter_title 提示词
    pub fn render_edit_chapter_title(
        &self,
        context: &EditChapterTitleContext,
    ) -> Result<String, tera::Error> {
        let tera_context = Context::from_serialize(context)?;
        self.tera.render("edit_chapter_title", &tera_context)
    }

    /// 渲染 generate_chapter_content 提示词
    pub fn render_generate_chapter_content(
        &self,
        context: &GenerateChapterContentContext,
    ) -> Result<String, tera::Error> {
        let tera_context = Context::from_serialize(context)?;
        self.tera.render("generate_chapter_content", &tera_context)
    }

    /// 渲染 novel_terms 提示词片段
    ///
    /// 将已按类型分组的小说名词列表渲染为 Markdown 片段，
    /// 供上层提示词拼接。模板文件位于 `templates/fragments/novel_terms.tera`。
    pub fn render_novel_terms(&self, context: &NovelTermsContext) -> Result<String, tera::Error> {
        let tera_context = Context::from_serialize(context)?;
        self.tera.render("novel_terms", &tera_context)
    }

    /// 渲染 extract_terms 提示词
    pub fn render_extract_terms(
        &self,
        context: &ExtractTermsContext,
    ) -> Result<String, tera::Error> {
        let tera_context = Context::from_serialize(context)?;
        self.tera.render("extract_terms", &tera_context)
    }
}
