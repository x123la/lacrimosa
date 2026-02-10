//! # Query Executor
//!
//! Evaluates parsed queries against the [`ConnectorRegistry`] event buffer.

use super::{CompareOp, Condition, Query, QueryResult};
use crate::connectors::registry::ConnectorRegistry;
use crate::connectors::StreamEvent;
use chrono::{DateTime, Duration, Utc};
use std::sync::Arc;
use std::time::Instant;

/// Execute a query against the connector registry's buffered events.
pub async fn execute(query: &Query, registry: &Arc<ConnectorRegistry>) -> QueryResult {
    let start = Instant::now();
    let now = Utc::now();

    let all_events = registry.buffered_events().await;

    // Filter by source streams
    let stream_filtered: Vec<&StreamEvent> = if query.from.is_empty() {
        all_events.iter().collect()
    } else {
        all_events
            .iter()
            .filter(|e| {
                query
                    .from
                    .iter()
                    .any(|f| e.stream.contains(f) || e.connector_id.contains(f))
            })
            .collect()
    };

    // Apply WHERE conditions
    let condition_filtered: Vec<&StreamEvent> = stream_filtered
        .into_iter()
        .filter(|e| evaluate_conditions(e, &query.conditions))
        .collect();

    // Apply temporal filters
    let since = query
        .since
        .as_deref()
        .and_then(|value| parse_time_expr(value, now));
    let until = query
        .until
        .as_deref()
        .and_then(|value| parse_time_expr(value, now));

    let temporal_filtered: Vec<&StreamEvent> = if since.is_some() || until.is_some() {
        condition_filtered
            .into_iter()
            .filter(|e| {
                let event_ts = parse_event_timestamp(e);
                match event_ts {
                    Some(ts) => {
                        let since_ok = match since {
                            Some(s) => ts >= s,
                            None => true,
                        };
                        let until_ok = match until {
                            Some(u) => ts <= u,
                            None => true,
                        };
                        since_ok && until_ok
                    }
                    None => false,
                }
            })
            .collect()
    } else {
        condition_filtered
    };

    let total = temporal_filtered.len();

    // Collect unique streams searched
    let streams_searched: Vec<String> = {
        let mut s: Vec<String> = temporal_filtered.iter().map(|e| e.stream.clone()).collect();
        s.sort();
        s.dedup();
        s
    };

    // Pagination
    let paginated: Vec<StreamEvent> = temporal_filtered
        .into_iter()
        .skip(query.offset)
        .take(query.limit)
        .cloned()
        .collect();

    QueryResult {
        events: paginated,
        total,
        query_time_ms: start.elapsed().as_millis() as u64,
        streams_searched,
    }
}

fn parse_event_timestamp(event: &StreamEvent) -> Option<DateTime<Utc>> {
    DateTime::parse_from_rfc3339(&event.timestamp)
        .ok()
        .map(|dt| dt.with_timezone(&Utc))
}

fn parse_time_expr(raw: &str, now: DateTime<Utc>) -> Option<DateTime<Utc>> {
    let value = raw.trim();
    if value.is_empty() {
        return None;
    }
    if let Ok(dt) = DateTime::parse_from_rfc3339(value) {
        return Some(dt.with_timezone(&Utc));
    }

    let (number, unit) = value.split_at(value.len().saturating_sub(1));
    let amount: i64 = number.parse().ok()?;
    let duration = match unit {
        "s" => Duration::seconds(amount),
        "m" => Duration::minutes(amount),
        "h" => Duration::hours(amount),
        "d" => Duration::days(amount),
        _ => return None,
    };
    Some(now - duration)
}

fn evaluate_conditions(event: &StreamEvent, conditions: &[Condition]) -> bool {
    conditions
        .iter()
        .all(|cond| evaluate_condition(event, cond))
}

fn evaluate_condition(event: &StreamEvent, cond: &Condition) -> bool {
    // Try to extract value from event payload or metadata
    let event_value = extract_field(event, &cond.field);

    match &event_value {
        Some(val) => compare(val, &cond.op, &cond.value),
        None => false,
    }
}

fn extract_field(event: &StreamEvent, field: &str) -> Option<serde_json::Value> {
    // Check top-level event fields
    match field {
        "id" => return Some(serde_json::Value::String(event.id.clone())),
        "connector_id" => return Some(serde_json::Value::String(event.connector_id.clone())),
        "stream" => return Some(serde_json::Value::String(event.stream.clone())),
        "sequence" => return Some(serde_json::json!(event.sequence)),
        "timestamp" => return Some(serde_json::Value::String(event.timestamp.clone())),
        _ => {}
    }

    // Check metadata
    if let Some(val) = event.metadata.get(field) {
        return Some(serde_json::Value::String(val.clone()));
    }

    // Check payload using JSON pointer syntax (e.g., "payload.amount" → "/amount")
    let field = field.strip_prefix("payload.").unwrap_or(field);
    let pointer = if field.starts_with('/') {
        field.to_string()
    } else {
        format!("/{}", field.replace('.', "/"))
    };

    event.payload.pointer(&pointer).cloned()
}

fn compare(a: &serde_json::Value, op: &CompareOp, b: &serde_json::Value) -> bool {
    match op {
        CompareOp::Eq => values_equal(a, b),
        CompareOp::Neq => !values_equal(a, b),
        CompareOp::Gt => numeric_cmp(a, b).map_or(false, |o| o == std::cmp::Ordering::Greater),
        CompareOp::Gte => numeric_cmp(a, b).map_or(false, |o| o != std::cmp::Ordering::Less),
        CompareOp::Lt => numeric_cmp(a, b).map_or(false, |o| o == std::cmp::Ordering::Less),
        CompareOp::Lte => numeric_cmp(a, b).map_or(false, |o| o != std::cmp::Ordering::Greater),
        CompareOp::Contains => {
            let a_str = value_to_string(a);
            let b_str = value_to_string(b);
            a_str.contains(&b_str)
        }
        CompareOp::StartsWith => {
            let a_str = value_to_string(a);
            let b_str = value_to_string(b);
            a_str.starts_with(&b_str)
        }
    }
}

fn values_equal(a: &serde_json::Value, b: &serde_json::Value) -> bool {
    // Try numeric comparison first
    if let (Some(an), Some(bn)) = (value_to_f64(a), value_to_f64(b)) {
        return (an - bn).abs() < f64::EPSILON;
    }
    // Fall back to string comparison
    value_to_string(a) == value_to_string(b)
}

fn numeric_cmp(a: &serde_json::Value, b: &serde_json::Value) -> Option<std::cmp::Ordering> {
    let an = value_to_f64(a)?;
    let bn = value_to_f64(b)?;
    an.partial_cmp(&bn)
}

fn value_to_f64(v: &serde_json::Value) -> Option<f64> {
    match v {
        serde_json::Value::Number(n) => n.as_f64(),
        serde_json::Value::String(s) => s.parse::<f64>().ok(),
        _ => None,
    }
}

fn value_to_string(v: &serde_json::Value) -> String {
    match v {
        serde_json::Value::String(s) => s.clone(),
        other => other.to_string(),
    }
}
