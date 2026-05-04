// 章节大纲相关的 Tauri Commands

use serde::Deserialize;
use tauri::State;
use uuid::Uuid;

use crate::AppState;

/// 编辑章节大纲的输入参数
#[derive(Debug, Deserialize)]
pub struct EditChapterOutlineInput {
    /// 小说 ID
    pub novel_id: String,

    /// 章节 ID（可选，为空时表示整本小说的通用大纲）
    #[serde(default)]
    pub chapter_id: Option<String>,

    /// 大纲内容（可选，如果未提供则不执行任何操作）
    #[serde(default)]
    pub positioning: Option<String>,
}

/// 编辑章节大纲
///
/// # 参数
/// - `input`: 编辑输入参数
///
/// # 返回
/// - `Ok(serde_json::Value)`: 更新后的大纲信息
/// - `Err(String)`: 错误信息
#[tauri::command]
pub async fn edit_chapter_outline(
    state: State<'_, AppState>,
    input: EditChapterOutlineInput,
) -> Result<serde_json::Value, String> {
    // 1. 如果未提供 positioning，则直接中止
    let positioning = input.positioning.ok_or("positioning 字段不能为空")?;

    // 2. 解析 novel_id
    let novel_uuid =
        Uuid::parse_str(&input.novel_id).map_err(|e| format!("小说 ID 格式错误: {}", e))?;

    // 3. 解析 chapter_id（如果提供）
    let chapter_uuid = input
        .chapter_id
        .map(|id| Uuid::parse_str(&id).map_err(|e| format!("章节 ID 格式错误: {}", e)))
        .transpose()?;

    // 4. 查询是否已有记录
    let outline_repo = state.chapter_outline_repo.read().await;
    let existing = outline_repo
        .find_by_novel_and_chapter(&novel_uuid, chapter_uuid.as_ref())
        .await
        .map_err(|e| format!("查询大纲失败: {}", e))?;
    drop(outline_repo);

    // 5. 构建或更新大纲对象
    use crate::database::models::chapter_outline::ChapterOutline;
    use chrono::Utc;

    let outline = if let Some(existing_outline) = existing {
        // 已有记录，更新 positioning
        ChapterOutline {
            id: existing_outline.id,
            novel_id: novel_uuid,
            chapter_id: chapter_uuid,
            positioning,
            created_at: existing_outline.created_at,
            updated_at: Utc::now(),
        }
    } else {
        // 未有记录，创建新记录
        ChapterOutline {
            id: 0, // 将在 upsert 中设置
            novel_id: novel_uuid,
            chapter_id: chapter_uuid,
            positioning,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    };

    // 6. 执行 upsert
    let result = state
        .chapter_outline_repo
        .read()
        .await
        .upsert(&outline)
        .await
        .map_err(|e| format!("保存大纲失败: {}", e))?;

    // 7. 返回结果
    Ok(serde_json::to_value(result).map_err(|e| format!("序列化结果失败: {}", e))?)
}

/// 获取章节大纲的输入参数
#[derive(Debug, Deserialize)]
pub struct GetChapterOutlineInput {
    /// 小说 ID
    pub novel_id: String,

    /// 章节 ID（可选，为空时表示获取整本小说的通用大纲）
    #[serde(default)]
    pub chapter_id: Option<String>,
}

/// 获取章节大纲
///
/// # 参数
/// - `input`: 查询输入参数
///
/// # 返回
/// - `Ok(Option<serde_json::Value>)`: 大纲信息，如果不存在则返回 null
/// - `Err(String)`: 错误信息
#[tauri::command]
pub async fn get_chapter_outline(
    state: State<'_, AppState>,
    input: GetChapterOutlineInput,
) -> Result<Option<serde_json::Value>, String> {
    // 1. 解析 novel_id
    let novel_uuid =
        Uuid::parse_str(&input.novel_id).map_err(|e| format!("小说 ID 格式错误: {}", e))?;

    // 2. 解析 chapter_id（如果提供）
    let chapter_uuid = input
        .chapter_id
        .map(|id| Uuid::parse_str(&id).map_err(|e| format!("章节 ID 格式错误: {}", e)))
        .transpose()?;

    // 3. 查询大纲
    let outline_repo = state.chapter_outline_repo.read().await;
    let outline = outline_repo
        .find_by_novel_and_chapter(&novel_uuid, chapter_uuid.as_ref())
        .await
        .map_err(|e| format!("查询大纲失败: {}", e))?;
    drop(outline_repo);

    // 4. 返回结果
    match outline {
        Some(o) => Ok(Some(
            serde_json::to_value(o).map_err(|e| format!("序列化结果失败: {}", e))?,
        )),
        None => Ok(None),
    }
}
