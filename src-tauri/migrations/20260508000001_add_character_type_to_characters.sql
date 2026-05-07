-- 为 characters 表新增角色类型字段（允许为空）
ALTER TABLE characters
ADD COLUMN character_type TEXT
    CHECK(character_type IS NULL OR character_type IN ('protagonist', 'second_protagonist', 'third_protagonist', 'supporting', 'minor_supporting'));
