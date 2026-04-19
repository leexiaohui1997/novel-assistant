use thiserror::Error;

#[derive(Error, Debug)]
pub enum DbError {
    #[error("数据库连接失败: {0}")]
    ConnectionFailed(#[from] sqlx::Error),

    #[error("无法获取应用数据目录")]
    DataDirNotFound,

    #[error("无法创建数据库目录: {0}")]
    CreateDirFailed(std::io::Error),

    #[error("迁移执行失败: {0}")]
    MigrationFailed(#[from] sqlx::migrate::MigrateError),
}
