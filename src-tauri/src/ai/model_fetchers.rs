/// AI 供应商类型枚举
///
/// 用于标识不同的 AI 服务供应商，便于策略分发。
/// 目前只提供 Default 变体，后续按需扩展（如 Openai、Anthropic 等）。
use crate::string_enum;

string_enum! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub enum ProviderType {
        Default,
        SiliconFlow,
    }
}

/// 从供应商拉取到的模型信息
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ModelInfo {
    pub model_id: String,
    pub model_name: String,
}

/// 拉取模型时的错误类型
#[derive(thiserror::Error, Debug)]
pub enum FetchError {
    #[error("网络请求失败: {0}")]
    RequestFailed(String),

    #[error("响应解析失败: {0}")]
    ParseFailed(String),

    #[error("不支持的供应商类型: {0}")]
    UnsupportedProvider(String),
}

use std::collections::HashMap;

/// 模型拉取策略 trait
///
/// 不同供应商实现此 trait 以提供各自的拉取逻辑。
#[async_trait::async_trait]
pub trait FetchStrategy: Send + Sync {
    async fn fetch_models(
        &self,
        base_url: &str,
        api_key: &str,
    ) -> Result<Vec<ModelInfo>, FetchError>;
}

/// 默认拉取策略（OpenAI 兼容接口）
///
/// 使用 async-openai SDK 调用供应商的 `/v1/models` 端点拉取模型列表。
pub struct DefaultFetchStrategy;

#[async_trait::async_trait]
impl FetchStrategy for DefaultFetchStrategy {
    async fn fetch_models(
        &self,
        base_url: &str,
        api_key: &str,
    ) -> Result<Vec<ModelInfo>, FetchError> {
        use async_openai::config::OpenAIConfig;

        // 确保 base_url 不含尾随斜杠，避免 async-openai URL 拼接产生双斜杠
        let trimmed_base_url = base_url.trim_end_matches('/');

        let config = OpenAIConfig::new()
            .with_api_base(trimmed_base_url)
            .with_api_key(api_key);

        let client = async_openai::Client::with_config(config);

        client
            .models()
            .list()
            .await
            .map(|models| {
                models
                    .data
                    .into_iter()
                    .map(|m| ModelInfo {
                        model_id: m.id.clone(),
                        model_name: m.id,
                    })
                    .collect()
            })
            .map_err(|e| FetchError::RequestFailed(e.to_string()))
    }
}

/// SiliconFlow 拉取策略
///
/// 使用 reqwest 直接请求 SiliconFlow 的 `/v1/models` 端点，
/// 额外传递 `type=text&sub_type=chat` 查询参数以筛选聊天模型。
pub struct SiliconFlowFetchStrategy;

#[async_trait::async_trait]
impl FetchStrategy for SiliconFlowFetchStrategy {
    async fn fetch_models(
        &self,
        base_url: &str,
        api_key: &str,
    ) -> Result<Vec<ModelInfo>, FetchError> {
        let trimmed_base_url = base_url.trim_end_matches('/');
        let url = format!("{}/v1/models?type=text&sub_type=chat", trimmed_base_url);

        let response = reqwest::Client::new()
            .get(&url)
            .header("Authorization", format!("Bearer {}", api_key))
            .send()
            .await
            .map_err(|e| FetchError::RequestFailed(e.to_string()))?;

        if !response.status().is_success() {
            return Err(FetchError::RequestFailed(format!(
                "HTTP 状态码: {}",
                response.status()
            )));
        }

        #[derive(serde::Deserialize)]
        struct ModelsResponse {
            data: Vec<ModelItem>,
        }

        #[derive(serde::Deserialize)]
        struct ModelItem {
            id: String,
        }

        let body: ModelsResponse = response
            .json()
            .await
            .map_err(|e| FetchError::ParseFailed(e.to_string()))?;

        Ok(body
            .data
            .into_iter()
            .map(|m| ModelInfo {
                model_id: m.id.clone(),
                model_name: m.id,
            })
            .collect())
    }
}

/// 模型拉取策略注册表
///
/// 通过 HashMap 管理 ProviderType → FetchStrategy 的映射，
/// 便于运行时动态注册新的拉取策略。
pub struct FetcherRegistry {
    strategies: HashMap<ProviderType, Box<dyn FetchStrategy>>,
}

impl FetcherRegistry {
    /// 创建注册表并注册默认策略
    pub fn new() -> Self {
        let mut strategies: HashMap<ProviderType, Box<dyn FetchStrategy>> = HashMap::new();
        strategies.insert(ProviderType::Default, Box::new(DefaultFetchStrategy));
        strategies.insert(
            ProviderType::SiliconFlow,
            Box::new(SiliconFlowFetchStrategy),
        );
        Self { strategies }
    }

    /// 注册新的拉取策略
    pub fn register(&mut self, provider_type: ProviderType, strategy: Box<dyn FetchStrategy>) {
        self.strategies.insert(provider_type, strategy);
    }

    /// 根据供应商类型拉取模型列表
    pub async fn fetch(
        &self,
        provider_type: ProviderType,
        base_url: &str,
        api_key: &str,
    ) -> Result<Vec<ModelInfo>, FetchError> {
        let strategy = self
            .strategies
            .get(&provider_type)
            .ok_or_else(|| FetchError::UnsupportedProvider(provider_type.to_string()))?;
        strategy.fetch_models(base_url, api_key).await
    }
}
