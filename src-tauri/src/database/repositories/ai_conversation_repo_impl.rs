use async_trait::async_trait;
use sqlx::{Pool, Sqlite};
use uuid::Uuid;

use crate::database::{
    error::DbError,
    models::ai_conversation::{AiConversation, CreateAiConversation},
    repositories::AiConversationRepository,
};

/// SQLite AI 会话 Repository 实现
pub struct SqliteAiConversationRepository {
    pool: Pool<Sqlite>,
}

impl SqliteAiConversationRepository {
    /// 构造函数
    ///
    /// # 参数
    /// - `pool`: SQLite 连接池
    pub fn new(pool: Pool<Sqlite>) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl AiConversationRepository for SqliteAiConversationRepository {
    async fn create(&self, data: CreateAiConversation) -> Result<AiConversation, DbError> {
        let id = Uuid::new_v4();

        sqlx::query_as::<_, AiConversation>(
            r#"
            INSERT INTO ai_conversations (
                id, title, conversation_type, conversation_params, is_pinned, remark
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6)
            RETURNING *
            "#,
        )
        .bind(id)
        .bind(data.title.as_deref())
        .bind(&data.conversation_type)
        .bind(data.conversation_params.as_deref())
        .bind(data.is_pinned)
        .bind(data.remark.as_deref())
        .fetch_one(&self.pool)
        .await
        .map_err(Into::into)
    }
}
