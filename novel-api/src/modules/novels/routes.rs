use axum::{
    routing::{delete, get, post, put},
    Router,
};

use crate::app_state::AppState;

use super::handlers;

pub fn routes() -> Router<AppState> {
    let public = Router::new()
        .route("/", get(handlers::list_novels))
        .route("/{id}", get(handlers::get_novel))
        .route("/by-slug/{slug}", get(handlers::get_novel_by_slug));

    let protected = Router::new()
        .route("/", post(handlers::create_novel))
        .route("/{id}", put(handlers::update_novel))
        .route("/{id}", delete(handlers::delete_novel));

    Router::new().merge(public).merge(protected)
}
