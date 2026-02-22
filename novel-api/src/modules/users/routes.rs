use axum::{
    middleware,
    routing::{get, post, put},
    Router,
};

use crate::app_state::AppState;
use crate::middleware::auth::auth_middleware;

use super::handlers;

pub fn routes() -> Router<AppState> {
    let public = Router::new()
        .route("/register", post(handlers::register))
        .route("/login", post(handlers::login))
        .route("/{user_id}/profile", get(handlers::get_profile));

    let protected = Router::new()
        .route("/me", get(handlers::me))
        .route("/me/profile", put(handlers::update_profile))
        .layer(middleware::from_fn_with_state(
            AppState::clone(&AppState::clone),
            auth_middleware,
        ));

    Router::new().merge(public).merge(protected)
}
