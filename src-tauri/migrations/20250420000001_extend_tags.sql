-- 扩展标签表：添加标签类型、目标读者和描述字段
-- 迁移版本：20250420000001

-- 1. 添加标签类型字段
ALTER TABLE tags ADD COLUMN tag_type TEXT NOT NULL DEFAULT 'theme' 
    CHECK(tag_type IN ('main_category', 'theme', 'character', 'plot'));

-- 2. 添加目标读者字段
ALTER TABLE tags ADD COLUMN target_audience TEXT NOT NULL DEFAULT 'both' 
    CHECK(target_audience IN ('male', 'female', 'both'));

-- 3. 添加描述字段
ALTER TABLE tags ADD COLUMN description TEXT;

-- 4. 创建索引以优化筛选查询性能
CREATE INDEX idx_tags_tag_type ON tags(tag_type);
CREATE INDEX idx_tags_target_audience ON tags(target_audience);
CREATE INDEX idx_tags_type_audience ON tags(tag_type, target_audience);
CREATE INDEX idx_tags_name ON tags(name);
