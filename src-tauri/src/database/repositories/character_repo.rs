use async_trait::async_trait;
use chrono::Utc;
use sqlx::SqlitePool;
use uuid::Uuid;

use crate::database::error::DbError;
use crate::database::models::character::Character;
use crate::utils::pagination::{PaginatedResult, PaginationParams};

/// 角色仓储 trait
#[async_trait]
pub trait CharacterRepository {
    /// 创建角色
    async fn create_character(&self, character: &Character) -> Result<Character, DbError>;

    /// 根据 ID 查询角色
    async fn find_by_id(&self, id: &Uuid) -> Result<Option<Character>, DbError>;

    /// 根据小说 ID 查询所有角色
    async fn find_by_novel_id(&self, novel_id: &Uuid) -> Result<Vec<Character>, DbError>;

    /// 分页查询角色（支持按小说 ID 筛选）
    async fn find_with_pagination(
        &self,
        params: &PaginationParams,
        novel_id: Option<&Uuid>,
    ) -> Result<PaginatedResult<Character>, DbError>;

    /// 更新角色
    async fn update_character(&self, character: &Character) -> Result<Character, DbError>;

    /// 删除角色
    async fn delete_character(&self, id: &Uuid) -> Result<(), DbError>;
}

/// SQLite 角色仓储实现
pub struct SqliteCharacterRepository {
    pool: SqlitePool,
}

impl SqliteCharacterRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl CharacterRepository for SqliteCharacterRepository {
    async fn create_character(&self, character: &Character) -> Result<Character, DbError> {
        let now = Utc::now();
        let id = Uuid::new_v4();

        sqlx::query(
            "INSERT INTO characters (id, novel_id, name, gender, background, appearance, personality, additional_info, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
        )
        .bind(&id)
        .bind(&character.novel_id)
        .bind(&character.name)
        .bind(&character.gender)
        .bind(&character.background)
        .bind(&character.appearance)
        .bind(&character.personality)
        .bind(&character.additional_info)
        .bind(now)
        .bind(now)
        .execute(&self.pool)
        .await?;

        // 返回创建的角色
        self.find_by_id(&id).await?.ok_or(DbError::Business(format!(
            "Character with id {} not found after creation",
            id
        )))
    }

    async fn find_by_id(&self, id: &Uuid) -> Result<Option<Character>, DbError> {
        let character = sqlx::query_as::<_, Character>("SELECT * FROM characters WHERE id = ?1")
            .bind(id)
            .fetch_optional(&self.pool)
            .await?;

        Ok(character)
    }

    async fn find_by_novel_id(&self, novel_id: &Uuid) -> Result<Vec<Character>, DbError> {
        let characters = sqlx::query_as::<_, Character>(
            "SELECT * FROM characters WHERE novel_id = ?1 ORDER BY created_at DESC",
        )
        .bind(novel_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(characters)
    }

    async fn find_with_pagination(
        &self,
        params: &PaginationParams,
        novel_id: Option<&Uuid>,
    ) -> Result<PaginatedResult<Character>, DbError> {
        let (count_sql, data_sql) = match novel_id {
            Some(_) => (
                "SELECT COUNT(*) FROM characters WHERE novel_id = ?1",
                "SELECT * FROM characters WHERE novel_id = ?1 ORDER BY created_at ASC LIMIT ?2 OFFSET ?3",
            ),
            None => (
                "SELECT COUNT(*) FROM characters",
                "SELECT * FROM characters ORDER BY created_at ASC LIMIT ?1 OFFSET ?2",
            ),
        };

        let offset = (params.page - 1) * params.page_size;

        // 查询总数
        let total: (i64,) = if let Some(nid) = novel_id {
            sqlx::query_as(count_sql)
                .bind(nid)
                .fetch_one(&self.pool)
                .await?
        } else {
            sqlx::query_as(count_sql).fetch_one(&self.pool).await?
        };

        // 查询分页数据
        let data: Vec<Character> = if let Some(nid) = novel_id {
            sqlx::query_as::<_, Character>(data_sql)
                .bind(nid)
                .bind(params.page_size)
                .bind(offset)
                .fetch_all(&self.pool)
                .await?
        } else {
            sqlx::query_as::<_, Character>(data_sql)
                .bind(params.page_size)
                .bind(offset)
                .fetch_all(&self.pool)
                .await?
        };

        Ok(PaginatedResult {
            data,
            total: total.0,
        })
    }

    async fn update_character(&self, character: &Character) -> Result<Character, DbError> {
        let now = Utc::now();

        sqlx::query(
            "UPDATE characters 
             SET name = ?2, gender = ?3, background = ?4, appearance = ?5, personality = ?6, additional_info = ?7, updated_at = ?8
             WHERE id = ?1",
        )
        .bind(&character.id)
        .bind(&character.name)
        .bind(&character.gender)
        .bind(&character.background)
        .bind(&character.appearance)
        .bind(&character.personality)
        .bind(&character.additional_info)
        .bind(now)
        .execute(&self.pool)
        .await?;

        // 返回更新后的角色
        self.find_by_id(&character.id)
            .await?
            .ok_or(DbError::Business(format!(
                "Character with id {} not found after update",
                character.id
            )))
    }

    async fn delete_character(&self, id: &Uuid) -> Result<(), DbError> {
        let result = sqlx::query("DELETE FROM characters WHERE id = ?1")
            .bind(id)
            .execute(&self.pool)
            .await?;

        if result.rows_affected() == 0 {
            return Err(DbError::Business(format!(
                "Character with id {} not found",
                id
            )));
        }

        Ok(())
    }
}
