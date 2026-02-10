//! # Cross-Stream Query Engine
//!
//! Simple query DSL for searching and filtering events across all connected
//! data streams. Supports field comparisons, temporal ranges, and cross-stream
//! correlation by trace_id.

pub mod executor;
pub mod parser;

use serde::{Deserialize, Serialize};

/// A parsed query.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Query {
    /// Stream names to search (empty = all).
    pub from: Vec<String>,
    /// Filter conditions.
    pub conditions: Vec<Condition>,
    /// Temporal range.
    pub since: Option<String>,
    pub until: Option<String>,
    /// Result limit.
    pub limit: usize,
    /// Offset for pagination.
    pub offset: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Condition {
    pub field: String,
    pub op: CompareOp,
    pub value: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum CompareOp {
    Eq,
    Neq,
    Gt,
    Gte,
    Lt,
    Lte,
    Contains,
    StartsWith,
}

/// Query execution result.
#[derive(Debug, Clone, Serialize)]
pub struct QueryResult {
    pub events: Vec<crate::connectors::StreamEvent>,
    pub total: usize,
    pub query_time_ms: u64,
    pub streams_searched: Vec<String>,
}

/// Request body for executing a query.
#[derive(Debug, Clone, Deserialize)]
pub struct QueryRequest {
    /// Raw query text (parsed by the DSL parser).
    pub query: Option<String>,
    /// Structured query (alternative to raw text).
    pub structured: Option<Query>,
}
