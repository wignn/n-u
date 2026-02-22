use axum::extract::{Path, Query, State};
use axum::Json;
use serde::Deserialize;

use crate::app_state::AppState;
use crate::error::AppResult;
use crate::middleware::auth::AuthUser;

use super::models::*;
use super::repository;

#[derive(Debug, Deserialize)]
pub struct PaginationQuery {
    pub page: Option<i64>,
    pub per_page: Option<i64>,
}

pub async fn add_bookmark(
    State(state): State<AppState>,
    axum::extract::Extension(auth_user): axum::extract::Extension<AuthUser>,
    Path(novel_id): Path<uuid::Uuid>,
) -> AppResult<Json<Bookmark>> {
    let bookmark = repository::add_bookmark(&state.db, auth_user.user_id, novel_id).await?;
    Ok(Json(bookmark))
}

pub async fn remove_bookmark(
    State(state): State<AppState>,
    axum::extract::Extension(auth_user): axum::extract::Extension<AuthUser>,
    Path(novel_id): Path<uuid::Uuid>,
) -> AppResult<axum::http::StatusCode> {
    repository::remove_bookmark(&state.db, auth_user.user_id, novel_id).await?;
    Ok(axum::http::StatusCode::NO_CONTENT)
}

pub async fn list_bookmarks(
    State(state): State<AppState>,
    axum::extract::Extension(auth_user): axum::extract::Extension<AuthUser>,
    Query(query): Query<PaginationQuery>,
) -> AppResult<Json<Vec<Bookmark>>> {
    let page = query.page.unwrap_or(1).max(1);
    let per_page = query.per_page.unwrap_or(20).min(100);
    let offset = (page - 1) * per_page;

    let bookmarks = repository::list_bookmarks(&state.db, auth_user.user_id, per_page, offset).await?;
    Ok(Json(bookmarks))
}

pub async fn update_reading_history(
    State(state): State<AppState>,
    axum::extract::Extension(auth_user): axum::extract::Extension<AuthUser>,
    Json(req): Json<UpdateReadingHistoryRequest>,
) -> AppResult<Json<ReadingHistory>> {
    let history = repository::upsert_reading_history(
        &state.db,
        auth_user.user_id,
        req.novel_id,
        req.chapter_id,
        req.chapter_number,
    )
    .await?;

    Ok(Json(history))
}

pub async fn get_reading_history(
    State(state): State<AppState>,
    axum::extract::Extension(auth_user): axum::extract::Extension<AuthUser>,
    Query(query): Query<PaginationQuery>,
) -> AppResult<Json<Vec<ReadingHistory>>> {
    let page = query.page.unwrap_or(1).max(1);
    let per_page = query.per_page.unwrap_or(20).min(100);
    let offset = (page - 1) * per_page;

    let history = repository::get_reading_history(&state.db, auth_user.user_id, per_page, offset).await?;
    Ok(Json(history))
}
