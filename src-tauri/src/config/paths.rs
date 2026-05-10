use std::path::PathBuf;
use tauri::AppHandle;
#[cfg(not(debug_assertions))]
use tauri::Manager;
use tracing::info;

use crate::database::error::DbError;

/// 获取应用数据目录
///
/// 返回平台特定的应用数据目录路径，例如：
/// - macOS: ~/Library/Application Support/com.novel-assistant.app
/// - Linux: ~/.local/share/com.novel-assistant.app
/// - Windows: C:\Users\{user}\AppData\Roaming\com.novel-assistant.app
pub fn get_app_data_dir() -> Result<PathBuf, DbError> {
    let base_dir = dirs::data_dir().ok_or(DbError::DataDirNotFound)?;

    // 根据环境选择不同的子目录
    #[cfg(debug_assertions)]
    let app_data_dir = base_dir.join("com.novel-assistant.dev");

    #[cfg(not(debug_assertions))]
    let app_data_dir = base_dir.join("com.novel-assistant");

    // 确保目录存在
    std::fs::create_dir_all(&app_data_dir).map_err(|e| {
        tracing::error!("无法创建数据目录 {:?}: {}", app_data_dir, e);
        DbError::CreateDirFailed(e)
    })?;

    // info!("应用数据目录就绪: {:?}", app_data_dir);
    Ok(app_data_dir)
}

/// 获取数据库文件路径
pub fn get_database_path() -> Result<PathBuf, DbError> {
    let app_data_dir = get_app_data_dir()?;
    info!("应用数据目录就绪: {:?}", app_data_dir);
    Ok(app_data_dir.join("novels.db"))
}

/// 获取模板根目录（`templates/`）
///
/// - **dev 模式**：使用源码目录 `src-tauri/templates`（通过
///   `CARGO_MANIFEST_DIR` 在编译期定位），保证改模板即时生效。
/// - **prod 模式**：使用 Tauri `resource_dir()`，需在
///   `tauri.conf.json` 中配置 `bundle.resources`。
pub fn get_templates_base(app: &AppHandle) -> Result<PathBuf, DbError> {
    #[cfg(debug_assertions)]
    {
        let _ = app;
        Ok(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("templates"))
    }

    #[cfg(not(debug_assertions))]
    {
        let resource_dir = app
            .path()
            .resource_dir()
            .map_err(|e| DbError::Business(format!("无法获取资源目录: {}", e)))?;
        Ok(resource_dir.join("templates"))
    }
}

/// 获取 AI v2 Tera 模板根目录（`templates/v2`）
pub fn get_template_root(app: &AppHandle) -> Result<PathBuf, DbError> {
    Ok(get_templates_base(app)?.join("v2"))
}
