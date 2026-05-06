-- 给 AI 调用记录表添加思考内容字段
-- 用于记录支持思考能力的模型（如 DeepSeek R1、Qwen3 等）返回的 reasoning_content
ALTER TABLE ai_call_logs ADD COLUMN thinking_content TEXT;
