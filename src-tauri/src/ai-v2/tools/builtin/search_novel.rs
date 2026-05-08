use std::sync::Arc;
use tokio::sync::RwLock;

use serde_json::json;

use crate::database::repositories::{NovelRepository, QueryOptions};
use crate::utils::formatters::{format_channel, format_tags};
use crate::utils::pagination::PaginationParams;

use super::super::traits::AiTool;

/// 小说查询工具
pub struct SearchNovelTool {
    novel_repo: Arc<RwLock<Box<dyn NovelRepository + Send + Sync>>>,
}

impl SearchNovelTool {
    pub fn new(novel_repo: Arc<RwLock<Box<dyn NovelRepository + Send + Sync>>>) -> Self {
        Self { novel_repo }
    }
}

impl AiTool for SearchNovelTool {
    fn name(&self) -> &str {
        "search_novel"
    }

    fn description(&self) -> &str {
        "根据关键词搜索小说，支持分页查询。返回小说ID、书名、频道、主分类标签和简介。"
    }

    fn parameters_schema(&self) -> serde_json::Value {
        json!({
            "type": "object",
            "properties": {
                "keyword": {
                    "type": "string",
                    "description": "搜索关键词（可选）"
                },
                "page": {
                    "type": "integer",
                    "description": "页码，默认为1",
                    "default": 1
                },
                "limit": {
                    "type": "integer",
                    "description": "每页数量，默认为10，0表示全部",
                    "default": 10
                }
            }
        })
    }

    fn execute(
        &self,
        args: serde_json::Value,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<String, String>> + Send>> {
        let novel_repo = self.novel_repo.clone();
        Box::pin(async move {
            let keyword = args
                .get("keyword")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string());
            let page = args.get("page").and_then(|v| v.as_i64()).unwrap_or(1) as i64;
            let limit = args.get("limit").and_then(|v| v.as_i64()).unwrap_or(10) as i64;

            let params = PaginationParams {
                page,
                page_size: limit,
            };

            let options = QueryOptions {
                with_tags: true,
                with_stats: false,
            };

            let repo = novel_repo.read().await;
            let result = repo
                .search_with_pagination(keyword.as_ref().map(|s| s.as_str()), &params, &options)
                .await
                .map_err(|e| format!("查询小说失败: {}", e))?;

            // 格式化返回数据
            let formatted_data: Vec<serde_json::Value> = result
                .data
                .iter()
                .map(|nwt| {
                    json!({
                        "id": nwt.novel.id,
                        "title": nwt.novel.title,
                        "channel": format_channel(&nwt.novel.target_reader),
                        "tags": format_tags(&nwt.tags),
                        "description": nwt.novel.description
                    })
                })
                .collect();

            serde_json::to_string(&formatted_data).map_err(|e| format!("序列化结果失败: {}", e))
        })
    }
}
