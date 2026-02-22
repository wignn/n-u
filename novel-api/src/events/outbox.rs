use sqlx::PgPool;
use uuid::Uuid;
use chrono::Utc;

use super::publisher::{EventPayload, EventPublisher};

pub async fn insert_outbox_event(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    entity_id: Uuid,
    event_type: &str,
) -> Result<i64, sqlx::Error> {
    let row = sqlx::query_scalar::<_, i64>(
        "INSERT INTO outbox_events (entity_id, event_type, timestamp) VALUES ($1, $2, $3) RETURNING id"
    )
    .bind(entity_id)
    .bind(event_type)
    .bind(Utc::now())
    .fetch_one(&mut **tx)
    .await?;

    Ok(row)
}

pub async fn poll_and_publish(
    pool: &PgPool,
    publisher: &EventPublisher,
    batch_size: i64,
) -> Result<usize, anyhow::Error> {
    let rows = sqlx::query_as::<_, OutboxRow>(
        "SELECT id, entity_id, event_type, timestamp FROM outbox_events WHERE published = FALSE ORDER BY id LIMIT $1 FOR UPDATE SKIP LOCKED"
    )
    .bind(batch_size)
    .fetch_all(pool)
    .await?;

    let count = rows.len();

    for row in rows {
        let payload = EventPayload {
            entity_id: row.entity_id,
            event_type: row.event_type.clone(),
            timestamp: row.timestamp,
        };

        publisher.publish(&row.event_type, &payload).await?;

        sqlx::query("UPDATE outbox_events SET published = TRUE, published_at = NOW() WHERE id = $1")
            .bind(row.id)
            .execute(pool)
            .await?;
    }

    Ok(count)
}

pub async fn start_outbox_poller(pool: PgPool, publisher: EventPublisher) {
    let mut interval = tokio::time::interval(std::time::Duration::from_millis(500));

    loop {
        interval.tick().await;

        match poll_and_publish(&pool, &publisher, 100).await {
            Ok(count) => {
                if count > 0 {
                    tracing::debug!("Published {} outbox events", count);
                }
            }
            Err(e) => {
                tracing::error!("Outbox poller error: {:?}", e);
                tokio::time::sleep(std::time::Duration::from_secs(5)).await;
            }
        }
    }
}

#[derive(sqlx::FromRow)]
struct OutboxRow {
    id: i64,
    entity_id: Uuid,
    event_type: String,
    timestamp: chrono::DateTime<Utc>,
}
