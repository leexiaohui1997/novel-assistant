-- 创建分卷表
CREATE TABLE IF NOT EXISTS volumes (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    novel_id UUID NOT NULL,
    name TEXT NOT NULL,
    sequence INTEGER NOT NULL,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    UNIQUE (novel_id, id),
    FOREIGN KEY (novel_id) REFERENCES novels(id) ON DELETE CASCADE
);

-- 创建章节表
CREATE TABLE IF NOT EXISTS chapters (
    id UUID PRIMARY KEY,
    novel_id UUID NOT NULL,
    title TEXT NOT NULL,
    content TEXT NOT NULL,
    word_count INTEGER NOT NULL DEFAULT 0,
    sequence INTEGER NOT NULL DEFAULT -1,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    UNIQUE (novel_id, id),
    FOREIGN KEY (novel_id) REFERENCES novels(id) ON DELETE CASCADE
);

-- 创建分卷-章节关联表
-- 通过 (novel_id, volume_id) 与 (novel_id, chapter_id) 复合外键，保证分卷与章节属于同一本小说
CREATE TABLE IF NOT EXISTS volume_chapters (
    novel_id UUID NOT NULL,
    volume_id INTEGER NOT NULL,
    chapter_id UUID NOT NULL UNIQUE,
    PRIMARY KEY (volume_id, chapter_id),
    FOREIGN KEY (novel_id, volume_id) REFERENCES volumes(novel_id, id) ON DELETE CASCADE,
    FOREIGN KEY (novel_id, chapter_id) REFERENCES chapters(novel_id, id) ON DELETE CASCADE
);

-- 索引优化
CREATE INDEX IF NOT EXISTS idx_volumes_novel_id_sequence ON volumes(novel_id, sequence);
CREATE INDEX IF NOT EXISTS idx_chapters_novel_id_sequence ON chapters(novel_id, sequence);
CREATE INDEX IF NOT EXISTS idx_volume_chapters_novel_id ON volume_chapters(novel_id);
CREATE INDEX IF NOT EXISTS idx_volume_chapters_volume_id ON volume_chapters(volume_id);