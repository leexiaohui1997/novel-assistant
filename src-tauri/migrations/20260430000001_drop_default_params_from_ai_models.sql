-- 从 AI 模型表中移除默认参数字段（default_temperature、default_max_tokens）
-- 这些参数将由应用层或独立的配置表管理，不再存于模型表

ALTER TABLE ai_models DROP COLUMN default_temperature;
ALTER TABLE ai_models DROP COLUMN default_max_tokens;
