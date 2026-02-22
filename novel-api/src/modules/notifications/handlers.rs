use axum::extract::{Path, Query, State};
use axum::Json;

use crate::app_state::AppState;
use crate::error::AppResult;
use crate::middleware::auth::AuthUser;

use super::models::*;
use super::repository;

pub async fn list_notifications(
    State(state): State<AppState>,
    axum::extract::Extension(auth_user): axum::extract::Extension<AuthUser>,
    Query(query): Query<NotificationQuery>,
) -> AppResult<Json<Vec<Notification>>> {
    let page = query.page.unwrap_or(1).max(1);
    let per_page = query.per_page.unwrap_or(20).min(50);
    let offset = (page - 1) * per_page;
    let unread_only = query.unread_only.unwrap_or(false);

    let notifications = repository::list_notifications(
        &state.db,
        auth_user.user_id,
        unread_only,
        per_page,
        offset,
    )
    .await?;

    Ok(Json(notifications))
}

pub async fn unread_count(
    State(state): State<AppState>,
    axum::extract::Extension(auth_user): axum::extract::Extension<AuthUser>,
) -> AppResult<Json<UnreadCount>> {
    let count = repository::unread_count(&state.db, auth_user.user_id).await?;
    Ok(Json(UnreadCount { count }))
}

pub async fn mark_as_read(
    State(state): State<AppState>,
    axum::extract::Extension(auth_user): axum::extract::Extension<AuthUser>,
    Path(notification_id): Path<uuid::Uuid>,
) -> AppResult<axum::http::StatusCode> {
    repository::mark_as_read(&state.db, auth_user.user_id, notification_id).await?;
    Ok(axum::http::StatusCode::NO_CONTENT)
}

pub async fn mark_all_as_read(
    State(state): State<AppState>,
    axum::extract::Extension(auth_user): axum::extract::Extension<AuthUser>,
) -> AppResult<axum::http::StatusCode> {
    repository::mark_all_as_read(&state.db, auth_user.user_id).await?;
    Ok(axum::http::StatusCode::NO_CONTENT)
}
