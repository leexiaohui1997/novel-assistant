//! 日志初始化模块
//!
//! 负责日志系统的初始化和配置

use std::fs;
use std::path::PathBuf;

use tracing_appender::rolling::{RollingFileAppender, Rotation};
use tracing_subscriber::{fmt, layer::SubscriberExt, EnvFilter, Registry};

use crate::logging::config::LogConfig;

/// 日志系统错误类型
#[derive(Debug, thiserror::Error)]
pub enum LogError {
    #[error("无法创建日志目录: {0}")]
    CreateDirFailed(String),

    #[error("无法创建日志文件: {0}")]
    CreateLogFileFailed(String),

    #[error("设置日志订阅者失败: {0}")]
    SetGlobalDefaultFailed(String),
}

/// 日志系统初始化结果
pub type LogResult<T> = Result<T, LogError>;

/// 初始化日志系统
///
/// # 参数
/// * `config` - 日志配置
/// * `base_dir` - 应用数据目录（日志目录将在此目录下创建）
///
/// # 返回
/// 返回 `tracing-appender` 的 guard，必须保持存活以确保日志正确写入
///
/// # 示例
/// ```rust,ignore
/// use novel_assistant_lib::logging::{init_logging, config::LogConfig};
///
/// let config = LogConfig::development();
/// let _guard = init_logging(&config, app_data_dir)?;
/// ```
pub fn init_logging(
    config: &LogConfig,
    base_dir: PathBuf,
) -> LogResult<tracing_appender::non_blocking::WorkerGuard> {
    // 创建日志目录
    let logs_dir = base_dir.join(&config.dir_name);
    fs::create_dir_all(&logs_dir).map_err(|e| LogError::CreateDirFailed(e.to_string()))?;

    // 创建文件追加器
    let rotation = match config.rotation {
        crate::logging::config::LogRotation::Hourly => Rotation::HOURLY,
        crate::logging::config::LogRotation::Daily => Rotation::DAILY,
        crate::logging::config::LogRotation::Never => Rotation::NEVER,
    };

    let file_appender = RollingFileAppender::builder()
        .rotation(rotation)
        .filename_prefix(&config.file_prefix)
        .filename_suffix(&config.file_suffix)
        .build(&logs_dir)
        .map_err(|e| LogError::CreateLogFileFailed(e.to_string()))?;

    let (non_blocking, guard) = tracing_appender::non_blocking(file_appender);

    // 构建环境变量过滤器
    let env_filter = EnvFilter::from_default_env().add_directive(match config.console_level {
        crate::logging::config::LogLevel::Error => "novel_assistant=error".parse().unwrap(),
        crate::logging::config::LogLevel::Warn => "novel_assistant=warn".parse().unwrap(),
        crate::logging::config::LogLevel::Info => "novel_assistant=info".parse().unwrap(),
        crate::logging::config::LogLevel::Debug => "novel_assistant=debug".parse().unwrap(),
        crate::logging::config::LogLevel::Trace => "novel_assistant=trace".parse().unwrap(),
    });

    // 控制台层
    let console_layer = fmt::layer()
        .with_ansi(config.console_ansi)
        .with_target(config.with_target)
        .with_thread_ids(config.with_thread_ids)
        .with_file(config.with_file)
        .with_line_number(config.with_line_number);

    // 文件层
    let file_layer = fmt::layer()
        .with_ansi(false) // 文件始终不使用 ANSI
        .with_writer(non_blocking)
        .with_target(config.with_target)
        .with_thread_ids(config.with_thread_ids);

    // 组合订阅者
    let subscriber = Registry::default()
        .with(env_filter)
        .with(console_layer)
        .with(file_layer);

    tracing::subscriber::set_global_default(subscriber)
        .map_err(|e| LogError::SetGlobalDefaultFailed(e.to_string()))?;

    tracing::info!("日志系统初始化完成，日志目录: {:?}", logs_dir);

    Ok(guard)
}

/// 根据当前编译环境初始化日志系统
///
/// 自动获取应用数据目录并创建日志目录
/// 如果初始化失败，将打印错误信息并退出程序
pub fn init_logging_for_env() -> tracing_appender::non_blocking::WorkerGuard {
    use crate::config::paths::get_app_data_dir;

    // 获取应用数据目录
    let app_data_dir = match get_app_data_dir() {
        Ok(dir) => dir,
        Err(e) => {
            eprintln!("错误: 无法获取应用数据目录: {}", e);
            std::process::exit(1);
        }
    };

    // 根据环境选择配置
    #[cfg(debug_assertions)]
    let config = LogConfig::development();

    #[cfg(not(debug_assertions))]
    let config = LogConfig::production();

    match init_logging(&config, app_data_dir) {
        Ok(guard) => guard,
        Err(e) => {
            eprintln!("警告: 日志系统初始化失败: {}", e);
            // 日志初始化失败不应该阻止应用运行
            // 返回一个空 guard（虽然实际上不会用到）
            // 这里我们直接退出，因为没有日志的应用很难调试
            std::process::exit(1);
        }
    }
}
