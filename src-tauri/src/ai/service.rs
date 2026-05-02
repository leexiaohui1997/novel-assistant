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

        // 3 构建 HTTP 请求（使用 reqwest 直接调用，避免 async-openai 的反序列化问题）
        use serde_json::json;

        // 构建消息列表
        let mut messages_json: Vec<serde_json::Value> = vec![];
        for msg in &request_data.messages {
            let role_str = match msg.role {
                MessageRole::System => "system",
                MessageRole::User => "user",
                MessageRole::Assistant => "assistant",
            };

            messages_json.push(json!({
                "role": role_str,
                "content": msg.content
            }));
        }

        // 构建请求体
        let request_body = json!({
            "model": model.model_id,
            "messages": messages_json,
            "temperature": 0.7,
            "max_tokens": 4096
        });

        // 发送 HTTP 请求
        let http_client = reqwest::Client::new();
        let response = http_client
            .post(format!("{}/chat/completions", provider.base_url))
            .header("Authorization", format!("Bearer {}", api_key))
            .header("Content-Type", "application/json")
            .json(&request_body)
            .send()
            .await
            .map_err(|e| format!("HTTP 请求失败: {}", e))?;

        // 检查 HTTP 状态
        let status = response.status();
        if !status.is_success() {
            // 克隆状态码用于错误消息，因为 text() 会消耗 response
            let status_code = status.as_u16();
            let error_text = response.text().await.unwrap_or_default();
            return Err(format!("API 返回错误 ({}): {}", status_code, error_text));
        }

        // 解析响应（使用宽松的解析方式）
        let response_json: serde_json::Value = response
            .json()
            .await
            .map_err(|e| format!("解析响应失败: {}", e))?;

        // 4 提取响应内容（容忍 role 字段为空或不规范）
        let content = response_json
            .get("choices")
            .and_then(|choices| choices.as_array())
            .and_then(|choices| choices.first())
            .and_then(|choice| choice.get("message"))
            .and_then(|message| message.get("content"))
            .and_then(|content| content.as_str())
            .unwrap_or("")
            .to_string();

        // 5 获取 token 使用统计
        let (input_tokens, output_tokens, total_tokens) = response_json
            .get("usage")
            .map(|usage| {
                (
                    usage
                        .get("prompt_tokens")
                        .and_then(|v| v.as_i64())
                        .unwrap_or(0) as i32,
                    usage
                        .get("completion_tokens")
                        .and_then(|v| v.as_i64())
                        .unwrap_or(0) as i32,
                    usage
                        .get("total_tokens")
                        .and_then(|v| v.as_i64())
                        .unwrap_or(0) as i32,
                )
            })
            .unwrap_or((0, 0, 0));

        // 6 计算耗时
        let duration_ms = start_time.elapsed().as_millis() as i64;

        // 7 提取最后一条用户消息
        let last_user_message = request_data
            .messages
            .iter()
            .rev()
            .find(|msg| msg.role == MessageRole::User)
            .map(|msg| msg.content.clone())
            .unwrap_or_default();

        // 8 保存调用记录
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

        Ok(AiChatResponse {
            content,
            response: response_json,
        })
    }
}
