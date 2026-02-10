//! # Stream Composition Pipelines
//!
//! Visual pipeline builder backend: define filter → join → aggregate chains
//! that process events from one or more connectors in real-time.

use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;

/// Pipeline status.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum PipelineStatus {
    Running,
    Stopped,
    Error,
}

/// A processing pipeline definition.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Pipeline {
    pub id: String,
    pub name: String,
    pub description: String,
    pub nodes: Vec<PipelineNode>,
    pub edges: Vec<PipelineEdge>,
    pub status: PipelineStatus,
    pub created_at: String,
    pub event_count: u64,
    pub error_count: u64,
}

/// A node in the pipeline graph.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PipelineNode {
    pub id: String,
    pub node_type: PipelineNodeType,
    pub config: serde_json::Value,
    pub position: Option<NodePosition>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodePosition {
    pub x: f64,
    pub y: f64,
}

/// Node types.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum PipelineNodeType {
    /// Source: reads from a connector
    Source,
    /// Filter: passes/blocks events based on conditions
    Filter,
    /// Transform: modifies event fields
    Transform,
    /// Join: correlates events from two streams
    Join,
    /// Aggregate: rolling window statistics
    Aggregate,
    /// Sink: writes to a connector or output
    Sink,
}

/// An edge connecting two nodes.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PipelineEdge {
    pub from_node: String,
    pub to_node: String,
}

/// Request to create a pipeline.
#[derive(Debug, Clone, Deserialize)]
pub struct CreatePipelineRequest {
    pub name: String,
    pub description: Option<String>,
    pub nodes: Vec<PipelineNode>,
    pub edges: Vec<PipelineEdge>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct UpdatePipelineRequest {
    pub nodes: Vec<PipelineNode>,
    pub edges: Vec<PipelineEdge>,
}

/// The pipeline manager.
pub struct PipelineManager {
    pub pipelines: RwLock<Vec<Pipeline>>,
}

impl PipelineManager {
    pub fn new() -> Self {
        Self {
            pipelines: RwLock::new(Vec::new()),
        }
    }

    pub async fn create(&self, req: CreatePipelineRequest) -> Pipeline {
        let pipeline = Pipeline {
            id: format!("pipe-{}", uuid::Uuid::new_v4().as_simple()),
            name: req.name,
            description: req.description.unwrap_or_default(),
            nodes: req.nodes,
            edges: req.edges,
            status: PipelineStatus::Stopped,
            created_at: chrono::Utc::now().to_rfc3339(),
            event_count: 0,
            error_count: 0,
        };

        let mut pipelines = self.pipelines.write().await;
        pipelines.push(pipeline.clone());
        pipeline
    }

    pub async fn list(&self) -> Vec<Pipeline> {
        self.pipelines.read().await.clone()
    }

    pub async fn get(&self, id: &str) -> Option<Pipeline> {
        self.pipelines
            .read()
            .await
            .iter()
            .find(|p| p.id == id)
            .cloned()
    }

    pub async fn delete(&self, id: &str) -> Result<(), String> {
        let mut pipelines = self.pipelines.write().await;
        let idx = pipelines
            .iter()
            .position(|p| p.id == id)
            .ok_or_else(|| format!("Pipeline '{}' not found", id))?;
        pipelines.remove(idx);
        Ok(())
    }

    pub async fn set_status(&self, id: &str, status: PipelineStatus) -> Result<Pipeline, String> {
        let mut pipelines = self.pipelines.write().await;
        let pipeline = pipelines
            .iter_mut()
            .find(|p| p.id == id)
            .ok_or_else(|| format!("Pipeline '{}' not found", id))?;
        pipeline.status = status;
        Ok(pipeline.clone())
    }

    pub async fn update_graph(
        &self,
        id: &str,
        nodes: Vec<PipelineNode>,
        edges: Vec<PipelineEdge>,
    ) -> Result<Pipeline, String> {
        let mut pipelines = self.pipelines.write().await;
        let pipeline = pipelines
            .iter_mut()
            .find(|p| p.id == id)
            .ok_or_else(|| format!("Pipeline '{}' not found", id))?;
        pipeline.nodes = nodes;
        pipeline.edges = edges;
        Ok(pipeline.clone())
    }
}
