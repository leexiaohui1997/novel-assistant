-- 扩展 character_type 字段的 CHECK 约束，支持新增的敌对角色类型
-- SQLite 不支持直接修改 CHECK 约束，需要重建表

CREATE TABLE characters_new (
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

INSERT INTO characters_new SELECT * FROM characters;
DROP TABLE characters;
ALTER TABLE characters_new RENAME TO characters;

CREATE INDEX IF NOT EXISTS idx_characters_novel_id ON characters(novel_id);
