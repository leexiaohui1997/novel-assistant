-- 创建 AI 会话消息表
-- 迁移版本：20260508110201

CREATE TABLE IF NOT EXISTS ai_conversation_messages (
    id UUID PRIMARY KEY,
    conversation_id UUID NOT NULL,
    message_type TEXT NOT NULL,
    content TEXT NOT NULL,
    thinking_content TEXT,
    input_tokens INTEGER NOT NULL DEFAULT 0,
    output_tokens INTEGER NOT NULL DEFAULT 0,
    sequence INTEGER NOT NULL,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    model_id UUID,
    provider_name TEXT,
    model_name TEXT,
    ext_1 TEXT,
    ext_2 TEXT,
    ext_3 TEXT,
    FOREIGN KEY (conversation_id) REFERENCES ai_conversations(id) ON DELETE CASCADE
);

-- 创建索引以提高查询性能
CREATE INDEX IF NOT EXISTS idx_messages_conversation_id ON ai_conversation_messages(conversation_id);
CREATE INDEX IF NOT EXISTS idx_messages_sequence ON ai_conversation_messages(conversation_id, sequence);
CREATE INDEX IF NOT EXISTS idx_messages_created_at ON ai_conversation_messages(created_at);
