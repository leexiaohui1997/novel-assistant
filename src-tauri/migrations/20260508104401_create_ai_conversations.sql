-- 创建 AI 会话表
-- 迁移版本：20260508104401

CREATE TABLE IF NOT EXISTS ai_conversations (
    id UUID PRIMARY KEY,
    title TEXT,
    conversation_type TEXT NOT NULL,
    conversation_params TEXT,
    is_pinned BOOLEAN NOT NULL DEFAULT 0,
    remark TEXT,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    last_message_at DATETIME
);

-- 创建索引以提高查询性能
CREATE INDEX IF NOT EXISTS idx_ai_conversations_type ON ai_conversations(conversation_type);
CREATE INDEX IF NOT EXISTS idx_ai_conversations_pinned ON ai_conversations(is_pinned);
CREATE INDEX IF NOT EXISTS idx_ai_conversations_last_message_at ON ai_conversations(last_message_at DESC);
