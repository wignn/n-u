use axum::{middleware as axum_mw, Router};
use tower_http::cors::{Any, CorsLayer};
use tower_http::trace::TraceLayer;
use tracing_subscriber::EnvFilter;

use novel_api::app_state::AppState;
use novel_api::config::Config;
use novel_api::db;
use novel_api::events::outbox::start_outbox_poller;
use novel_api::events::publisher::EventPublisher;
use novel_api::middleware::auth::auth_middleware;
use novel_api::middleware::request_id::request_id_middleware;
use novel_api::modules;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .json()
        .init();

    let config = Config::from_env()?;

    let pool = db::init_pool(&config.database_url).await?;
    db::run_migrations(&pool).await?;
    tracing::info!("Database connected and migrations applied");

    let redis = redis::Client::open(config.redis_url.as_str())?
        .get_connection_manager()
        .await?;
    tracing::info!("Redis connected");

    let event_publisher = EventPublisher::new(&config.nats_url)
        .await
        .map_err(|e| anyhow::anyhow!(e))?;
    tracing::info!("NATS JetStream connected");

    let search_client = meilisearch_sdk::client::Client::new(
        &config.meilisearch_url,
        Some(&config.meilisearch_api_key),
    )?;
    tracing::info!("Meilisearch client initialized");

    let state = AppState {
        db: pool.clone(),
        redis,
        config: config.clone(),
        event_publisher: event_publisher.clone(),
        search_client,
    };

    tokio::spawn(start_outbox_poller(pool.clone(), event_publisher.clone()));
    tracing::info!("Outbox poller started");

    let protected_routes = Router::new()
        .nest("/users", modules::users::routes::routes())
        .nest("/novels", modules::novels::routes::routes())
        .nest("/comments", modules::comments::routes::routes())
        .nest("/bookmarks", modules::bookmarks::routes::routes())
        .nest("/notifications", modules::notifications::routes::routes())
        .nest("/admin", modules::admin::routes::routes())
        .layer(axum_mw::from_fn_with_state(state.clone(), auth_middleware));

    let public_routes = Router::new()
        .nest("/users", modules::users::routes::routes())
        .nest("/novels", modules::novels::routes::routes())
        .nest("/search", modules::search::routes::routes())
        .nest("/forums", modules::forums::routes::routes())
        .merge(modules::chapters::routes::routes())
        .merge(modules::reviews::routes::routes());

    let app = Router::new()
        .nest("/api/v1", public_routes)
        .nest("/api/v1", protected_routes)
        .layer(axum_mw::from_fn(request_id_middleware))
        .layer(TraceLayer::new_for_http())
        .layer(
            CorsLayer::new()
                .allow_origin(Any)
                .allow_methods(Any)
                .allow_headers(Any),
        )
        .with_state(state);

    let addr = config.server_addr();
    tracing::info!("Server starting on {}", addr);

    let listener = tokio::net::TcpListener::bind(&addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
