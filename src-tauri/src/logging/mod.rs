//! 日志系统模块
//!
//! 提供可配置的日志记录功能，支持：
//! - 多输出目标（控制台 + 文件）
//! - 自动日志轮转
//! - 环境自适应配置（开发/生产）
//! - 结构化日志输出
//!
//! # 模块结构
//!
//! - `config` - 日志配置定义
//! - `setup` - 日志初始化逻辑
//!
//! # 使用示例
//!
//! ```rust,ignore
//! use novel_assistant_lib::{config::paths::get_app_data_dir, logging};
//!
//! let app_data_dir = get_app_data_dir()?;
//! let _guard = logging::init_logging_for_env(app_data_dir)?;
//! ```

pub mod config;
pub mod setup;

pub use config::LogConfig;
pub use setup::{init_logging, init_logging_for_env, LogError, LogResult};
