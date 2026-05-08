-- 为 ai_conversations 表新增会话状态与会话提示字段
-- 迁移版本：20260508131801

-- 会话状态：TEXT，NOT NULL，默认 'Init'
-- 可选值对应 Rust 后端枚举 `crate::ai_v2::ConversationStatus`
-- （Init / Ready / Computing / Completed / Terminated / Error），不做 CHECK 约束
ALTER TABLE ai_conversations ADD COLUMN status TEXT NOT NULL DEFAULT 'Init';

-- 会话提示：TEXT，允许为 NULL，无默认值
-- 用于保存会话级别的非结构化提示文本，与结构化的 conversation_params 区分
ALTER TABLE ai_conversations ADD COLUMN prompt TEXT;

-- 针对 status 建立索引，便于按状态过滤列表
CREATE INDEX IF NOT EXISTS idx_ai_conversations_status ON ai_conversations(status);
