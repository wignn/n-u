use axum::{
    routing::{get, post, put},
    Router,
};

use crate::app_state::AppState;

use super::handlers;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/register", post(handlers::register))
        .route("/login", post(handlers::login))
        .route("/{user_id}/profile", get(handlers::get_profile))
        .route("/me", get(handlers::me))
        .route("/me/profile", put(handlers::update_profile))
}
