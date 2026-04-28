-- 创建 AI 服务供应商表
-- 存储不同 AI 服务商的基础配置（API 地址、密钥等），一个供应商下可挂载多个模型
CREATE TABLE IF NOT EXISTS ai_providers (
    id UUID PRIMARY KEY,
    name TEXT NOT NULL,
    base_url TEXT NOT NULL,
    api_key TEXT,
    is_enabled BOOLEAN NOT NULL DEFAULT 1,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- 创建 AI 模型表
-- 存储具体模型配置（模型标识、默认参数等），每个模型归属于一个供应商
CREATE TABLE IF NOT EXISTS ai_models (
    id UUID PRIMARY KEY,
    provider_id UUID NOT NULL,
    model_id TEXT NOT NULL,
    alias TEXT NOT NULL,
    default_temperature REAL NOT NULL DEFAULT 0.7,
    default_max_tokens INTEGER NOT NULL DEFAULT 4096,
    is_default BOOLEAN NOT NULL DEFAULT 0,
    is_enabled BOOLEAN NOT NULL DEFAULT 1,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (provider_id) REFERENCES ai_providers(id) ON DELETE CASCADE
);

-- 索引优化：按供应商查询模型、快速定位默认模型
CREATE INDEX IF NOT EXISTS idx_ai_models_provider_id ON ai_models(provider_id);
CREATE INDEX IF NOT EXISTS idx_ai_models_is_default ON ai_models(is_default);
