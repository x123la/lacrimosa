use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use tokio::sync::RwLock;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Span {
    pub trace_id: String,
    pub span_id: String,
    pub parent_span_id: Option<String>,
    pub name: String, // Operation name
    pub service_name: String,
    pub start_time_unix_nano: u64,
    pub end_time_unix_nano: u64,
    pub attributes: HashMap<String, serde_json::Value>,
    pub status: SpanStatus,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum SpanStatus {
    Unset,
    Ok,
    Error(String),
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Trace {
    pub trace_id: String,
    pub spans: Vec<Span>,
    pub root_span: Option<Span>,
    pub start_time: DateTime<Utc>,
    pub duration_ms: u64,
    pub services: HashSet<String>,
    pub error_count: usize,
}

pub struct TraceStore {
    traces: RwLock<HashMap<String, Trace>>,
    max_traces: usize,
}

#[derive(Deserialize)]
pub struct SpanIngestionRequest {
    pub spans: Vec<Span>,
}

#[derive(Deserialize)]
pub struct TraceSearchParams {
    pub service: Option<String>,
    pub operation: Option<String>,
    pub min_duration_ms: Option<u64>,
    pub limit: Option<usize>,
    pub since: Option<String>, // ISO8601
}

impl TraceStore {
    pub fn new(max_traces: usize) -> Self {
        Self {
            traces: RwLock::new(HashMap::new()),
            max_traces,
        }
    }

    pub async fn ingest(&self, spans: Vec<Span>) {
        let mut store = self.traces.write().await;

        for span in spans {
            let trace = store.entry(span.trace_id.clone()).or_insert_with(|| Trace {
                trace_id: span.trace_id.clone(),
                spans: Vec::new(),
                root_span: None,
                start_time: DateTime::from_timestamp_nanos(span.start_time_unix_nano as i64),
                duration_ms: 0,
                services: HashSet::new(),
                error_count: 0,
            });

            trace.spans.push(span);
            recompute_trace_summary(trace);
        }

        // Re-calculate durations for updated traces
        // Pruning logic would go here (LRU or random drop when > max_traces)
        if store.len() > self.max_traces {
            // Simple random prune for now
            if let Some(k) = store.keys().next().cloned() {
                store.remove(&k);
            }
        }
    }

    pub async fn get_trace(&self, trace_id: &str) -> Option<Trace> {
        self.traces.read().await.get(trace_id).cloned()
    }

    pub async fn search(&self, params: TraceSearchParams) -> Vec<Trace> {
        let store = self.traces.read().await;
        let since_filter = params.since.as_deref().and_then(parse_since);

        let mut results: Vec<Trace> = store
            .values()
            .filter(|t| {
                if let Some(svc) = &params.service {
                    if !t.services.contains(svc) {
                        return false;
                    }
                }

                if let Some(op) = &params.operation {
                    let op_lc = op.to_lowercase();
                    let matches_operation = t
                        .spans
                        .iter()
                        .any(|s| s.name.to_lowercase().contains(&op_lc));
                    if !matches_operation {
                        return false;
                    }
                }

                if let Some(since) = since_filter {
                    if t.start_time < since {
                        return false;
                    }
                }

                if let Some(min_dur) = params.min_duration_ms {
                    if t.duration_ms < min_dur {
                        return false;
                    }
                }

                true
            })
            .cloned()
            .collect();

        // Sort by time desc
        results.sort_by(|a, b| b.start_time.cmp(&a.start_time));

        results
            .into_iter()
            .take(params.limit.unwrap_or(50))
            .collect()
    }

    pub async fn get_service_graph(&self) -> Vec<ServiceDependency> {
        let store = self.traces.read().await;
        let mut edges = HashMap::<(String, String), usize>::new();

        for trace in store.values() {
            // Map span_id -> Span for quick lookup
            let span_map: HashMap<&String, &Span> =
                trace.spans.iter().map(|s| (&s.span_id, s)).collect();

            for span in &trace.spans {
                if let Some(parent_id) = &span.parent_span_id {
                    if let Some(parent) = span_map.get(parent_id) {
                        if parent.service_name != span.service_name {
                            *edges
                                .entry((parent.service_name.clone(), span.service_name.clone()))
                                .or_insert(0) += 1;
                        }
                    }
                }
            }
        }

        edges
            .into_iter()
            .map(|((from, to), count)| ServiceDependency { from, to, count })
            .collect()
    }
}

#[derive(Serialize)]
pub struct ServiceDependency {
    pub from: String,
    pub to: String,
    pub count: usize,
}

fn recompute_trace_summary(trace: &mut Trace) {
    if trace.spans.is_empty() {
        trace.duration_ms = 0;
        trace.error_count = 0;
        trace.services.clear();
        trace.root_span = None;
        return;
    }

    let mut min_start = u64::MAX;
    let mut max_end = 0u64;
    let mut services = HashSet::new();
    let mut error_count = 0usize;
    let mut root_span: Option<Span> = None;

    for span in &trace.spans {
        min_start = min_start.min(span.start_time_unix_nano);
        max_end = max_end.max(span.end_time_unix_nano);
        services.insert(span.service_name.clone());
        if matches!(span.status, SpanStatus::Error(_)) {
            error_count += 1;
        }
        if span.parent_span_id.is_none() && root_span.is_none() {
            root_span = Some(span.clone());
        }
    }

    trace.start_time = DateTime::from_timestamp_nanos(min_start as i64);
    trace.duration_ms = max_end.saturating_sub(min_start) / 1_000_000;
    trace.services = services;
    trace.error_count = error_count;
    trace.root_span = root_span.or_else(|| trace.spans.first().cloned());
}

fn parse_since(value: &str) -> Option<DateTime<Utc>> {
    DateTime::parse_from_rfc3339(value)
        .ok()
        .map(|dt| dt.with_timezone(&Utc))
}
