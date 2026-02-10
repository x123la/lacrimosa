//! # Stream Connector Framework
//!
//! The universal abstraction for all data sources flowing through the
//! LACRIMOSA Control Center. Every data stream — internal journal,
//! Kafka topic, NATS subject, webhook endpoint — implements [`StreamConnector`].

pub mod journal;
pub mod registry;
pub mod webhook;

#[cfg(feature = "kafka")]
pub mod kafka;
#[cfg(feature = "nats")]
pub mod nats;

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tokio::sync::broadcast;

// =============================================================================
// Core Trait
// =============================================================================

/// A normalized event emitted by any connector.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamEvent {
    /// Unique event ID (connector-scoped).
    pub id: String,
    /// Source connector ID.
    pub connector_id: String,
    /// Source stream/topic/subject name.
    pub stream: String,
    /// Logical timestamp (Lamport, Kafka offset, NATS sequence, etc).
    pub sequence: u64,
    /// Wall-clock timestamp (ISO 8601).
    pub timestamp: String,
    /// Decoded payload as JSON value (or raw hex if undecoded).
    pub payload: serde_json::Value,
    /// Optional key-value metadata (headers, trace context, etc).
    pub metadata: HashMap<String, String>,
}

/// Health status of a connector.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ConnectorStatus {
    Connected,
    Connecting,
    Disconnected,
    Error,
    Stopped,
}

/// Runtime metrics for a single connector.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ConnectorMetrics {
    pub events_total: u64,
    pub events_per_sec: f64,
    pub bytes_total: u64,
    pub bytes_per_sec: f64,
    pub errors_total: u64,
    pub last_event_at: Option<String>,
}

/// Connector type descriptor — used for the creation wizard.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ConnectorKind {
    Journal,
    Kafka,
    Nats,
    Webhook,
    Http,
}

impl std::fmt::Display for ConnectorKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Journal => write!(f, "journal"),
            Self::Kafka => write!(f, "kafka"),
            Self::Nats => write!(f, "nats"),
            Self::Webhook => write!(f, "webhook"),
            Self::Http => write!(f, "http"),
        }
    }
}

/// Serializable connector info for API responses.
#[derive(Debug, Clone, Serialize)]
pub struct ConnectorInfo {
    pub id: String,
    pub name: String,
    pub kind: ConnectorKind,
    pub status: ConnectorStatus,
    pub config: serde_json::Value,
    pub metrics: ConnectorMetrics,
    pub created_at: String,
}

/// Configuration for creating a new connector.
#[derive(Debug, Clone, Deserialize)]
pub struct ConnectorConfig {
    pub name: String,
    pub kind: ConnectorKind,
    /// Connector-specific configuration (brokers, topic, subject, etc).
    #[serde(default)]
    pub params: HashMap<String, String>,
}

/// The core trait every data source must implement.
///
/// Connectors are long-lived async tasks that emit [`StreamEvent`]s via
/// a broadcast channel. The registry manages their lifecycle.
#[async_trait::async_trait]
pub trait StreamConnector: Send + Sync {
    /// Unique identifier for this connector instance.
    fn id(&self) -> &str;

    /// Current health status.
    fn status(&self) -> ConnectorStatus;

    /// Start consuming events. Returns when stopped.
    async fn start(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;

    /// Signal the connector to stop.
    async fn stop(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;

    /// Subscribe to the event stream.
    fn subscribe(&self) -> broadcast::Receiver<StreamEvent>;

    /// Snapshot of current metrics.
    fn metrics(&self) -> ConnectorMetrics;

    /// Get serializable info.
    fn info(&self) -> ConnectorInfo;

    /// Ingest a payload (for push-based connectors like Webhooks).
    async fn ingest(
        &self,
        _payload: serde_json::Value,
        _headers: HashMap<String, String>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        Err("Ingestion not supported by this connector".into())
    }
}
