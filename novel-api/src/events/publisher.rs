use async_nats::jetstream;
use serde::Serialize;
use uuid::Uuid;

#[derive(Debug, Serialize, Clone)]
pub struct EventPayload {
    pub entity_id: Uuid,
    pub event_type: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

#[derive(Clone)]
pub struct EventPublisher {
    pub jetstream: jetstream::Context,
}

impl EventPublisher {
    pub async fn new(nats_url: &str) -> Result<Self, async_nats::Error> {
        let client = async_nats::connect(nats_url).await?;
        let jetstream = jetstream::new(client);
        Ok(Self { jetstream })
    }

    pub async fn publish(&self, subject: &str, payload: &EventPayload) -> Result<(), anyhow::Error> {
        let data = serde_json::to_vec(payload)?;
        self.jetstream
            .publish(subject.to_string(), data.into())
            .await?
            .await?;
        Ok(())
    }
}
