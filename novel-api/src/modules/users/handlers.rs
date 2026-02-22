use axum::{extract::State, Json};
use validator::Validate;

use crate::app_state::AppState;
use crate::error::{AppError, AppResult};
use crate::middleware::auth::{generate_token, AuthUser};

use super::models::*;
use super::repository;

pub async fn register(
    State(state): State<AppState>,
    Json(req): Json<RegisterRequest>,
) -> AppResult<Json<AuthResponse>> {
    req.validate()
        .map_err(|e| AppError::BadRequest(e.to_string()))?;

    let password_hash =
        bcrypt::hash(&req.password, 12).map_err(|e| AppError::Internal(e.into()))?;

    let user = repository::create_user(
        &state.db,
        &req.username,
        &req.email,
        &password_hash,
        req.display_name.as_deref(),
    )
    .await?;

    let token = generate_token(
        user.id,
        &user.role,
        &state.config.jwt_secret,
        state.config.jwt_expiry_hours,
    )?;

    Ok(Json(AuthResponse {
        token,
        user: UserProfile {
            id: user.id,
            username: user.username,
            display_name: user.display_name,
            avatar_url: user.avatar_url,
            bio: user.bio,
            created_at: user.created_at,
        },
    }))
}

pub async fn login(
    State(state): State<AppState>,
    Json(req): Json<LoginRequest>,
) -> AppResult<Json<AuthResponse>> {
    let user = repository::find_by_email(&state.db, &req.email)
        .await?
        .ok_or(AppError::Unauthorized)?;

    if user.status != "active" {
        return Err(AppError::Forbidden);
    }

    let valid =
        bcrypt::verify(&req.password, &user.password_hash).map_err(|e| AppError::Internal(e.into()))?;

    if !valid {
        return Err(AppError::Unauthorized);
    }

    repository::update_last_login(&state.db, user.id).await?;

    let token = generate_token(
        user.id,
        &user.role,
        &state.config.jwt_secret,
        state.config.jwt_expiry_hours,
    )?;

    Ok(Json(AuthResponse {
        token,
        user: UserProfile {
            id: user.id,
            username: user.username,
            display_name: user.display_name,
            avatar_url: user.avatar_url,
            bio: user.bio,
            created_at: user.created_at,
        },
    }))
}

pub async fn get_profile(
    State(state): State<AppState>,
    axum::extract::Path(user_id): axum::extract::Path<uuid::Uuid>,
) -> AppResult<Json<UserProfile>> {
    let profile = repository::get_profile(&state.db, user_id).await?;
    Ok(Json(profile))
}

pub async fn update_profile(
    State(state): State<AppState>,
    axum::extract::Extension(auth_user): axum::extract::Extension<AuthUser>,
    Json(req): Json<UpdateProfileRequest>,
) -> AppResult<Json<UserProfile>> {
    req.validate()
        .map_err(|e| AppError::BadRequest(e.to_string()))?;

    let user = repository::update_profile(
        &state.db,
        auth_user.user_id,
        req.display_name.as_deref(),
        req.avatar_url.as_deref(),
        req.bio.as_deref(),
    )
    .await?;

    Ok(Json(UserProfile {
        id: user.id,
        username: user.username,
        display_name: user.display_name,
        avatar_url: user.avatar_url,
        bio: user.bio,
        created_at: user.created_at,
    }))
}

pub async fn me(
    State(state): State<AppState>,
    axum::extract::Extension(auth_user): axum::extract::Extension<AuthUser>,
) -> AppResult<Json<UserProfile>> {
    let profile = repository::get_profile(&state.db, auth_user.user_id).await?;
    Ok(Json(profile))
}
