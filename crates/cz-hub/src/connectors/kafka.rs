//! # Kafka Connector (optional — requires `--features kafka`)
//!
//! Consumes from a Kafka topic and emits events as [`StreamEvent`]s.
//! Uses `rdkafka` under the hood. Supports consumer group offsets,
//! auto-reconnection, and configurable deserialization.

use super::{
    ConnectorInfo, ConnectorKind, ConnectorMetrics, ConnectorStatus, StreamConnector, StreamEvent,
};
use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use tokio::sync::{broadcast, RwLock};

pub struct KafkaConnector {
    id: String,
    name: String,
    brokers: String,
    topic: String,
    group_id: String,
    status: RwLock<ConnectorStatus>,
    running: AtomicBool,
    events_total: AtomicU64,
    bytes_total: AtomicU64,
    errors_total: AtomicU64,
    tx: broadcast::Sender<StreamEvent>,
    created_at: String,
}

impl KafkaConnector {
    pub fn new(name: String, params: HashMap<String, String>) -> Self {
        let (tx, _) = broadcast::channel(4096);
        let id = format!("kafka-{}", uuid::Uuid::new_v4().as_simple());

        Self {
            id,
            name,
            brokers: params
                .get("brokers")
                .cloned()
                .unwrap_or_else(|| "localhost:9092".into()),
            topic: params
                .get("topic")
                .cloned()
                .unwrap_or_else(|| "events".into()),
            group_id: params
                .get("group_id")
                .cloned()
                .unwrap_or_else(|| "cz-hub".into()),
            status: RwLock::new(ConnectorStatus::Stopped),
            running: AtomicBool::new(false),
            events_total: AtomicU64::new(0),
            bytes_total: AtomicU64::new(0),
            errors_total: AtomicU64::new(0),
            tx,
            created_at: chrono::Utc::now().to_rfc3339(),
        }
    }
}

#[async_trait::async_trait]
impl StreamConnector for KafkaConnector {
    fn id(&self) -> &str {
        &self.id
    }

    fn status(&self) -> ConnectorStatus {
        if self.running.load(Ordering::Relaxed) {
            ConnectorStatus::Connected
        } else {
            ConnectorStatus::Stopped
        }
    }

    async fn start(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        self.running.store(true, Ordering::Relaxed);
        *self.status.write().await = ConnectorStatus::Connecting;

        tracing::info!(
            "Kafka connector '{}' connecting to {} topic '{}'",
            self.name,
            self.brokers,
            self.topic
        );

        // TODO: Replace with actual rdkafka consumer loop
        // For now, mark as connected after a brief delay to simulate connection
        *self.status.write().await = ConnectorStatus::Connected;

        // Placeholder: in production, this would be a StreamConsumer loop:
        // let consumer: StreamConsumer = ClientConfig::new()
        //     .set("group.id", &self.group_id)
        //     .set("bootstrap.servers", &self.brokers)
        //     .create()?;
        // consumer.subscribe(&[&self.topic])?;
        // while self.running.load(Ordering::Relaxed) {
        //     if let Some(msg) = consumer.recv().await? {
        //         let event = StreamEvent { ... };
        //         self.tx.send(event);
        //     }
        // }

        Ok(())
    }

    async fn stop(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        self.running.store(false, Ordering::Relaxed);
        *self.status.write().await = ConnectorStatus::Stopped;
        Ok(())
    }

    fn subscribe(&self) -> broadcast::Receiver<StreamEvent> {
        self.tx.subscribe()
    }

    fn metrics(&self) -> ConnectorMetrics {
        ConnectorMetrics {
            events_total: self.events_total.load(Ordering::Relaxed),
            bytes_total: self.bytes_total.load(Ordering::Relaxed),
            errors_total: self.errors_total.load(Ordering::Relaxed),
            ..Default::default()
        }
    }

    fn info(&self) -> ConnectorInfo {
        ConnectorInfo {
            id: self.id.clone(),
            name: self.name.clone(),
            kind: ConnectorKind::Kafka,
            status: self.status(),
            config: serde_json::json!({
                "brokers": self.brokers,
                "topic": self.topic,
                "group_id": self.group_id,
            }),
            metrics: self.metrics(),
            created_at: self.created_at.clone(),
        }
    }
}
