use sqlx::SqlitePool;

/// 工具执行上下文
///
/// 为所有 AI 工具提供统一的依赖注入，如数据库连接池。
pub struct ToolContext {
    pub pool: SqlitePool,
}

impl ToolContext {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }
}
