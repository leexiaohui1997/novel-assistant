-- 创建小说名词表
CREATE TABLE IF NOT EXISTS novel_terms (
    id UUID PRIMARY KEY,
    novel_id UUID NOT NULL,
    term_type TEXT NOT NULL,
    name TEXT NOT NULL,
    description TEXT,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (novel_id) REFERENCES novels(id) ON DELETE CASCADE
);

-- 创建索引以提高按小说查询性能
CREATE INDEX IF NOT EXISTS idx_novel_terms_novel_id ON novel_terms(novel_id);

-- 创建复合索引以优化按小说 + 类型组合过滤
CREATE INDEX IF NOT EXISTS idx_novel_terms_novel_id_term_type ON novel_terms(novel_id, term_type);
