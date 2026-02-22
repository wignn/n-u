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
pub struct CommentQuery {
    pub entity_type: String,
    pub entity_id: uuid::Uuid,
    pub page: Option<i64>,
    pub per_page: Option<i64>,
}

pub async fn create_comment(
    State(state): State<AppState>,
    axum::extract::Extension(auth_user): axum::extract::Extension<AuthUser>,
    Json(req): Json<CreateCommentRequest>,
) -> AppResult<Json<Comment>> {
    req.validate()
        .map_err(|e| AppError::BadRequest(e.to_string()))?;

    let mut tx = state.db.begin().await?;

    let comment = sqlx::query_as::<_, Comment>(
        "INSERT INTO comments (user_id, entity_type, entity_id, parent_id, depth, body) VALUES ($1, $2::content_type, $3, $4, $5, $6) RETURNING *"
    )
    .bind(auth_user.user_id)
    .bind(&req.entity_type)
    .bind(req.entity_id)
    .bind(req.parent_id)
    .bind(0i16)
    .bind(&req.body)
    .fetch_one(&mut *tx)
    .await?;

    insert_outbox_event(&mut tx, comment.id, "novel.comment.created").await?;

    tx.commit().await?;

    Ok(Json(comment))
}

pub async fn list_comments(
    State(state): State<AppState>,
    Query(query): Query<CommentQuery>,
) -> AppResult<Json<Vec<Comment>>> {
    let page = query.page.unwrap_or(1).max(1);
    let per_page = query.per_page.unwrap_or(50).min(100);
    let offset = (page - 1) * per_page;

    let comments = repository::list_by_entity(
        &state.db,
        &query.entity_type,
        query.entity_id,
        per_page,
        offset,
    )
    .await?;

    Ok(Json(comments))
}

pub async fn delete_comment(
    State(state): State<AppState>,
    axum::extract::Extension(auth_user): axum::extract::Extension<AuthUser>,
    Path(id): Path<uuid::Uuid>,
) -> AppResult<axum::http::StatusCode> {
    let mut tx = state.db.begin().await?;

    let result = sqlx::query("DELETE FROM comments WHERE id = $1 AND user_id = $2")
        .bind(id)
        .bind(auth_user.user_id)
        .execute(&mut *tx)
        .await?;

    if result.rows_affected() == 0 {
        return Err(AppError::NotFound("Comment not found or unauthorized".to_string()));
    }

    insert_outbox_event(&mut tx, id, "novel.comment.deleted").await?;

    tx.commit().await?;

    Ok(axum::http::StatusCode::NO_CONTENT)
}
