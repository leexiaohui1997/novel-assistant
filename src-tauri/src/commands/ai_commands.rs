use tauri::State;
use uuid::Uuid;

use crate::ai::service::AiService;
use crate::ai::types::{AiRequestData, Message, MessageRole};
use crate::AppState;

/// 测试模型是否可用
///
/// 发送一个简单的"你好"消息到指定模型，验证模型配置和连接是否正常
#[tauri::command]
pub async fn test_model(
    state: State<'_, AppState>,
    model_id: String,
) -> Result<serde_json::Value, String> {
    let model_uuid = Uuid::parse_str(&model_id).map_err(|e| e.to_string())?;

    // 构建测试请求
    let request_data = AiRequestData {
        model_id: Some(model_uuid),
        messages: vec![Message {
            role: MessageRole::User,
            content: "你好".to_string(),
        }],
    };

    // 创建 AI 服务实例
    let ai_service = AiService::new(
        state.model_repo.clone(),
        state.provider_repo.clone(),
        state.call_log_repo.clone(),
    );

    // 执行测试
    let response = ai_service.chat(request_data).await?;

    // 返回测试结果
    Ok(serde_json::json!({
        "success": true,
        "content": response.content,
        "usage": response.response.get("usage")
    }))
}
