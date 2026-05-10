//! 日志查询相关的 Tauri 命令
//!
//! 提供日志文件列表查询和日志内容读取功能

use serde::Serialize;
use std::fs;

use crate::config::paths::get_app_data_dir;
use crate::logging::LogConfig;

/// 日志文件信息
#[derive(Debug, Clone, Serialize)]
pub struct LogFileInfo {
    /// 文件名
    pub filename: String,
    /// 文件大小（字节）
    pub size: u64,
    /// 修改时间（Unix 时间戳，毫秒）
    pub modified: i64,
}

/// 获取日志文件列表
///
/// 从应用数据目录下的日志目录中读取所有日志文件信息，
/// 按修改时间倒序排列（最新的在前）
///
/// # 返回
/// - `Ok(Vec<LogFileInfo>)`: 日志文件信息列表
/// - `Err(String)`: 错误信息
#[tauri::command]
pub async fn get_log_files() -> Result<Vec<LogFileInfo>, String> {
    // 获取应用数据目录
    let app_data_dir = get_app_data_dir().map_err(|e| format!("获取应用数据目录失败: {}", e))?;

    // 从配置中读取日志目录名称
    let config = LogConfig::default();
    let logs_dir = app_data_dir.join(&config.dir_name);

    // 检查日志目录是否存在
    if !logs_dir.exists() {
        return Ok(vec![]);
    }

    // 读取目录内容
    let entries =
        fs::read_dir(&logs_dir).map_err(|e| format!("读取日志目录失败 {:?}: {}", logs_dir, e))?;

    let mut log_files: Vec<LogFileInfo> = Vec::new();

    for entry in entries {
        let entry = entry.map_err(|e| format!("读取目录项失败: {}", e))?;
        let path = entry.path();

        // 只处理文件
        if !path.is_file() {
            continue;
        }

        // 获取文件名
        let filename = match path.file_name().and_then(|n| n.to_str()) {
            Some(name) => name.to_string(),
            None => continue,
        };

        // 过滤出符合配置的日志文件（以前缀开头、以后缀结尾）
        if !filename.starts_with(&config.file_prefix) || !filename.ends_with(&config.file_suffix) {
            continue;
        }

        // 获取文件元数据
        let metadata = match fs::metadata(&path) {
            Ok(meta) => meta,
            Err(_) => continue,
        };

        // 获取文件大小
        let size = metadata.len();

        // 获取修改时间
        let modified = metadata
            .modified()
            .ok()
            .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
            .map(|d| d.as_millis() as i64)
            .unwrap_or(0);

        log_files.push(LogFileInfo {
            filename,
            size,
            modified,
        });
    }

    // 按修改时间倒序排列（最新的在前）
    log_files.sort_by(|a, b| b.modified.cmp(&a.modified));

    Ok(log_files)
}

/// 获取指定日志文件的内容
///
/// # 参数
/// - `filename`: 日志文件名
///
/// # 返回
/// - `Ok(String)`: 日志文件内容（纯文本）
/// - `Err(String)`: 错误信息
#[tauri::command]
pub async fn get_log_file_content(filename: String) -> Result<String, String> {
    // 验证文件名安全性：防止路径遍历攻击
    if filename.contains("..") || filename.contains('/') || filename.contains('\\') {
        return Err("非法的文件名".to_string());
    }

    // 获取应用数据目录
    let app_data_dir = get_app_data_dir().map_err(|e| format!("获取应用数据目录失败: {}", e))?;

    // 从配置中读取日志目录名称和文件命名规则
    let config = LogConfig::default();

    // 验证文件名符合配置的命名规则
    if !filename.starts_with(&config.file_prefix) || !filename.ends_with(&config.file_suffix) {
        return Err(format!(
            "文件名不符合规范，应以 '{}' 开头并以 '{}' 结尾",
            config.file_prefix, config.file_suffix
        ));
    }

    // 构建完整文件路径
    let logs_dir = app_data_dir.join(&config.dir_name);
    let file_path = logs_dir.join(&filename);

    // 安全检查：确保文件路径在日志目录内
    let canonical_logs_dir = logs_dir
        .canonicalize()
        .map_err(|e| format!("无法解析日志目录路径 {:?}: {}", logs_dir, e))?;

    let canonical_file_path = file_path
        .canonicalize()
        .map_err(|e| format!("无法解析文件路径 {:?}: {}", file_path, e))?;

    if !canonical_file_path.starts_with(&canonical_logs_dir) {
        return Err("文件路径不在日志目录内".to_string());
    }

    // 读取文件内容
    let content = fs::read_to_string(&canonical_file_path)
        .map_err(|e| format!("读取日志文件失败 {:?}: {}", canonical_file_path, e))?;

    Ok(content)
}
