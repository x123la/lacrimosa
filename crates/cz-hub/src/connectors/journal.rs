//! # Internal Journal Connector
//!
//! Wraps the existing `cz-io` journal as a [`StreamConnector`], unifying
//! it with external data sources under the same abstraction.

use super::{
    ConnectorInfo, ConnectorKind, ConnectorMetrics, ConnectorStatus, StreamConnector, StreamEvent,
};
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use tokio::sync::{broadcast, RwLock};

pub struct JournalConnector {
    id: String,
    name: String,
    path: PathBuf,
    status: RwLock<ConnectorStatus>,
    running: AtomicBool,
    events_total: AtomicU64,
    bytes_total: AtomicU64,
    tx: broadcast::Sender<StreamEvent>,
    created_at: String,
}

impl JournalConnector {
    pub fn new(path: PathBuf) -> Self {
        let (tx, _) = broadcast::channel(2048);
        let name = format!(
            "journal:{}",
            path.file_name()
                .map(|f| f.to_string_lossy().to_string())
                .unwrap_or_else(|| "unknown".into())
        );
        let id = format!("journal-{}", uuid::Uuid::new_v4().as_simple());

        Self {
            id,
            name,
            path,
            status: RwLock::new(ConnectorStatus::Connected),
            running: AtomicBool::new(true),
            events_total: AtomicU64::new(0),
            bytes_total: AtomicU64::new(0),
            tx,
            created_at: chrono::Utc::now().to_rfc3339(),
        }
    }

}

#[async_trait::async_trait]
impl StreamConnector for JournalConnector {
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
        // Journal connector is passive — events are pushed in by the metrics collector.
        // Just mark as running.
        self.running.store(true, Ordering::Relaxed);
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
            ..Default::default()
        }
    }

    fn info(&self) -> ConnectorInfo {
        ConnectorInfo {
            id: self.id.clone(),
            name: self.name.clone(),
            kind: ConnectorKind::Journal,
            status: self.status(),
            config: serde_json::json!({ "path": self.path.to_string_lossy() }),
            metrics: self.metrics(),
            created_at: self.created_at.clone(),
        }
    }
}
