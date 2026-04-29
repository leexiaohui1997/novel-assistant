-- 为 ai_providers 表添加 model_fetch_type 列
-- 用于标识不同供应商的模型拉取策略（如 "default"、"silicon_flow" 等）
ALTER TABLE ai_providers ADD COLUMN model_fetch_type TEXT NOT NULL DEFAULT 'default';
