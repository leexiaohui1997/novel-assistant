//! Markdown 工具函数
//!
//! 基于 [`markdown`] crate（`markdown-rs`）对 Markdown 源文本进行 AST 解析，
//! 并以回调方式遍历其中的围栏代码块（fenced code block）。

use markdown::{mdast::Node, to_mdast, ParseOptions};
use thiserror::Error;

/// Markdown 工具函数可能产生的错误
#[derive(Debug, Error)]
pub enum MarkdownWalkError {
    /// Markdown 源文本解析失败，内部为 `markdown-rs` 的原始错误描述
    #[error("Markdown 解析失败: {0}")]
    ParseFailed(String),
}

/// 遍历 Markdown 文本中的所有围栏代码块，并按回调处理命中的代码块。
///
/// # 说明
///
/// 本函数基于 [`markdown-rs`](https://docs.rs/markdown) 解析 Markdown AST，
/// 对 AST 中所有 [`Node::Code`]（围栏 / 缩进式块级代码块）节点进行回调式处理。
///
/// # 参数
///
/// - `content`：Markdown 源文本。
/// - `label_filter`：筛选闭包，入参依次为 `label`、`info`，返回 `true` 时
///   本代码块将进入处理阶段；`label` 对应 AST 中 [`Code::lang`] 字段，
///   `info` 对应 [`Code::meta`] 字段，二者为 `None` 时均以 `""` 传入。
/// - `content_handle`：处理闭包，入参依次为 `label`、`info`、`content`，
///   仅当 `label_filter` 返回 `true` 时被调用。`content` 为代码块正文
///   （对应 [`Code::value`]，不含首尾围栏）。
///
/// # 返回
///
/// - `Ok(())`：遍历正常完成（包括空输入、无代码块输入）。
/// - `Err(MarkdownWalkError)`：Markdown 解析阶段出错。
///
/// # 示例
///
/// ```ignore
/// use novel_assistant_lib::utils::markdown::walk_code_blocks;
///
/// let md = "一些文本\n\n```outline\nHello\n```\n\n```other\nSkip\n```\n";
/// let mut hits: Vec<String> = Vec::new();
/// walk_code_blocks(
///     md,
///     |label, _info| label == "outline",
///     |_label, _info, content| hits.push(content.to_string()),
/// )
/// .unwrap();
/// assert_eq!(hits, vec!["Hello".to_string()]);
/// ```
///
/// [`Code::lang`]: markdown::mdast::Code::lang
/// [`Code::meta`]: markdown::mdast::Code::meta
/// [`Code::value`]: markdown::mdast::Code::value
pub fn walk_code_blocks<F, H>(
    content: &str,
    label_filter: F,
    mut content_handle: H,
) -> Result<(), MarkdownWalkError>
where
    F: Fn(&str, &str) -> bool,
    H: FnMut(&str, &str, &str),
{
    let root = to_mdast(content, &ParseOptions::default())
        .map_err(|err| MarkdownWalkError::ParseFailed(err.to_string()))?;

    walk_node(&root, &label_filter, &mut content_handle);
    Ok(())
}

/// 递归遍历 AST 节点。
///
/// 仅拆分"命中 Code 节点"与"递归 children"两种分支，保持圈复杂度 < 5。
fn walk_node<F, H>(node: &Node, label_filter: &F, content_handle: &mut H)
where
    F: Fn(&str, &str) -> bool,
    H: FnMut(&str, &str, &str),
{
    if let Node::Code(code) = node {
        dispatch_code(code, label_filter, content_handle);
        return;
    }

    if let Some(children) = node.children() {
        for child in children {
            walk_node(child, label_filter, content_handle);
        }
    }
}

/// 将 `Code` 节点派发给调用方回调。
fn dispatch_code<F, H>(code: &markdown::mdast::Code, label_filter: &F, content_handle: &mut H)
where
    F: Fn(&str, &str) -> bool,
    H: FnMut(&str, &str, &str),
{
    let label = code.lang.as_deref().unwrap_or("");
    let info = code.meta.as_deref().unwrap_or("");
    if label_filter(label, info) {
        content_handle(label, info, &code.value);
    }
}
