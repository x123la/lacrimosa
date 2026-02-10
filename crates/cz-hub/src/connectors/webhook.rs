//! # Webhook Connector
//!
//! HTTP ingestion endpoint that receives POST payloads and emits them
//! as [`StreamEvent`]s. Supports JSON, form, and raw body formats.
//! Provider-specific schema mapping (GitHub, Stripe, PagerDuty) normalizes
//! incoming payloads to a common structure.

use super::{
    ConnectorInfo, ConnectorKind, ConnectorMetrics, ConnectorStatus, StreamConnector, StreamEvent,
};
use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use tokio::sync::{broadcast, RwLock};

pub struct WebhookConnector {
    id: String,
    name: String,
    status: RwLock<ConnectorStatus>,
    running: AtomicBool,
    events_total: AtomicU64,
    bytes_total: AtomicU64,
    errors_total: AtomicU64,
    tx: broadcast::Sender<StreamEvent>,
    params: HashMap<String, String>,
    created_at: String,
    sequence: AtomicU64,
}

impl WebhookConnector {
    pub fn new(name: String, params: HashMap<String, String>) -> Self {
        let (tx, _) = broadcast::channel(2048);
        let id = format!("webhook-{}", uuid::Uuid::new_v4().as_simple());

        Self {
            id,
            name,
            status: RwLock::new(ConnectorStatus::Stopped),
            running: AtomicBool::new(false),
            events_total: AtomicU64::new(0),
            bytes_total: AtomicU64::new(0),
            errors_total: AtomicU64::new(0),
            tx,
            params,
            created_at: chrono::Utc::now().to_rfc3339(),
            sequence: AtomicU64::new(0),
        }
    }

    /// Ingest a webhook payload. Called by the HTTP route handler.
    fn ingest_payload(
        &self,
        payload: serde_json::Value,
        headers: HashMap<String, String>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let seq = self.sequence.fetch_add(1, Ordering::Relaxed);
        let provider = self
            .params
            .get("provider")
            .cloned()
            .unwrap_or_else(|| "generic".into());

        // Normalize payload based on provider
        let normalized = self.normalize_payload(&provider, &payload, &headers);

        let event = StreamEvent {
            id: format!("{}-{}", self.id, seq),
            connector_id: self.id.clone(),
            stream: format!("webhook:{}", provider),
            sequence: seq,
            timestamp: chrono::Utc::now().to_rfc3339(),
            payload: normalized,
            metadata: headers,
        };

        let payload_size = event.payload.to_string().len() as u64;
        self.events_total.fetch_add(1, Ordering::Relaxed);
        self.bytes_total.fetch_add(payload_size, Ordering::Relaxed);

        let _ = self.tx.send(event);
        Ok(())
    }

    fn normalize_payload(
        &self,
        provider: &str,
        payload: &serde_json::Value,
        _headers: &HashMap<String, String>,
    ) -> serde_json::Value {
        match provider {
            "github" => {
                // Extract key fields from GitHub webhook events
                serde_json::json!({
                    "provider": "github",
                    "action": payload.get("action").unwrap_or(&serde_json::Value::Null),
                    "repository": payload.pointer("/repository/full_name").unwrap_or(&serde_json::Value::Null),
                    "sender": payload.pointer("/sender/login").unwrap_or(&serde_json::Value::Null),
                    "raw": payload,
                })
            }
            "stripe" => {
                serde_json::json!({
                    "provider": "stripe",
                    "type": payload.get("type").unwrap_or(&serde_json::Value::Null),
                    "id": payload.get("id").unwrap_or(&serde_json::Value::Null),
                    "data": payload.get("data").unwrap_or(&serde_json::Value::Null),
                })
            }
            "pagerduty" => {
                serde_json::json!({
                    "provider": "pagerduty",
                    "event_action": payload.pointer("/event/event_action").unwrap_or(&serde_json::Value::Null),
                    "incident": payload.pointer("/event/data").unwrap_or(&serde_json::Value::Null),
                })
            }
            _ => {
                // Generic — pass through
                payload.clone()
            }
        }
    }
}

#[async_trait::async_trait]
impl StreamConnector for WebhookConnector {
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
        *self.status.write().await = ConnectorStatus::Connected;
        tracing::info!("Webhook connector '{}' started", self.name);
        Ok(())
    }

    async fn stop(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        self.running.store(false, Ordering::Relaxed);
        *self.status.write().await = ConnectorStatus::Stopped;
        tracing::info!("Webhook connector '{}' stopped", self.name);
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
        let mut config = serde_json::Map::new();
        for (k, v) in &self.params {
            config.insert(k.clone(), serde_json::Value::String(v.clone()));
        }
        ConnectorInfo {
            id: self.id.clone(),
            name: self.name.clone(),
            kind: ConnectorKind::Webhook,
            status: self.status(),
            config: serde_json::Value::Object(config),
            metrics: self.metrics(),
            created_at: self.created_at.clone(),
        }
    }

    async fn ingest(
        &self,
        payload: serde_json::Value,
        headers: HashMap<String, String>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        self.ingest_payload(payload, headers)
    }
}
