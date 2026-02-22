use axum::{
    routing::{delete, get, post, put},
    Router,
};

use crate::app_state::AppState;

use super::handlers;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/bookmarks", get(handlers::list_bookmarks))
        .route("/bookmarks/{novel_id}", post(handlers::add_bookmark))
        .route("/bookmarks/{novel_id}", delete(handlers::remove_bookmark))
        .route("/history", get(handlers::get_reading_history))
        .route("/history", put(handlers::update_reading_history))
}
