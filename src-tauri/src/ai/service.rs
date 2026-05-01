use std::sync::Arc;
use std::time::Instant;
use tokio::sync::RwLock;

use crate::database::models::ai_call_log::CreateAiCallLog;
use crate::database::repositories::{AiCallLogRepository, ModelRepository, ProviderRepository};

use super::types::{AiChatResponse, AiRequestData, MessageRole};

/// AI 服务
///
/// 提供统一的 AI 能力接口，供后端各业务模块调用
pub struct AiService {
    model_repo: Arc<RwLock<Box<dyn ModelRepository + Send + Sync>>>,
    provider_repo: Arc<RwLock<Box<dyn ProviderRepository + Send + Sync>>>,
    call_log_repo: Arc<RwLock<Box<dyn AiCallLogRepository + Send + Sync>>>,
}

impl AiService {
    pub fn new(
        model_repo: Arc<RwLock<Box<dyn ModelRepository + Send + Sync>>>,
        provider_repo: Arc<RwLock<Box<dyn ProviderRepository + Send + Sync>>>,
        call_log_repo: Arc<RwLock<Box<dyn AiCallLogRepository + Send + Sync>>>,
    ) -> Self {
        Self {
            model_repo,
            provider_repo,
            call_log_repo,
        }
    }

    /// 获取第一个可用模型
    ///
    /// # 参数
    /// - `fallback_reason`: 回退原因（用于日志）
    ///
    /// # 返回
    /// - `Ok(Model)`: 第一个可用模型
    /// - `Err(String)`: 错误信息
    async fn get_first_available_model(
        &self,
        fallback_reason: &str,
    ) -> Result<crate::database::models::model::Model, String> {
        let available_models = self.get_available_models().await?;

        if available_models.is_empty() {
            return Err("没有可用的模型，请先添加模型".to_string());
        }

        let first_model = &available_models[0];
        tracing::warn!("{}，使用默认模型: {}", fallback_reason, first_model.id);

        Ok(first_model.clone())
    }

    /// 获取所有可用的模型列表
    ///
    /// 当前实现：从数据库中查询所有启用的模型
    /// TODO: 未来可以从配置或用户偏好中筛选
    ///
    /// # 返回
    /// - `Ok(Vec<Model>)`: 模型列表
    /// - `Err(String)`: 错误信息（如查询失败）
    pub async fn get_available_models(
        &self,
    ) -> Result<Vec<crate::database::models::model::Model>, String> {
        let repo = self.model_repo.read().await;

        // 查询所有启用的模型
        let models = repo
            .find_all(Some(true))
            .await
            .map_err(|e| format!("查询模型列表失败: {}", e))?;

        Ok(models)
    }

    /// 执行 AI 聊天调用
    ///
    /// # 参数
    /// - `request_data`: AI 请求数据（包含 model_id、messages）
    ///
    /// # 返回
    /// - `Ok(AiChatResponse)`: 包含 content 和原始 response 对象
    pub async fn chat(&self, request_data: AiRequestData) -> Result<AiChatResponse, String> {
        // 记录开始时间
        use chrono::Utc;
        let start_time = Instant::now();
        let call_time = Utc::now();

        // 1. 根据 model_id 从表里读出模型数据
        let model = {
            let repo = self.model_repo.read().await;

            // 如果提供了 model_id，尝试查询指定的模型
            if let Some(model_id) = request_data.model_id {
                match repo.find_by_id(model_id).await {
                    Ok(model) => model,
                    Err(_) => {
                        // 如果查询失败，使用第一个可用模型
                        drop(repo); // 释放读锁
                        self.get_first_available_model(&format!("指定的模型 {} 不存在", model_id))
                            .await?
                    }
                }
            } else {
                // 未提供 model_id，直接使用第一个可用模型
                drop(repo); // 释放读锁
                self.get_first_available_model("未指定模型").await?
            }
        };

        // 2 根据模型的供应商ID 从供应商表读出供应商数据
        let provider = {
            let repo = self.provider_repo.read().await;
            repo.find_by_id(model.provider_id)
                .await
                .map_err(|e| format!("查询供应商失败: {}", e))?
        };

        // 检查 API Key 是否存在
        let api_key = provider.api_key.ok_or("供应商未配置 API Key")?;

        // 3 创建 OpenAI 客户端
        use async_openai::types::{
            ChatCompletionRequestMessage, ChatCompletionRequestSystemMessage,
            ChatCompletionRequestUserMessage, CreateChatCompletionRequest,
        };
        use async_openai::{config::OpenAIConfig, Client};

        let config = OpenAIConfig::new()
            .with_api_key(api_key)
            .with_api_base(&provider.base_url);

        let client = Client::with_config(config);

        // 4 构建消息列表
        let mut messages: Vec<ChatCompletionRequestMessage> = vec![];

        // 将 AiRequestData 的 messages 转换为 OpenAI 格式
        for msg in &request_data.messages {
            match msg.role {
                super::types::MessageRole::System => {
                    messages.push(ChatCompletionRequestMessage::System(
                        ChatCompletionRequestSystemMessage {
                            content: msg.content.clone().into(),
                            name: None,
                        },
                    ));
                }
                super::types::MessageRole::User => {
                    messages.push(ChatCompletionRequestMessage::User(
                        ChatCompletionRequestUserMessage {
                            content: msg.content.clone().into(),
                            name: None,
                        },
                    ));
                }
                super::types::MessageRole::Assistant => {
                    // Assistant 消息暂时不支持，跳过或可以扩展
                }
            }
        }

        // 5 构建请求
        let request = CreateChatCompletionRequest {
            model: model.model_id.clone(),
            messages,
            temperature: Some(0.7),
            max_tokens: Some(4096),
            ..Default::default()
        };

        // 6 调用 API
        let response = client
            .chat()
            .create(request)
            .await
            .map_err(|e| format!("AI 调用失败: {}", e))?;

        // 7 提取响应内容
        let content = response
            .choices
            .first()
            .and_then(|choice| choice.message.content.clone())
            .unwrap_or_default();

        // 8 获取 token 使用统计
        let (input_tokens, output_tokens, total_tokens) = if let Some(usage) = &response.usage {
            (
                usage.prompt_tokens as i32,
                usage.completion_tokens as i32,
                usage.total_tokens as i32,
            )
        } else {
            (0, 0, 0)
        };

        // 9 计算耗时
        let duration_ms = start_time.elapsed().as_millis() as i64;

        // 10 提取最后一条用户消息
        let last_user_message = request_data
            .messages
            .iter()
            .rev()
            .find(|msg| msg.role == MessageRole::User)
            .map(|msg| msg.content.clone())
            .unwrap_or_default();

        // 11 保存调用记录
        let call_log = CreateAiCallLog {
            provider_id: model.provider_id,
            model_id: model.id,
            model_name: model.model_id.clone(),
            provider_name: provider.name.clone(),
            input_tokens,
            output_tokens,
            total_tokens,
            duration_ms,
            message: last_user_message,
            response: Some(content.clone()),
            status: "success".to_string(),
            error_message: None,
            call_time,
        };

        // 异步保存日志（不阻塞返回）
        let log_repo = self.call_log_repo.clone();
        tokio::spawn(async move {
            if let Err(e) = log_repo.write().await.create(call_log).await {
                eprintln!("保存 AI 调用记录失败: {}", e);
            }
        });

        // 将 response 转换为 JSON Value
        let response_json =
            serde_json::to_value(&response).map_err(|e| format!("序列化响应失败: {}", e))?;

        Ok(AiChatResponse {
            content,
            response: response_json,
        })
    }
}
