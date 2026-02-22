use sqlx::PgPool;
use uuid::Uuid;

use super::models::Comment;
use crate::error::{AppError, AppResult};

pub async fn create(
    pool: &PgPool,
    user_id: Uuid,
    entity_type: &str,
    entity_id: Uuid,
    parent_id: Option<Uuid>,
    body: &str,
) -> AppResult<Comment> {
    let depth: i16 = if let Some(pid) = parent_id {
        let parent = sqlx::query_scalar::<_, i16>("SELECT depth FROM comments WHERE id = $1")
            .bind(pid)
            .fetch_optional(pool)
            .await?
            .ok_or_else(|| AppError::NotFound("Parent comment not found".to_string()))?;

        if parent >= 5 {
            return Err(AppError::BadRequest("Maximum comment depth reached".to_string()));
        }
        parent + 1
    } else {
        0
    };

    let comment = sqlx::query_as::<_, Comment>(
        "INSERT INTO comments (user_id, entity_type, entity_id, parent_id, depth, body) VALUES ($1, $2::content_type, $3, $4, $5, $6) RETURNING *"
    )
    .bind(user_id)
    .bind(entity_type)
    .bind(entity_id)
    .bind(parent_id)
    .bind(depth)
    .bind(body)
    .fetch_one(pool)
    .await?;

    Ok(comment)
}

pub async fn list_by_entity(
    pool: &PgPool,
    entity_type: &str,
    entity_id: Uuid,
    limit: i64,
    offset: i64,
) -> AppResult<Vec<Comment>> {
    let comments = sqlx::query_as::<_, Comment>(
        "SELECT * FROM comments WHERE entity_type = $1::content_type AND entity_id = $2 AND is_visible = TRUE ORDER BY created_at ASC LIMIT $3 OFFSET $4"
    )
    .bind(entity_type)
    .bind(entity_id)
    .bind(limit)
    .bind(offset)
    .fetch_all(pool)
    .await?;

    Ok(comments)
}

pub async fn delete(pool: &PgPool, id: Uuid, user_id: Uuid) -> AppResult<()> {
    let result = sqlx::query("DELETE FROM comments WHERE id = $1 AND user_id = $2")
        .bind(id)
        .bind(user_id)
        .execute(pool)
        .await?;

    if result.rows_affected() == 0 {
        return Err(AppError::NotFound("Comment not found or unauthorized".to_string()));
    }
    Ok(())
}
