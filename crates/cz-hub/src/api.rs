//! # API Handlers
//!
//! Axum handlers for the new Control Center capabilities.

use crate::alerts::{AlertRuleV2, Incident};
use crate::auth::CreateApiKeyRequest;
use crate::connectors::{ConnectorConfig, ConnectorInfo};
use crate::dashboards::{CreateDashboardRequest, Dashboard, UpdateDashboardRequest};
use crate::pipelines::{CreatePipelineRequest, Pipeline, UpdatePipelineRequest};
use crate::query::{QueryRequest, QueryResult};
use crate::traces::{ServiceDependency, SpanIngestionRequest, Trace, TraceSearchParams};
use crate::AppState;
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use std::collections::HashMap;
use std::sync::Arc;

// =============================================================================
// Connectors
// =============================================================================

pub async fn list_connectors(State(state): State<Arc<AppState>>) -> Json<Vec<ConnectorInfo>> {
    let connectors = state.connector_registry.list().await;
    Json(connectors)
}

pub async fn create_connector(
    State(state): State<Arc<AppState>>,
    Json(config): Json<ConnectorConfig>,
) -> Result<Json<ConnectorInfo>, (StatusCode, String)> {
    match state.connector_registry.create_from_config(config).await {
        Ok(info) => Ok(Json(info)),
        Err(e) => Err((StatusCode::BAD_REQUEST, e.to_string())),
    }
}

pub async fn delete_connector(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Result<StatusCode, (StatusCode, String)> {
    match state.connector_registry.remove(&id).await {
        Ok(_) => Ok(StatusCode::NO_CONTENT),
        Err(e) => Err((StatusCode::NOT_FOUND, e.to_string())),
    }
}

pub async fn ingest_webhook(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
    headers: axum::http::HeaderMap,
    Json(payload): Json<serde_json::Value>,
) -> Result<StatusCode, (StatusCode, String)> {
    let connector = state
        .connector_registry
        .get(&id)
        .await
        .ok_or((StatusCode::NOT_FOUND, "Connector not found".to_string()))?;

    let normalized_headers: HashMap<String, String> = headers
        .iter()
        .filter_map(|(k, v)| {
            v.to_str()
                .ok()
                .map(|value| (k.as_str().to_string(), value.to_string()))
        })
        .collect();

    connector
        .ingest(payload, normalized_headers)
        .await
        .map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))?;

    Ok(StatusCode::ACCEPTED)
}

// =============================================================================
// Query
// =============================================================================

pub async fn execute_query(
    State(state): State<Arc<AppState>>,
    Json(req): Json<QueryRequest>,
) -> Result<Json<QueryResult>, (StatusCode, String)> {
    let query = if let Some(q) = req.structured {
        q
    } else if let Some(text) = &req.query {
        crate::query::parser::parse(text).map_err(|e| (StatusCode::BAD_REQUEST, e))?
    } else {
        return Err((StatusCode::BAD_REQUEST, "Missing query".into()));
    };

    let result = crate::query::executor::execute(&query, &state.connector_registry).await;
    Ok(Json(result))
}

// =============================================================================
// Alerts
// =============================================================================

pub async fn list_incidents(State(state): State<Arc<AppState>>) -> Json<Vec<Incident>> {
    let incidents = state.alert_engine.list_active().await;
    Json(incidents)
}

pub async fn acknowledge_incident(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Result<Json<Incident>, (StatusCode, String)> {
    let incident = state
        .alert_engine
        .acknowledge_incident(&id, "admin")
        .await // hardcoded actor for now
        .map_err(|e| (StatusCode::NOT_FOUND, e))?;
    Ok(Json(incident))
}

pub async fn resolve_incident(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Result<Json<Incident>, (StatusCode, String)> {
    let incident = state
        .alert_engine
        .resolve_incident(&id, "admin")
        .await
        .map_err(|e| (StatusCode::NOT_FOUND, e))?;
    Ok(Json(incident))
}

pub async fn create_test_incident(State(state): State<Arc<AppState>>) -> Json<Incident> {
    let rule = crate::alerts::AlertRuleV2 {
        id: "rule-test".into(),
        name: "Test Alert Rule".into(),
        rule_type: crate::alerts::RuleType::Threshold,
        stream: Some("test-stream".into()),
        field: "cpu".into(),
        threshold: 90.0,
        duration_seconds: 60,
        severity: "critical".into(),
        enabled: true,
        notification_channels: vec![],
        runbook_url: None,
    };
    let incident = state
        .alert_engine
        .create_incident(&rule, "Test incident triggered manually".into())
        .await;
    Json(incident)
}

pub async fn create_alert_rule(
    State(state): State<Arc<AppState>>,
    Json(rule): Json<AlertRuleV2>,
) -> Json<String> {
    let mut rules = state.alert_engine.rules.write().await;
    rules.push(rule);
    Json("created".into())
}

// =============================================================================
// Traces
// =============================================================================

pub async fn list_traces(
    State(state): State<Arc<AppState>>,
    Query(params): Query<TraceSearchParams>,
) -> Json<Vec<Trace>> {
    let traces = state.trace_store.search(params).await;
    Json(traces)
}

pub async fn get_trace(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Json<Option<Trace>> {
    let trace = state.trace_store.get_trace(&id).await;
    Json(trace)
}

pub async fn get_service_graph(State(state): State<Arc<AppState>>) -> Json<Vec<ServiceDependency>> {
    let graph = state.trace_store.get_service_graph().await;
    Json(graph)
}

pub async fn ingest_spans(
    State(state): State<Arc<AppState>>,
    Json(body): Json<SpanIngestionRequest>,
) -> StatusCode {
    state.trace_store.ingest(body.spans).await;
    StatusCode::ACCEPTED
}

// =============================================================================
// Pipelines
// =============================================================================

pub async fn list_pipelines(State(state): State<Arc<AppState>>) -> Json<Vec<Pipeline>> {
    let pipelines = state.pipeline_manager.list().await;
    Json(pipelines)
}

pub async fn create_pipeline(
    State(state): State<Arc<AppState>>,
    Json(req): Json<CreatePipelineRequest>,
) -> Json<Pipeline> {
    let pipeline = state.pipeline_manager.create(req).await;
    Json(pipeline)
}

pub async fn get_pipeline(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Json<Option<Pipeline>> {
    let pipeline = state.pipeline_manager.get(&id).await;
    Json(pipeline)
}

pub async fn update_pipeline(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
    Json(req): Json<UpdatePipelineRequest>,
) -> Result<Json<Pipeline>, (StatusCode, String)> {
    let pipeline = state
        .pipeline_manager
        .update_graph(&id, req.nodes, req.edges)
        .await
        .map_err(|e| (StatusCode::NOT_FOUND, e))?;
    Ok(Json(pipeline))
}

pub async fn delete_pipeline(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Result<StatusCode, (StatusCode, String)> {
    state
        .pipeline_manager
        .delete(&id)
        .await
        .map_err(|e| (StatusCode::NOT_FOUND, e))?;
    Ok(StatusCode::NO_CONTENT)
}

pub async fn run_pipeline(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Result<Json<Pipeline>, (StatusCode, String)> {
    let pipeline = state
        .pipeline_manager
        .set_status(&id, crate::pipelines::PipelineStatus::Running)
        .await
        .map_err(|e| (StatusCode::NOT_FOUND, e))?;
    Ok(Json(pipeline))
}

pub async fn stop_pipeline(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Result<Json<Pipeline>, (StatusCode, String)> {
    let pipeline = state
        .pipeline_manager
        .set_status(&id, crate::pipelines::PipelineStatus::Stopped)
        .await
        .map_err(|e| (StatusCode::NOT_FOUND, e))?;
    Ok(Json(pipeline))
}

// =============================================================================
// Dashboards
// =============================================================================

pub async fn list_dashboards(State(state): State<Arc<AppState>>) -> Json<Vec<Dashboard>> {
    let dashboards = state.dashboard_manager.list().await;
    Json(dashboards)
}

pub async fn create_dashboard(
    State(state): State<Arc<AppState>>,
    Json(req): Json<CreateDashboardRequest>,
) -> Json<Dashboard> {
    let dashboard = state
        .dashboard_manager
        .create(req.name, req.description)
        .await;
    Json(dashboard)
}

pub async fn get_dashboard(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Json<Option<Dashboard>> {
    let dashboard = state.dashboard_manager.get(&id).await;
    Json(dashboard)
}

pub async fn update_dashboard(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
    Json(req): Json<UpdateDashboardRequest>,
) -> Result<Json<Dashboard>, (StatusCode, String)> {
    let dashboard = state
        .dashboard_manager
        .update(&id, req.layout, req.widgets)
        .await
        .map_err(|e| (StatusCode::NOT_FOUND, e))?;
    Ok(Json(dashboard))
}

pub async fn delete_dashboard(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Result<StatusCode, (StatusCode, String)> {
    state
        .dashboard_manager
        .delete(&id)
        .await
        .map_err(|e| (StatusCode::NOT_FOUND, e))?;
    Ok(StatusCode::NO_CONTENT)
}

// =============================================================================
// Auth
// =============================================================================

pub async fn create_api_key(
    State(state): State<Arc<AppState>>,
    Json(req): Json<CreateApiKeyRequest>,
) -> Json<crate::auth::ApiKey> {
    let key = state.auth_layer.create_key(req).await;
    Json(key)
}

pub async fn list_api_keys(State(state): State<Arc<AppState>>) -> Json<Vec<crate::auth::ApiKey>> {
    let keys = state.auth_layer.list_keys().await;
    Json(keys)
}

pub async fn revoke_api_key(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Result<StatusCode, (StatusCode, String)> {
    state
        .auth_layer
        .revoke_key(&id)
        .await
        .map_err(|e| (StatusCode::NOT_FOUND, e))?;
    Ok(StatusCode::NO_CONTENT)
}

pub async fn get_audit_log(
    State(state): State<Arc<AppState>>,
) -> Json<Vec<crate::auth::AuditEntry>> {
    let log = state.auth_layer.get_audit_log(100).await;
    Json(log)
}
