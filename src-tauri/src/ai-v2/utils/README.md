# AI v2 工具函数索引

本目录汇集 AI v2 模块下跨仓储 / 跨服务的推导型工具函数。新增工具请同步登记至本文件。

## find_recommended_model

跨表推导"推荐模型"，供 AI v2 入口在前端未指定 `modelId` 时稳定挑选一个可用模型。
文件：`src-tauri/src/ai-v2/utils/recommended_model.rs`

**签名：**

```rust
pub async fn find_recommended_model(
    ai_call_log_repo: &(dyn AiCallLogRepository + Send + Sync),
    model_repo: &(dyn ModelRepository + Send + Sync),
) -> Result<Model, String>
```

**参数：**

- `ai_call_log_repo` - AI 调用日志仓储（需提供 `find_latest`）
- `model_repo` - 模型仓储（需提供 `find_enabled_by_id` / `find_first_enabled_preferring_default`）

**返回值：**

- `Ok(Model)` - 推荐模型实体
- `Err(String)` - 底层仓储错误或"未找到可用的推荐模型"

**推导规则：**

1. 取 `ai_call_logs` 中最近一条记录（`call_time DESC`），反查其 `model_id` 对应模型；若该模型仍 `is_enabled = true` 则直接返回。
2. 最近记录命中失败时，按 `is_enabled = true` 过滤、`is_default DESC, created_at ASC` 排序取第一条模型。
3. 两步均无命中时返回 `Err("未找到可用的推荐模型")`。

**使用示例：**

```rust
use crate::ai_v2::utils::find_recommended_model;

let call_log_repo = state.call_log_repo.read().await;
let model_repo = state.model_repo.read().await;
let model = find_recommended_model(call_log_repo.as_ref(), model_repo.as_ref()).await?;
```
