//! 日志配置模块
//!
//! 提供日志系统的配置结构体和默认配置

use serde::{Deserialize, Serialize};

/// 日志轮转策略
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LogRotation {
    /// 每小时轮转
    Hourly,
    /// 每天轮转
    Daily,
    /// 永不轮转（单文件）
    Never,
}

impl Default for LogRotation {
    fn default() -> Self {
        Self::Daily
    }
}

/// 日志级别
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LogLevel {
    Error,
    Warn,
    Info,
    Debug,
    Trace,
}

impl Default for LogLevel {
    fn default() -> Self {
        Self::Info
    }
}

/// 日志配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogConfig {
    /// 日志目录名称
    pub dir_name: String,
    /// 日志文件前缀
    pub file_prefix: String,
    /// 日志文件后缀
    pub file_suffix: String,
    /// 日志轮转策略
    pub rotation: LogRotation,
    /// 控制台输出日志级别
    pub console_level: LogLevel,
    /// 文件输出日志级别
    pub file_level: LogLevel,
    /// 是否启用控制台彩色输出
    pub console_ansi: bool,
    /// 是否显示目标模块名
    pub with_target: bool,
    /// 是否显示线程 ID
    pub with_thread_ids: bool,
    /// 是否显示文件名
    pub with_file: bool,
    /// 是否显示行号
    pub with_line_number: bool,
}

impl Default for LogConfig {
    fn default() -> Self {
        Self {
            dir_name: "logs".to_string(),
            file_prefix: "novel-assistant".to_string(),
            file_suffix: "log".to_string(),
            rotation: LogRotation::Daily,
            console_level: LogLevel::Info,
            file_level: LogLevel::Info,
            console_ansi: true,
            with_target: true,
            with_thread_ids: false,
            with_file: true,
            with_line_number: true,
        }
    }
}

impl LogConfig {
    /// 获取开发环境配置
    pub fn development() -> Self {
        Self {
            console_level: LogLevel::Debug,
            ..Default::default()
        }
    }

    /// 获取生产环境配置
    pub fn production() -> Self {
        Self {
            console_level: LogLevel::Info,
            file_level: LogLevel::Info,
            console_ansi: false, // 生产环境关闭彩色输出
            ..Default::default()
        }
    }
}
