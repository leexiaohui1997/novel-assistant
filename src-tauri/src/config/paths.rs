use std::path::PathBuf;
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

    info!("应用数据目录就绪: {:?}", app_data_dir);
    Ok(app_data_dir)
}

/// 获取数据库文件路径
pub fn get_database_path() -> Result<PathBuf, DbError> {
    let app_data_dir = get_app_data_dir()?;
    Ok(app_data_dir.join("novels.db"))
}
