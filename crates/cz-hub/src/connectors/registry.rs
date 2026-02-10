//! # Connector Registry
//!
//! Thread-safe manager for all active [`StreamConnector`] instances.
//! Handles creation, lifecycle, event fan-out, and metrics aggregation.

use super::{
    ConnectorConfig, ConnectorInfo, ConnectorKind, StreamConnector, StreamEvent,
};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{broadcast, RwLock};

/// Central registry for all active connectors.
pub struct ConnectorRegistry {
    connectors: RwLock<HashMap<String, Arc<dyn StreamConnector>>>,
    /// Unified event bus — all connectors fan-in here.
    event_tx: broadcast::Sender<StreamEvent>,
    /// Buffer of recent events for query engine access.
    event_buffer: Arc<RwLock<Vec<StreamEvent>>>,
    buffer_capacity: usize,
}

impl ConnectorRegistry {
    pub fn new(buffer_capacity: usize) -> Self {
        let (event_tx, _) = broadcast::channel(4096);
        Self {
            connectors: RwLock::new(HashMap::new()),
            event_tx,
            event_buffer: Arc::new(RwLock::new(Vec::with_capacity(buffer_capacity))),
            buffer_capacity,
        }
    }

    /// Register and start a connector.
    pub async fn add(
        &self,
        connector: Arc<dyn StreamConnector>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let id = connector.id().to_string();

        // Store it
        {
            let mut connectors = self.connectors.write().await;
            connectors.insert(id.clone(), connector.clone());
        }

        // Spawn a task that forwards events to the unified bus
        let tx = self.event_tx.clone();
        let buffer = self.event_buffer.clone();
        let cap = self.buffer_capacity;
        let mut rx = connector.subscribe();

        tokio::spawn(async move {
            loop {
                match rx.recv().await {
                    Ok(event) => {
                        let _ = tx.send(event.clone());
                        // Buffer for query engine
                        let mut buf: tokio::sync::RwLockWriteGuard<Vec<StreamEvent>> =
                            buffer.write().await;
                        if buf.len() >= cap {
                            buf.remove(0);
                        }
                        buf.push(event);
                    }
                    Err(broadcast::error::RecvError::Lagged(n)) => {
                        tracing::warn!("Connector event bus lagged by {} events", n);
                    }
                    Err(broadcast::error::RecvError::Closed) => break,
                }
            }
        });

        // Start the connector
        let c = connector.clone();
        tokio::spawn(async move {
            if let Err(e) = c.start().await {
                tracing::error!("Connector {} failed: {}", c.id(), e);
            }
        });

        Ok(())
    }

    /// Remove and stop a connector.
    pub async fn remove(&self, id: &str) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let connector = {
            let mut connectors = self.connectors.write().await;
            connectors.remove(id)
        };

        if let Some(c) = connector {
            c.stop().await?;
            Ok(())
        } else {
            Err(format!("Connector '{}' not found", id).into())
        }
    }

    /// List all connectors with their current info.
    pub async fn list(&self) -> Vec<ConnectorInfo> {
        let connectors = self.connectors.read().await;
        connectors.values().map(|c| c.info()).collect()
    }

    /// Get a specific connector.
    pub async fn get(&self, id: &str) -> Option<Arc<dyn StreamConnector>> {
        let connectors = self.connectors.read().await;
        connectors.get(id).cloned()
    }

    /// Get the buffered events (for query engine).
    pub async fn buffered_events(&self) -> Vec<StreamEvent> {
        self.event_buffer.read().await.clone()
    }

    /// Create a connector from config and register it.
    pub async fn create_from_config(
        &self,
        config: ConnectorConfig,
    ) -> Result<ConnectorInfo, Box<dyn std::error::Error + Send + Sync>> {
        let connector: Arc<dyn StreamConnector> = match config.kind {
            ConnectorKind::Webhook => Arc::new(super::webhook::WebhookConnector::new(
                config.name.clone(),
                config.params.clone(),
            )),
            #[cfg(feature = "kafka")]
            ConnectorKind::Kafka => Arc::new(super::kafka::KafkaConnector::new(
                config.name.clone(),
                config.params.clone(),
            )),
            #[cfg(not(feature = "kafka"))]
            ConnectorKind::Kafka => {
                return Err("Kafka support not compiled. Rebuild with --features kafka".into());
            }
            #[cfg(feature = "nats")]
            ConnectorKind::Nats => Arc::new(super::nats::NatsConnector::new(
                config.name.clone(),
                config.params.clone(),
            )),
            #[cfg(not(feature = "nats"))]
            ConnectorKind::Nats => {
                return Err("NATS support not compiled. Rebuild with --features nats".into());
            }
            ConnectorKind::Journal => {
                return Err("Journal connectors are managed automatically".into());
            }
            ConnectorKind::Http => {
                return Err("HTTP polling connector not yet implemented".into());
            }
        };

        let info = connector.info();
        self.add(connector).await?;
        Ok(info)
    }
}
