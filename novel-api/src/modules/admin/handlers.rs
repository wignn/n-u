use axum::extract::{Path, Query, State};
use axum::Json;

use crate::app_state::AppState;
use crate::error::AppResult;
use crate::middleware::auth::{require_role, AuthUser};

use super::models::*;
use super::repository;

pub async fn create_report(
    State(state): State<AppState>,
    axum::extract::Extension(auth_user): axum::extract::Extension<AuthUser>,
    Json(req): Json<CreateReportRequest>,
) -> AppResult<Json<Report>> {
    let report = repository::create_report(
        &state.db,
        auth_user.user_id,
        &req.entity_type,
        req.entity_id,
        &req.reason,
    )
    .await?;

    Ok(Json(report))
}

pub async fn list_reports(
    State(state): State<AppState>,
    axum::extract::Extension(auth_user): axum::extract::Extension<AuthUser>,
    Query(query): Query<ReportListQuery>,
) -> AppResult<Json<Vec<Report>>> {
    require_role(&auth_user, "moderator")?;

    let page = query.page.unwrap_or(1).max(1);
    let per_page = query.per_page.unwrap_or(20).min(50);
    let offset = (page - 1) * per_page;

    let reports =
        repository::list_reports(&state.db, query.status.as_deref(), per_page, offset).await?;

    Ok(Json(reports))
}

pub async fn resolve_report(
    State(state): State<AppState>,
    axum::extract::Extension(auth_user): axum::extract::Extension<AuthUser>,
    Path(report_id): Path<uuid::Uuid>,
    Json(req): Json<ResolveReportRequest>,
) -> AppResult<Json<Report>> {
    require_role(&auth_user, "moderator")?;

    let report = repository::resolve_report(
        &state.db,
        report_id,
        auth_user.user_id,
        &req.status,
        req.moderator_note.as_deref(),
    )
    .await?;

    Ok(Json(report))
}

pub async fn set_shadowban(
    State(state): State<AppState>,
    axum::extract::Extension(auth_user): axum::extract::Extension<AuthUser>,
    Json(req): Json<ShadowbanRequest>,
) -> AppResult<axum::http::StatusCode> {
    require_role(&auth_user, "admin")?;
    repository::set_shadowban(&state.db, req.user_id, req.shadowban).await?;
    Ok(axum::http::StatusCode::NO_CONTENT)
}

pub async fn update_user_status(
    State(state): State<AppState>,
    axum::extract::Extension(auth_user): axum::extract::Extension<AuthUser>,
    Json(req): Json<UpdateUserStatusRequest>,
) -> AppResult<axum::http::StatusCode> {
    require_role(&auth_user, "admin")?;
    repository::update_user_status(&state.db, req.user_id, &req.status).await?;
    Ok(axum::http::StatusCode::NO_CONTENT)
}

pub async fn get_stats(
    State(state): State<AppState>,
    axum::extract::Extension(auth_user): axum::extract::Extension<AuthUser>,
) -> AppResult<Json<AdminStats>> {
    require_role(&auth_user, "moderator")?;
    let stats = repository::get_stats(&state.db).await?;
    Ok(Json(stats))
}
