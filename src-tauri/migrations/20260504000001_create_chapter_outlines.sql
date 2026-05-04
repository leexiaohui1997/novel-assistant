-- 创建章节大纲表
-- 迁移版本：20260504000001

CREATE TABLE IF NOT EXISTS chapter_outlines (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    novel_id UUID NOT NULL,
    chapter_id UUID UNIQUE,
    positioning TEXT NOT NULL,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (novel_id) REFERENCES novels(id) ON DELETE CASCADE,
    FOREIGN KEY (chapter_id) REFERENCES chapters(id) ON DELETE CASCADE
);

-- 创建索引以提高查询性能
CREATE INDEX IF NOT EXISTS idx_chapter_outlines_novel_id ON chapter_outlines(novel_id);
CREATE INDEX IF NOT EXISTS idx_chapter_outlines_chapter_id ON chapter_outlines(chapter_id);
