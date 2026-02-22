use axum::extract::{Path, Query, State};
use axum::Json;
use serde::Deserialize;
use validator::Validate;

use crate::app_state::AppState;
use crate::error::{AppError, AppResult};
use crate::events::outbox::insert_outbox_event;
use crate::middleware::auth::AuthUser;

use super::models::*;
use super::repository;

#[derive(Debug, Deserialize)]
pub struct ReviewQuery {
    pub page: Option<i64>,
    pub per_page: Option<i64>,
}

pub async fn create_review(
    State(state): State<AppState>,
    axum::extract::Extension(auth_user): axum::extract::Extension<AuthUser>,
    Path(novel_id): Path<uuid::Uuid>,
    Json(req): Json<CreateReviewRequest>,
) -> AppResult<Json<Review>> {
    req.validate()
        .map_err(|e| AppError::BadRequest(e.to_string()))?;

    let mut tx = state.db.begin().await?;

    let review = sqlx::query_as::<_, Review>(
        "INSERT INTO reviews (user_id, novel_id, rating, title, body) VALUES ($1, $2, $3, $4, $5) RETURNING *"
    )
    .bind(auth_user.user_id)
    .bind(novel_id)
    .bind(req.rating)
    .bind(req.title.as_deref())
    .bind(&req.body)
    .fetch_one(&mut *tx)
    .await
    .map_err(|e| match e {
        sqlx::Error::Database(ref db_err) if db_err.constraint().is_some() => {
            AppError::Conflict("You have already reviewed this novel".to_string())
        }
        _ => AppError::Database(e),
    })?;

    insert_outbox_event(&mut tx, review.id, "novel.review.created").await?;

    tx.commit().await?;

    Ok(Json(review))
}

pub async fn list_reviews(
    State(state): State<AppState>,
    Path(novel_id): Path<uuid::Uuid>,
    Query(query): Query<ReviewQuery>,
) -> AppResult<Json<Vec<Review>>> {
    let page = query.page.unwrap_or(1).max(1);
    let per_page = query.per_page.unwrap_or(20).min(50);
    let offset = (page - 1) * per_page;

    let reviews = repository::list_by_novel(&state.db, novel_id, per_page, offset).await?;

    Ok(Json(reviews))
}

pub async fn update_review(
    State(state): State<AppState>,
    axum::extract::Extension(auth_user): axum::extract::Extension<AuthUser>,
    Path((novel_id, review_id)): Path<(uuid::Uuid, uuid::Uuid)>,
    Json(req): Json<UpdateReviewRequest>,
) -> AppResult<Json<Review>> {
    req.validate()
        .map_err(|e| AppError::BadRequest(e.to_string()))?;

    let mut tx = state.db.begin().await?;

    let review = sqlx::query_as::<_, Review>(
        "UPDATE reviews SET rating = COALESCE($3, rating), title = COALESCE($4, title), body = COALESCE($5, body) WHERE id = $1 AND user_id = $2 AND novel_id = $6 RETURNING *"
    )
    .bind(review_id)
    .bind(auth_user.user_id)
    .bind(req.rating)
    .bind(req.title.as_deref())
    .bind(req.body.as_deref())
    .bind(novel_id)
    .fetch_optional(&mut *tx)
    .await?
    .ok_or_else(|| AppError::NotFound("Review not found or unauthorized".to_string()))?;

    insert_outbox_event(&mut tx, review.id, "novel.review.updated").await?;

    tx.commit().await?;

    Ok(Json(review))
}

pub async fn delete_review(
    State(state): State<AppState>,
    axum::extract::Extension(auth_user): axum::extract::Extension<AuthUser>,
    Path((_novel_id, review_id)): Path<(uuid::Uuid, uuid::Uuid)>,
) -> AppResult<axum::http::StatusCode> {
    let mut tx = state.db.begin().await?;

    let result = sqlx::query("DELETE FROM reviews WHERE id = $1 AND user_id = $2")
        .bind(review_id)
        .bind(auth_user.user_id)
        .execute(&mut *tx)
        .await?;

    if result.rows_affected() == 0 {
        return Err(AppError::NotFound("Review not found or unauthorized".to_string()));
    }

    insert_outbox_event(&mut tx, review_id, "novel.review.deleted").await?;

    tx.commit().await?;

    Ok(axum::http::StatusCode::NO_CONTENT)
}
