//! Tera 模板管理器
//!
//! 负责扫描 `templates/v2/prompts/` 与 `templates/v2/skills/` 下的所有
//! `*.tera` 文件并按稳定 ID（`prompts/xxx`、`skills/xxx`）注册到内部
//! `tera::Tera` 实例，对外提供只读、线程安全的按 ID 渲染能力。

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use serde_json::Value;
use tera::Tera;
use tracing::warn;

use super::error::TemplateError;

/// 模板管理器（线程安全、只读）
///
/// 内部持有 `tera::Tera`。加载完成后不再变更，因此无需锁；
/// 可直接以 `Arc<TemplateManager>` 形式在各 Service 间共享。
pub struct TemplateManager {
    tera: Tera,
}

impl TemplateManager {
    /// 创建模板管理器
    ///
    /// 扫描 `root_dir/prompts/` 与 `root_dir/skills/` 下所有 `*.tera` 文件，
    /// 以 `"{category}/{relative_path_without_ext}"` 作为模板 ID 注册。
    pub fn new<P: AsRef<Path>>(root_dir: P) -> Result<Self, TemplateError> {
        let root = root_dir.as_ref();
        let mut templates: HashMap<String, PathBuf> = HashMap::new();

        for category in SUPPORTED_CATEGORIES {
            collect_category(root, category, &mut templates)?;
        }

        let tera = build_tera(&templates)?;
        Ok(Self { tera })
    }

    /// 按模板 ID 渲染（异步）
    ///
    /// - `template_id`：形如 `prompts/xxx`、`skills/xxx`；
    /// - `data`：必须为 JSON 对象，将被转换为 `tera::Context`。
    ///
    /// # 异步说明
    /// Tera 渲染本身是 CPU 同步操作，此处仅在签名上异步化，
    /// 以便 `TemplateInstance` 等上层链路可在异步上下文中混合 IO（如 DB 查询）。
    pub async fn render(&self, template_id: &str, data: &Value) -> Result<String, TemplateError> {
        if !self.has_template(template_id) {
            return Err(TemplateError::NotFound {
                id: template_id.to_string(),
            });
        }
        let ctx = build_context(data)?;
        let rendered = self.tera.render(template_id, &ctx)?;
        Ok(rendered)
    }

    /// 判断给定模板 ID 是否已注册
    fn has_template(&self, template_id: &str) -> bool {
        self.tera
            .get_template_names()
            .any(|name| name == template_id)
    }
}

/// 将 JSON 对象转换为 `tera::Context`；非对象直接返回 `InvalidContext`
fn build_context(data: &Value) -> Result<tera::Context, TemplateError> {
    let obj = data
        .as_object()
        .ok_or_else(|| TemplateError::InvalidContext("模板上下文必须为 JSON 对象".to_string()))?;
    let mut ctx = tera::Context::new();
    for (k, v) in obj {
        ctx.insert(k, v);
    }
    Ok(ctx)
}

/// 当前支持的模板分类；未来新增仅需扩展此常量
const SUPPORTED_CATEGORIES: &[&str] = &["prompts", "skills"];

/// 收集单个分类目录下所有 `.tera` 文件并注册到 `templates` 映射
fn collect_category(
    root: &Path,
    category: &str,
    templates: &mut HashMap<String, PathBuf>,
) -> Result<(), TemplateError> {
    let category_dir = root.join(category);
    if !category_dir.exists() {
        warn!(
            category,
            path = %category_dir.display(),
            "模板分类目录不存在，已跳过"
        );
        return Ok(());
    }

    let files = collect_tera_files(&category_dir)?;
    for file in files {
        let id = build_template_id(category, &category_dir, &file)?;
        if let Some(existing) = templates.insert(id.clone(), file.clone()) {
            return Err(TemplateError::InvalidContext(format!(
                "模板 ID 冲突：{}（{} 与 {}）",
                id,
                existing.display(),
                file.display()
            )));
        }
    }
    Ok(())
}

/// 递归收集目录下所有 `.tera` 文件的绝对路径
fn collect_tera_files(dir: &Path) -> Result<Vec<PathBuf>, TemplateError> {
    let mut result = Vec::new();
    walk_tera_files(dir, &mut result)?;
    Ok(result)
}

/// 深度优先遍历目录，收集 `.tera` 文件
fn walk_tera_files(dir: &Path, out: &mut Vec<PathBuf>) -> Result<(), TemplateError> {
    for entry in std::fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            walk_tera_files(&path, out)?;
        } else if is_tera_file(&path) {
            out.push(path);
        }
    }
    Ok(())
}

/// 判断路径是否为 `.tera` 模板文件
fn is_tera_file(path: &Path) -> bool {
    path.extension().and_then(|s| s.to_str()) == Some("tera")
}

/// 根据分类名与文件路径生成模板 ID（`category/relative_without_ext`）
fn build_template_id(
    category: &str,
    category_dir: &Path,
    file: &Path,
) -> Result<String, TemplateError> {
    let rel = file
        .strip_prefix(category_dir)
        .map_err(|e| TemplateError::InvalidContext(format!("无法计算模板相对路径: {}", e)))?;
    let without_ext = rel.with_extension("");
    let rel_str = without_ext
        .to_str()
        .ok_or_else(|| {
            TemplateError::InvalidContext(format!("模板路径包含非 UTF-8 字符: {}", file.display()))
        })?
        // 统一为正斜杠，保证 Windows 下 ID 与 *nix 一致
        .replace('\\', "/");
    Ok(format!("{}/{}", category, rel_str))
}

/// 基于收集到的模板映射构建 `Tera` 实例
fn build_tera(templates: &HashMap<String, PathBuf>) -> Result<Tera, TemplateError> {
    let mut tera = Tera::default();
    // Tera 的 add_template_files 接收 (路径, Option<模板名>) 列表
    let files: Vec<(PathBuf, Option<&str>)> = templates
        .iter()
        .map(|(id, path)| (path.clone(), Some(id.as_str())))
        .collect();
    tera.add_template_files(files)?;
    Ok(tera)
}
