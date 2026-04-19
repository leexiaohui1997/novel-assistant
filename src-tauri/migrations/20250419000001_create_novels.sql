-- 创建小说表
CREATE TABLE IF NOT EXISTS novels (
    id UUID PRIMARY KEY,
    title TEXT NOT NULL,
    target_reader TEXT NOT NULL CHECK(target_reader IN ('male', 'female')),
    description TEXT NOT NULL,
    cover_image TEXT,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- 创建标签字典表
CREATE TABLE IF NOT EXISTS tags (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL UNIQUE,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- 创建小说-标签关联表
CREATE TABLE IF NOT EXISTS novel_tags (
    novel_id UUID NOT NULL,
    tag_id INTEGER NOT NULL,
    PRIMARY KEY (novel_id, tag_id),
    FOREIGN KEY (novel_id) REFERENCES novels(id) ON DELETE CASCADE,
    FOREIGN KEY (tag_id) REFERENCES tags(id) ON DELETE CASCADE
);

-- 创建索引以优化查询性能
CREATE INDEX IF NOT EXISTS idx_novels_created_at ON novels(created_at DESC);
CREATE INDEX IF NOT EXISTS idx_novel_tags_tag_id ON novel_tags(tag_id);
