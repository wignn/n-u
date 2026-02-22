use crate::config::Config;
use crate::events::publisher::EventPublisher;
use redis::aio::ConnectionManager;
use sqlx::PgPool;

#[derive(Clone)]
pub struct AppState {
    pub db: PgPool,
    pub redis: ConnectionManager,
    pub config: Config,
    pub event_publisher: EventPublisher,
    pub search_client: meilisearch_sdk::Client,
}
