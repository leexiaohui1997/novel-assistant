/// 应用配置结构体
///
/// 用于管理应用的所有配置项，从环境变量中读取
#[derive(Debug, Clone)]
pub struct AppConfig {}

impl AppConfig {
    /// 从环境变量加载配置
    ///
    /// 加载顺序：
    /// 1. 先加载默认的 .env 文件
    /// 2. 再加载特定环境的 .env.{environment} 文件（会覆盖默认值）
    pub fn load() -> Self {
        // 加载默认配置文件
        dotenvy::dotenv().ok();

        // 获取当前环境（默认为 development）
        let env = std::env::var("APP_ENV").unwrap_or_else(|_| "development".to_string());

        // 加载特定环境的配置文件（如果存在）
        let env_file = format!(".env.{}", env);
        dotenvy::from_filename(&env_file).ok();

        // 返回配置实例
        Self {}
    }
}

impl Default for AppConfig {
    fn default() -> Self {
        Self::load()
    }
}
