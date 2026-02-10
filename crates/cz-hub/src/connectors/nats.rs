//! # NATS Connector (optional — requires `--features nats`)
//!
//! Subscribes to a NATS subject (or JetStream consumer) and emits events
//! as [`StreamEvent`]s.

use super::{
    ConnectorInfo, ConnectorKind, ConnectorMetrics, ConnectorStatus, StreamConnector, StreamEvent,
};
use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use tokio::sync::{broadcast, RwLock};

pub struct NatsConnector {
    id: String,
    name: String,
    url: String,
    subject: String,
    status: RwLock<ConnectorStatus>,
    running: AtomicBool,
    events_total: AtomicU64,
    bytes_total: AtomicU64,
    errors_total: AtomicU64,
    tx: broadcast::Sender<StreamEvent>,
    created_at: String,
}

impl NatsConnector {
    pub fn new(name: String, params: HashMap<String, String>) -> Self {
        let (tx, _) = broadcast::channel(4096);
        let id = format!("nats-{}", uuid::Uuid::new_v4().as_simple());

        Self {
            id,
            name,
            url: params
                .get("url")
                .cloned()
                .unwrap_or_else(|| "nats://localhost:4222".into()),
            subject: params.get("subject").cloned().unwrap_or_else(|| ">".into()),
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
impl StreamConnector for NatsConnector {
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
            "NATS connector '{}' connecting to {} subject '{}'",
            self.name,
            self.url,
            self.subject
        );

        // TODO: Replace with actual async-nats subscription loop
        // let client = async_nats::connect(&self.url).await?;
        // let mut sub = client.subscribe(self.subject.clone()).await?;
        // while self.running.load(Ordering::Relaxed) {
        //     if let Some(msg) = sub.next().await {
        //         let event = StreamEvent { ... };
        //         self.tx.send(event);
        //     }
        // }

        *self.status.write().await = ConnectorStatus::Connected;
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
            kind: ConnectorKind::Nats,
            status: self.status(),
            config: serde_json::json!({
                "url": self.url,
                "subject": self.subject,
            }),
            metrics: self.metrics(),
            created_at: self.created_at.clone(),
        }
    }
}
