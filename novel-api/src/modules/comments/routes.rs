use axum::{
    routing::{delete, get, post},
    Router,
};

use crate::app_state::AppState;

use super::handlers;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/", get(handlers::list_comments))
        .route("/", post(handlers::create_comment))
        .route("/{id}", delete(handlers::delete_comment))
}
