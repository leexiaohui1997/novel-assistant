-- 创建创作状态表
-- 当前阶段仅存储 novel_id 作为标识，后续可根据需要添加具体状态字段
CREATE TABLE IF NOT EXISTS creation_states (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    novel_id UUID NOT NULL UNIQUE,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (novel_id) REFERENCES novels(id) ON DELETE CASCADE
);

-- 创建索引优化查询
CREATE INDEX IF NOT EXISTS idx_creation_states_novel_id ON creation_states(novel_id);
