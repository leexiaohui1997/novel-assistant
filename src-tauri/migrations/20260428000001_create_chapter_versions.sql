-- 创建章节历史版本表
-- 用于存储章节每次更新前的标题 / 正文快照，支持回溯与"应用历史版本"能力
CREATE TABLE IF NOT EXISTS chapter_versions (
    id UUID PRIMARY KEY,
    chapter_id UUID NOT NULL,
    title TEXT NOT NULL,
    content TEXT NOT NULL,
    word_count INTEGER NOT NULL DEFAULT 0,
    -- 该版本对应文章的更新时间（即当时 chapters.updated_at）
    saved_at DATETIME NOT NULL,
    -- 快照记录的插入时间
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (chapter_id) REFERENCES chapters(id) ON DELETE CASCADE
);

-- 索引优化：按章节查询、按章节+时间倒序查询
CREATE INDEX IF NOT EXISTS idx_chapter_versions_chapter_id
    ON chapter_versions(chapter_id);
CREATE INDEX IF NOT EXISTS idx_chapter_versions_chapter_id_saved_at
    ON chapter_versions(chapter_id, saved_at);
