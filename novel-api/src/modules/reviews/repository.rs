use sqlx::PgPool;
use uuid::Uuid;

use super::models::Review;
use crate::error::{AppError, AppResult};

pub async fn create(
    pool: &PgPool,
    user_id: Uuid,
    novel_id: Uuid,
    rating: i16,
    title: Option<&str>,
    body: &str,
) -> AppResult<Review> {
    let review = sqlx::query_as::<_, Review>(
        "INSERT INTO reviews (user_id, novel_id, rating, title, body) VALUES ($1, $2, $3, $4, $5) RETURNING *"
    )
    .bind(user_id)
    .bind(novel_id)
    .bind(rating)
    .bind(title)
    .bind(body)
    .fetch_one(pool)
    .await
    .map_err(|e| match e {
        sqlx::Error::Database(ref db_err) if db_err.constraint().is_some() => {
            AppError::Conflict("You have already reviewed this novel".to_string())
        }
        _ => AppError::Database(e),
    })?;

    Ok(review)
}

pub async fn list_by_novel(
    pool: &PgPool,
    novel_id: Uuid,
    limit: i64,
    offset: i64,
) -> AppResult<Vec<Review>> {
    let reviews = sqlx::query_as::<_, Review>(
        "SELECT * FROM reviews WHERE novel_id = $1 AND is_visible = TRUE ORDER BY created_at DESC LIMIT $2 OFFSET $3"
    )
    .bind(novel_id)
    .bind(limit)
    .bind(offset)
    .fetch_all(pool)
    .await?;

    Ok(reviews)
}

pub async fn update(
    pool: &PgPool,
    id: Uuid,
    user_id: Uuid,
    rating: Option<i16>,
    title: Option<&str>,
    body: Option<&str>,
) -> AppResult<Review> {
    let review = sqlx::query_as::<_, Review>(
        "UPDATE reviews SET rating = COALESCE($3, rating), title = COALESCE($4, title), body = COALESCE($5, body) WHERE id = $1 AND user_id = $2 RETURNING *"
    )
    .bind(id)
    .bind(user_id)
    .bind(rating)
    .bind(title)
    .bind(body)
    .fetch_optional(pool)
    .await?
    .ok_or_else(|| AppError::NotFound("Review not found or unauthorized".to_string()))?;

    Ok(review)
}

pub async fn delete(pool: &PgPool, id: Uuid, user_id: Uuid) -> AppResult<()> {
    let result = sqlx::query("DELETE FROM reviews WHERE id = $1 AND user_id = $2")
        .bind(id)
        .bind(user_id)
        .execute(pool)
        .await?;

    if result.rows_affected() == 0 {
        return Err(AppError::NotFound("Review not found or unauthorized".to_string()));
    }
    Ok(())
}
