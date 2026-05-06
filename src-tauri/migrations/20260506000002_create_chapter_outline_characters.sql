-- 创建章节大纲-角色关联表
-- 迁移版本：20260506000002

CREATE TABLE IF NOT EXISTS chapter_outline_characters (
    chapter_outline_id INTEGER NOT NULL,
    character_id UUID NOT NULL,
    PRIMARY KEY (chapter_outline_id, character_id),
    FOREIGN KEY (chapter_outline_id) REFERENCES chapter_outlines(id) ON DELETE CASCADE,
    FOREIGN KEY (character_id) REFERENCES characters(id) ON DELETE CASCADE
);

-- 创建索引以提高按角色反查效率
CREATE INDEX IF NOT EXISTS idx_chapter_outline_characters_character_id
    ON chapter_outline_characters(character_id);
