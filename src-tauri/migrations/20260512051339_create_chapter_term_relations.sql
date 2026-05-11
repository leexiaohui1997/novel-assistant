-- 创建小说名词-章节关联表
CREATE TABLE IF NOT EXISTS chapter_term_relations (
    id UUID PRIMARY KEY,
    chapter_id UUID NOT NULL,
    term_id UUID NOT NULL,
    description TEXT,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (chapter_id) REFERENCES chapters(id) ON DELETE CASCADE,
    FOREIGN KEY (term_id) REFERENCES novel_terms(id) ON DELETE CASCADE
);

-- 创建索引以提高按章节查询性能
CREATE INDEX IF NOT EXISTS idx_chapter_term_relations_chapter_id ON chapter_term_relations(chapter_id);

-- 创建索引以提高按名词查询性能
CREATE INDEX IF NOT EXISTS idx_chapter_term_relations_term_id ON chapter_term_relations(term_id);

-- 创建复合索引以优化按章节 + 名词组合查询
CREATE INDEX IF NOT EXISTS idx_chapter_term_relations_chapter_term ON chapter_term_relations(chapter_id, term_id);

-- 创建索引以支持按创建时间排序
CREATE INDEX IF NOT EXISTS idx_chapter_term_relations_created_at ON chapter_term_relations(created_at);

-- 创建索引以支持按更新时间排序
CREATE INDEX IF NOT EXISTS idx_chapter_term_relations_updated_at ON chapter_term_relations(updated_at);
