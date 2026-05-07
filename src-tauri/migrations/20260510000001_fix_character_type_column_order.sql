-- 修复 character_type 字段数据错位问题
-- 原因：之前的迁移中，新表改变了列顺序，但使用了 SELECT * 导致数据错位
-- 解决方案：显式指定列名进行数据映射

CREATE TABLE characters_fixed (
    id UUID PRIMARY KEY,
    novel_id UUID NOT NULL,
    name TEXT NOT NULL,
    gender TEXT NOT NULL CHECK(gender IN ('male', 'female', 'other', 'unknown')),
    character_type TEXT,
    background TEXT,
    appearance TEXT,
    personality TEXT,
    additional_info TEXT,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (novel_id) REFERENCES novels(id) ON DELETE CASCADE
);

-- 显式指定列名，确保数据正确映射
INSERT INTO characters_fixed (
    id, novel_id, name, gender, background, appearance, personality, additional_info, created_at, updated_at, character_type
)
SELECT 
    id, novel_id, name, gender, character_type, background, appearance, personality, additional_info, created_at, updated_at
FROM characters;

DROP TABLE characters;
ALTER TABLE characters_fixed RENAME TO characters;

CREATE INDEX IF NOT EXISTS idx_characters_novel_id ON characters(novel_id);
