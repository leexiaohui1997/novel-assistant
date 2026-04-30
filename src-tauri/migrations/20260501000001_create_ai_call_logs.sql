-- 创建 AI 调用记录表
-- 记录每次 AI 调用的详细信息，用于成本统计、性能分析和审计
CREATE TABLE IF NOT EXISTS ai_call_logs (
    id UUID PRIMARY KEY,
    
    -- 供应商和模型信息
    provider_id UUID NOT NULL,
    model_id UUID NOT NULL,
    model_name TEXT NOT NULL,
    provider_name TEXT NOT NULL,
    
    -- Token 统计
    input_tokens INTEGER NOT NULL DEFAULT 0,
    output_tokens INTEGER NOT NULL DEFAULT 0,
    total_tokens INTEGER NOT NULL DEFAULT 0,
    
    -- 性能指标
    duration_ms INTEGER NOT NULL DEFAULT 0,
    
    -- 请求和响应内容
    message TEXT NOT NULL,  -- 最后一条用户消息
    response TEXT,
    
    -- 调用状态
    status TEXT NOT NULL DEFAULT 'success',
    error_message TEXT,
    
    -- 时间信息
    call_time DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    
    FOREIGN KEY (provider_id) REFERENCES ai_providers(id) ON DELETE CASCADE,
    FOREIGN KEY (model_id) REFERENCES ai_models(id) ON DELETE CASCADE
);

-- 索引优化：按时间范围查询、按模型统计、按状态筛选
CREATE INDEX IF NOT EXISTS idx_ai_call_logs_call_time ON ai_call_logs(call_time);
CREATE INDEX IF NOT EXISTS idx_ai_call_logs_model_id ON ai_call_logs(model_id);
CREATE INDEX IF NOT EXISTS idx_ai_call_logs_provider_id ON ai_call_logs(provider_id);
CREATE INDEX IF NOT EXISTS idx_ai_call_logs_status ON ai_call_logs(status);
