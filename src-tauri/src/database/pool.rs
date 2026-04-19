use sqlx::SqlitePool;
use tracing::info;

use crate::config::paths::get_database_path;
use crate::database::error::DbError;

pub async fn init_pool() -> Result<SqlitePool, DbError> {
    let db_path = get_database_path()?;

    info!("数据库路径: {}", db_path.display());

    let options = sqlx::sqlite::SqliteConnectOptions::new()
        .filename(&db_path)
        .create_if_missing(true);

    let pool = SqlitePool::connect_with(options).await?;
    info!("数据库连接池创建成功");

    // 执行数据库迁移（会自动创建表结构）
    sqlx::migrate!("./migrations").run(&pool).await?;
    info!("数据库迁移执行完成");

    Ok(pool)
}
