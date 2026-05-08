use async_trait::async_trait;
use sqlx::{Pool, Sqlite, Transaction};
use uuid::Uuid;

use crate::database::{
    error::DbError,
    models::ai_conversation_message::{AiConversationMessage, CreateAiConversationMessage},
    repositories::AiConversationMessageRepository,
};

/// SQLite AI 会话消息 Repository 实现
pub struct SqliteAiConversationMessageRepository {
    pool: Pool<Sqlite>,
}

impl SqliteAiConversationMessageRepository {
    /// 构造函数
    ///
    /// # 参数
    /// - `pool`: SQLite 连接池
    pub fn new(pool: Pool<Sqlite>) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl AiConversationMessageRepository for SqliteAiConversationMessageRepository {
    async fn create(
        &self,
        payload: CreateAiConversationMessage,
    ) -> Result<AiConversationMessage, DbError> {
        sqlx::query_as::<_, AiConversationMessage>(
            r#"
            INSERT INTO ai_conversation_messages (
                id,
                conversation_id,
                message_type,
                content,
                thinking_content,
                input_tokens,
                output_tokens,
                sequence,
                model_id,
                provider_name,
                model_name,
                ext_1,
                ext_2,
                ext_3
            ) VALUES (
                ?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14
            )
            RETURNING *
            "#,
        )
        .bind(payload.id)
        .bind(payload.conversation_id)
        .bind(&payload.message_type)
        .bind(&payload.content)
        .bind(payload.thinking_content.as_deref())
        .bind(payload.input_tokens)
        .bind(payload.output_tokens)
        .bind(payload.sequence)
        .bind(payload.model_id)
        .bind(payload.provider_name.as_deref())
        .bind(payload.model_name.as_deref())
        .bind(payload.ext_1.as_deref())
        .bind(payload.ext_2.as_deref())
        .bind(payload.ext_3.as_deref())
        .fetch_one(&self.pool)
        .await
        .map_err(Into::into)
    }

    async fn count_by_conversation(&self, conversation_id: Uuid) -> Result<i64, DbError> {
        let (count,): (i64,) = sqlx::query_as(
            r#"
            SELECT COUNT(*) FROM ai_conversation_messages
            WHERE conversation_id = ?1
            "#,
        )
        .bind(conversation_id)
        .fetch_one(&self.pool)
        .await?;

        Ok(count)
    }

    async fn list_by_conversation(
        &self,
        conversation_id: Uuid,
    ) -> Result<Vec<AiConversationMessage>, DbError> {
        sqlx::query_as::<_, AiConversationMessage>(
            r#"
            SELECT * FROM ai_conversation_messages
            WHERE conversation_id = ?1
            ORDER BY sequence ASC
            "#,
        )
        .bind(conversation_id)
        .fetch_all(&self.pool)
        .await
        .map_err(Into::into)
    }

    async fn finalize_assistant_turn(
        &self,
        payload: CreateAiConversationMessage,
        conversation_status: &str,
    ) -> Result<AiConversationMessage, DbError> {
        let mut tx = self.pool.begin().await?;

        let conversation_id = payload.conversation_id;
        let inserted = insert_message_tx(&mut tx, payload).await?;
        update_conversation_after_insert_tx(
            &mut tx,
            conversation_id,
            inserted.created_at,
            conversation_status,
        )
        .await?;

        tx.commit().await?;
        Ok(inserted)
    }
}

/// 在给定事务内插入一条会话消息并返回完整实体
///
/// 抽取为独立函数以保持 `finalize_assistant_turn` 主流程的圈复杂度 < 5。
async fn insert_message_tx(
    tx: &mut Transaction<'_, Sqlite>,
    payload: CreateAiConversationMessage,
) -> Result<AiConversationMessage, DbError> {
    sqlx::query_as::<_, AiConversationMessage>(
        r#"
        INSERT INTO ai_conversation_messages (
            id,
            conversation_id,
            message_type,
            content,
            thinking_content,
            input_tokens,
            output_tokens,
            sequence,
            model_id,
            provider_name,
            model_name,
            ext_1,
            ext_2,
            ext_3
        ) VALUES (
            ?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14
        )
        RETURNING *
        "#,
    )
    .bind(payload.id)
    .bind(payload.conversation_id)
    .bind(&payload.message_type)
    .bind(&payload.content)
    .bind(payload.thinking_content.as_deref())
    .bind(payload.input_tokens)
    .bind(payload.output_tokens)
    .bind(payload.sequence)
    .bind(payload.model_id)
    .bind(payload.provider_name.as_deref())
    .bind(payload.model_name.as_deref())
    .bind(payload.ext_1.as_deref())
    .bind(payload.ext_2.as_deref())
    .bind(payload.ext_3.as_deref())
    .fetch_one(&mut **tx)
    .await
    .map_err(Into::into)
}

/// 在给定事务内同步更新会话的 `last_message_at` 与 `status`
///
/// 抽取为独立函数以保持 `finalize_assistant_turn` 主流程的圈复杂度 < 5。
async fn update_conversation_after_insert_tx(
    tx: &mut Transaction<'_, Sqlite>,
    conversation_id: Uuid,
    last_message_at: chrono::DateTime<chrono::Utc>,
    status: &str,
) -> Result<(), DbError> {
    sqlx::query(
        r#"
        UPDATE ai_conversations
        SET last_message_at = ?1, status = ?2
        WHERE id = ?3
        "#,
    )
    .bind(last_message_at)
    .bind(status)
    .bind(conversation_id)
    .execute(&mut **tx)
    .await?;

    Ok(())
}
