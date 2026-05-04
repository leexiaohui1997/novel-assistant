-- 为章节大纲表添加 plot 字段
-- 迁移版本：20260504000002

ALTER TABLE chapter_outlines ADD COLUMN plot TEXT;
