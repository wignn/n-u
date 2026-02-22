use axum::extract::{Path, Query, State};
use axum::Json;
use validator::Validate;

use crate::app_state::AppState;
use crate::error::{AppError, AppResult};
use crate::events::outbox::insert_outbox_event;
use crate::middleware::auth::AuthUser;

use super::models::*;
use super::repository;

pub async fn list_categories(
    State(state): State<AppState>,
) -> AppResult<Json<Vec<ForumCategory>>> {
    let categories = repository::list_categories(&state.db).await?;
    Ok(Json(categories))
}

pub async fn create_thread(
    State(state): State<AppState>,
    axum::extract::Extension(auth_user): axum::extract::Extension<AuthUser>,
    Json(req): Json<CreateThreadRequest>,
) -> AppResult<Json<ForumThread>> {
    req.validate()
        .map_err(|e| AppError::BadRequest(e.to_string()))?;

    let thread_slug = slug::slugify(&req.title);

    let mut tx = state.db.begin().await?;

    let thread = sqlx::query_as::<_, ForumThread>(
        "INSERT INTO forum_threads (category_id, user_id, title, slug, body) VALUES ($1, $2, $3, $4, $5) RETURNING *"
    )
    .bind(req.category_id)
    .bind(auth_user.user_id)
    .bind(&req.title)
    .bind(&thread_slug)
    .bind(&req.body)
    .fetch_one(&mut *tx)
    .await?;

    insert_outbox_event(&mut tx, thread.id, "novel.forum.thread.created").await?;

    tx.commit().await?;

    Ok(Json(thread))
}

pub async fn list_threads(
    State(state): State<AppState>,
    Path(category_id): Path<uuid::Uuid>,
    Query(query): Query<ThreadListQuery>,
) -> AppResult<Json<Vec<ForumThread>>> {
    let page = query.page.unwrap_or(1).max(1);
    let per_page = query.per_page.unwrap_or(20).min(50);
    let offset = (page - 1) * per_page;

    let threads = repository::list_threads_by_category(
        &state.db, category_id, per_page, offset,
    )
    .await?;

    Ok(Json(threads))
}

pub async fn get_thread(
    State(state): State<AppState>,
    Path(thread_id): Path<uuid::Uuid>,
) -> AppResult<Json<ForumThread>> {
    let thread = repository::find_thread_by_id(&state.db, thread_id)
        .await?
        .ok_or_else(|| AppError::NotFound("Thread not found".to_string()))?;

    repository::increment_thread_view(&state.db, thread_id).await.ok();

    Ok(Json(thread))
}

pub async fn create_reply(
    State(state): State<AppState>,
    axum::extract::Extension(auth_user): axum::extract::Extension<AuthUser>,
    Path(thread_id): Path<uuid::Uuid>,
    Json(req): Json<CreateReplyRequest>,
) -> AppResult<Json<ForumReply>> {
    req.validate()
        .map_err(|e| AppError::BadRequest(e.to_string()))?;

    let thread = repository::find_thread_by_id(&state.db, thread_id)
        .await?
        .ok_or_else(|| AppError::NotFound("Thread not found".to_string()))?;

    if thread.is_locked {
        return Err(AppError::Forbidden);
    }

    let mut tx = state.db.begin().await?;

    let reply = sqlx::query_as::<_, ForumReply>(
        "INSERT INTO forum_replies (thread_id, user_id, body) VALUES ($1, $2, $3) RETURNING *"
    )
    .bind(thread_id)
    .bind(auth_user.user_id)
    .bind(&req.body)
    .fetch_one(&mut *tx)
    .await?;

    insert_outbox_event(&mut tx, reply.id, "novel.forum.reply.created").await?;

    tx.commit().await?;

    Ok(Json(reply))
}

pub async fn list_replies(
    State(state): State<AppState>,
    Path(thread_id): Path<uuid::Uuid>,
    Query(query): Query<ThreadListQuery>,
) -> AppResult<Json<Vec<ForumReply>>> {
    let page = query.page.unwrap_or(1).max(1);
    let per_page = query.per_page.unwrap_or(50).min(100);
    let offset = (page - 1) * per_page;

    let replies = repository::list_replies(&state.db, thread_id, per_page, offset).await?;

    Ok(Json(replies))
}
