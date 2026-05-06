-- 为 AI 模型表添加 support_thinking 字段
-- 迁移版本：20260507000001

ALTER TABLE ai_models ADD COLUMN support_thinking BOOLEAN NOT NULL DEFAULT FALSE;
