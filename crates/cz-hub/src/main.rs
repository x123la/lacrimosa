use axum::{
    extract::Request,
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        Query, State,
    },
    http::{header, Method, StatusCode},
    middleware::{self, Next},
    response::{IntoResponse, Response},
    routing::{get, post},
    Json, Router,
};
use clap::Parser;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::atomic::Ordering;
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::RwLock;
use tower_http::cors::CorsLayer;
use tower_http::services::ServeDir;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use cz_core::CausalEvent;
use cz_io::cursor::Cursor;
use cz_io::journal::{Journal, INDEX_RING_CAPACITY, INDEX_RING_SIZE};

mod alerts;
mod api;
mod auth;
mod connectors;
mod dashboards;
mod pipelines;
mod query;
mod traces;

// =============================================================================
// CLI
// =============================================================================

#[derive(Parser)]
#[command(
    name = "cz-hub",
    version = "0.3.0",
    about = "LACRIMOSA Control Center"
)]
struct Args {
    /// Path to the journal file(s)
    #[arg(long, default_value = "journal.db")]
    journals: Vec<PathBuf>,

    /// Journal size in bytes (only used if creating a new journal)
    #[arg(long, default_value_t = 2 * 1024 * 1024)]
    journal_size: u64,

    /// Server bind address
    #[arg(long, default_value = "127.0.0.1:3000")]
    bind: String,

    /// Path to config file
    #[arg(long, default_value = "cz-hub.toml")]
    config: PathBuf,
}

// =============================================================================
// Config
// =============================================================================

#[derive(Deserialize, Default, Clone)]
struct Config {
    #[serde(default)]
    alerts: AlertConfig,
    #[serde(default)]
    server: ServerConfig,
}

#[derive(Deserialize, Clone)]
struct AlertConfig {
    #[serde(default = "default_ring_threshold")]
    ring_utilization_warn: f64,
    #[serde(default = "default_ring_critical")]
    ring_utilization_critical: f64,
    #[serde(default = "default_tps_drop")]
    tps_drop_threshold: f64,
    #[serde(default = "default_idle_timeout")]
    #[allow(dead_code)]
    idle_timeout_secs: u64,
}

impl Default for AlertConfig {
    fn default() -> Self {
        Self {
            ring_utilization_warn: 70.0,
            ring_utilization_critical: 90.0,
            tps_drop_threshold: 50.0,
            idle_timeout_secs: 30,
        }
    }
}

#[derive(Deserialize, Clone)]
struct ServerConfig {
    #[serde(default = "default_metrics_interval")]
    metrics_interval_ms: u64,
    #[serde(default = "default_history_capacity")]
    history_capacity: usize,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            metrics_interval_ms: 200,
            history_capacity: 3600,
        }
    }
}

fn default_ring_threshold() -> f64 {
    70.0
}
fn default_ring_critical() -> f64 {
    90.0
}
fn default_tps_drop() -> f64 {
    50.0
}
fn default_idle_timeout() -> u64 {
    30
}
fn default_metrics_interval() -> u64 {
    200
}
fn default_history_capacity() -> usize {
    3600
}

// =============================================================================
// Application State
// =============================================================================

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
#[serde(rename_all = "snake_case")]
enum PlaybackMode {
    #[default]
    RealTime,
    Paused {
        at_slot: usize,
        at_ts: u64,
    },
}

struct AppState {
    journals: RwLock<HashMap<PathBuf, Arc<JournalState>>>,
    playback: RwLock<PlaybackMode>,
    start_time: Instant,
    config: Config,
    metrics_history: RwLock<VecDeque<MetricsSnapshot>>,

    // Legacy fields (will migrate to new modules)
    alerts: RwLock<Vec<Alert>>,
    alert_rules: RwLock<Vec<AlertRule>>,

    // New Capability Modules
    connector_registry: Arc<connectors::registry::ConnectorRegistry>,
    alert_engine: Arc<alerts::AlertEngine>,
    trace_store: Arc<traces::TraceStore>,
    pipeline_manager: Arc<pipelines::PipelineManager>,
    dashboard_manager: Arc<dashboards::DashboardManager>,
    auth_layer: Arc<auth::AuthLayer>,
}

#[derive(Deserialize)]
struct PlaybackSetParams {
    mode: String, // "real_time" or "paused"
    slot: Option<usize>,
    ts: Option<u64>,
}

impl AppState {
    async fn get_journal(&self, path: Option<String>) -> Option<Arc<JournalState>> {
        let journals = self.journals.read().await;
        if let Some(p) = path {
            journals.get(&PathBuf::from(p)).cloned()
        } else {
            journals.values().next().cloned()
        }
    }
}

struct JournalState {
    path: PathBuf,
    journal: RwLock<Journal>,
    cursor: RwLock<Cursor>,
}

// =============================================================================
// Types
// =============================================================================

#[derive(Serialize, Clone)]
struct MetricsSnapshot {
    timestamp: String,
    events: u64,
    bytes: u64,
    tps: f64,
    bps: f64,
    head: usize,
    tail: usize,
    utilization_pct: f64,
    uptime_seconds: u64,
    playback_mode: PlaybackMode,
}
#[derive(Serialize, Clone)]
struct Alert {
    id: u64,
    severity: String, // "warn", "critical", "info"
    message: String,
    timestamp: String,
    rule_name: String,
    resolved: bool,
}

#[derive(Serialize, Deserialize, Clone)]
struct AlertRule {
    name: String,
    condition: String, // "ring_utilization_gt", "tps_drop_gt", "idle_timeout"
    threshold: f64,
    severity: String,
    enabled: bool,
}

#[derive(Serialize)]
struct SystemStatus {
    version: &'static str,
    engine: &'static str,
    zero_copy: bool,
    uptime_seconds: u64,
    event_size_bytes: usize,
    journal_path: String,
    journal_size_bytes: u64,
    index_ring_capacity: usize,
    index_ring_size_bytes: usize,
    events_processed: u64,
    bytes_processed: u64,
    current_tps: f64,
    current_bps: f64,
}

#[derive(Serialize)]
struct RingState {
    head: usize,
    tail: usize,
    capacity: usize,
    used: usize,
    utilization_pct: f64,
    is_full: bool,
    is_empty: bool,
    bytes_per_slot: usize,
    total_bytes: usize,
}

#[derive(Serialize)]
struct EventRecord {
    slot: usize,
    lamport_ts: u64,
    node_id: u32,
    stream_id: u16,
    payload_offset: u64,
    checksum: u32,
    checkpoint: bool,
}

#[derive(Serialize)]
struct EventDetailRecord {
    #[serde(flatten)]
    event: EventRecord,
    payload_hex: String,
    payload_ascii: String,
    payload_size: usize,
}

#[derive(Serialize)]
struct EventListResponse {
    events: Vec<EventRecord>,
    total: usize,
    offset: usize,
    limit: usize,
}

#[derive(Deserialize)]
struct EventQueryParams {
    journal: Option<String>,
    node_id: Option<u32>,
    stream_id: Option<u16>,
    ts_min: Option<u64>,
    ts_max: Option<u64>,
    offset: Option<usize>,
    limit: Option<usize>,
    query: Option<String>, // e.g. "node_id == 1 && stream_id > 0"
}

#[derive(Deserialize)]
struct ExportParams {
    format: Option<String>,
    journal: Option<String>,
    limit: Option<usize>,
}

#[derive(Serialize)]
struct VerifyResult {
    success: bool,
    output: String,
    duration_ms: u64,
    timestamp: String,
}

#[derive(Deserialize)]
struct SimulateParams {
    journal: Option<String>,
    count: Option<usize>,
    node_id: Option<u32>,
    stream_id: Option<u16>,
}

#[derive(Serialize)]
struct SimulateResult {
    events_created: usize,
    head_after: usize,
}

#[derive(Deserialize)]
struct ReplayParams {
    journal: Option<String>,
    start_slot: usize,
    end_slot: usize,
    target_journal: Option<String>,
}

#[derive(Serialize)]
struct ReplayResult {
    events_replayed: usize,
    new_head: usize,
}

#[derive(Serialize)]
struct TopologyNode {
    node_id: u32,
    event_count: usize,
    streams: Vec<u16>,
    first_seen_ts: u64,
    last_seen_ts: u64,
}

#[derive(Serialize)]
struct TopologyResponse {
    nodes: Vec<TopologyNode>,
    total_nodes: usize,
    total_streams: usize,
    total_events: usize,
}

#[derive(Serialize)]
struct StreamStat {
    stream_id: u16,
    event_count: usize,
    nodes: Vec<u32>,
    min_ts: u64,
    max_ts: u64,
}

#[derive(Serialize)]
struct StreamsResponse {
    streams: Vec<StreamStat>,
    total_streams: usize,
}

#[derive(Serialize)]
struct JournalLayout {
    total_size_bytes: u64,
    index_ring_start: usize,
    index_ring_end: usize,
    index_ring_size_bytes: usize,
    index_ring_slot_count: usize,
    index_ring_slot_size: usize,
    blob_storage_start: usize,
    blob_storage_end: u64,
    blob_storage_size_bytes: u64,
    slots_used: usize,
    slots_free: usize,
}

#[derive(Serialize)]
struct SystemResources {
    pid: u32,
    memory_rss_kb: u64,
    memory_vms_kb: u64,
    threads: u64,
    uptime_seconds: u64,
}

#[derive(Serialize)]
struct MetricsMessage {
    r#type: &'static str,
    data: MetricsSnapshot,
}

#[derive(Serialize)]
struct ApiError {
    error: String,
}

// =============================================================================
// Main
// =============================================================================

#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "cz_hub=info,tower_http=info".into()),
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();

    let args = Args::parse();

    // Load config
    let config = if args.config.exists() {
        let content = std::fs::read_to_string(&args.config).unwrap_or_default();
        toml::from_str(&content).unwrap_or_default()
    } else {
        Config::default()
    };

    let mut journals = HashMap::new();
    for path in &args.journals {
        let journal = match Journal::open(path, args.journal_size) {
            Ok(j) => j,
            Err(e) => {
                tracing::error!("Failed to open journal at {:?}: {}", path, e);
                continue;
            }
        };
        let cursor = Cursor::for_index_ring();
        journals.insert(
            path.clone(),
            Arc::new(JournalState {
                path: path.clone(),
                journal: RwLock::new(journal),
                cursor: RwLock::new(cursor),
            }),
        );
    }

    if journals.is_empty() {
        tracing::error!("No journals opened. Exiting.");
        std::process::exit(1);
    }

    // Initial alert rules container

    // Default alert rules
    let default_rules = vec![
        AlertRule {
            name: "Ring Utilization Warning".into(),
            condition: "ring_utilization_gt".into(),
            threshold: config.alerts.ring_utilization_warn,
            severity: "warn".into(),
            enabled: true,
        },
        AlertRule {
            name: "Ring Utilization Critical".into(),
            condition: "ring_utilization_gt".into(),
            threshold: config.alerts.ring_utilization_critical,
            severity: "critical".into(),
            enabled: true,
        },
        AlertRule {
            name: "TPS Drop".into(),
            condition: "tps_drop_gt".into(),
            threshold: config.alerts.tps_drop_threshold,
            severity: "warn".into(),
            enabled: true,
        },
    ];

    let connector_registry = Arc::new(connectors::registry::ConnectorRegistry::new(1000));
    let alert_engine = Arc::new(alerts::AlertEngine::new(100));
    let trace_store = Arc::new(traces::TraceStore::new(1000));
    let pipeline_manager = Arc::new(pipelines::PipelineManager::new());
    let dashboard_manager = Arc::new(dashboards::DashboardManager::new());
    let auth_layer = Arc::new(auth::AuthLayer::new(1000));

    // Register internal journals as connectors
    for (path, _j_state) in &journals {
        let connector = Arc::new(connectors::journal::JournalConnector::new(path.clone()));
        // Note: In a real implementation we'd probably want to share the journal access better,
        // but for now the connector manages its own view or we'd refactor JournalState to use it.
        // For MVP, we just register it so it shows up in the UI.
        connector_registry.add(connector).await.ok();
    }

    let state = Arc::new(AppState {
        journals: RwLock::new(journals),
        playback: RwLock::new(PlaybackMode::default()),
        start_time: Instant::now(),
        config: config.clone(),
        metrics_history: RwLock::new(VecDeque::with_capacity(config.server.history_capacity)),
        alerts: RwLock::new(Vec::new()),
        alert_rules: RwLock::new(default_rules),
        connector_registry,
        alert_engine,
        trace_store,
        pipeline_manager,
        dashboard_manager,
        auth_layer,
    });

    // Spawn background metrics collector
    let bg_state = state.clone();
    tokio::spawn(async move { metrics_collector(bg_state).await });

    // Spawn IPC listener
    let ipc_state = state.clone();
    tokio::spawn(async move { ipc_listener(ipc_state).await });

    // Generate Root API Key on startup
    {
        let root_key = state
            .auth_layer
            .create_key(crate::auth::CreateApiKeyRequest {
                label: "Root Key (Startup)".into(),
                scopes: vec![
                    crate::auth::Scope::Admin,
                    crate::auth::Scope::Read,
                    crate::auth::Scope::Write,
                ],
            })
            .await;

        tracing::info!(
            "🔑 GENERATED ROOT API KEY: {}",
            root_key.key.as_ref().unwrap()
        );
        tracing::warn!("⚠️  Copy this key! It will not be shown again.");
    }

    let dist_path = PathBuf::from("crates/cz-hub/ui/dist");

    let app = Router::new()
        // Core APIs
        .route("/api/status", get(api_status))
        .route("/api/ring", get(api_ring))
        .route("/api/events", get(api_events))
        .route("/api/events/{slot}", get(api_event_detail))
        .route("/api/verify", post(api_verify))
        // New APIs
        .route("/api/simulate", post(api_simulate))
        .route("/api/topology", get(api_topology))
        .route("/api/streams", get(api_streams))
        .route("/api/journal/layout", get(api_journal_layout))
        .route("/api/system", get(api_system))
        .route("/api/metrics/history", get(api_metrics_history))
        .route("/api/alerts", get(api_alerts_get))
        .route("/api/alerts/rules", get(api_alert_rules_get))
        .route("/api/alerts/rules", post(api_alert_rules_set))
        .route("/api/export", get(api_export))
        .route("/metrics", get(api_metrics_prometheus))
        .route("/api/playback", get(api_playback_get))
        .route("/api/playback", post(api_playback_set))
        // New Capability APIs
        .route(
            "/api/connectors",
            get(api::list_connectors).post(api::create_connector),
        )
        .route(
            "/api/connectors/:id",
            axum::routing::delete(api::delete_connector),
        )
        .route("/api/connectors/:id/ingest", post(api::ingest_webhook))
        .route("/api/query", post(api::execute_query))
        .route("/api/alerts/incidents", get(api::list_incidents))
        .route(
            "/api/alerts/incidents/test",
            post(api::create_test_incident),
        )
        .route(
            "/api/alerts/incidents/:id/acknowledge",
            post(api::acknowledge_incident),
        )
        .route(
            "/api/alerts/incidents/:id/resolve",
            post(api::resolve_incident),
        )
        .route("/api/alerts/rules/v2", post(api::create_alert_rule))
        .route("/api/traces", get(api::list_traces))
        .route("/api/traces/ingest", post(api::ingest_spans))
        .route("/api/traces/:id", get(api::get_trace))
        .route("/api/traces/service-graph", get(api::get_service_graph))
        .route(
            "/api/pipelines",
            get(api::list_pipelines).post(api::create_pipeline),
        )
        .route(
            "/api/pipelines/:id",
            get(api::get_pipeline)
                .put(api::update_pipeline)
                .delete(api::delete_pipeline),
        )
        .route("/api/pipelines/:id/run", post(api::run_pipeline))
        .route("/api/pipelines/:id/stop", post(api::stop_pipeline))
        .route(
            "/api/dashboards",
            get(api::list_dashboards).post(api::create_dashboard),
        )
        .route(
            "/api/dashboards/:id",
            get(api::get_dashboard)
                .put(api::update_dashboard)
                .delete(api::delete_dashboard),
        )
        .route(
            "/api/auth/keys",
            post(api::create_api_key).get(api::list_api_keys),
        )
        .route(
            "/api/auth/keys/:id",
            axum::routing::delete(api::revoke_api_key),
        )
        .route("/api/auth/audit", get(api::get_audit_log))
        .route("/api/replay", post(api_replay))
        // Apply Auth Middleware to all API routes defined above
        // Note: middleware applies to routes added BEFORE it if using .layer() on the router?
        // No, .layer() wraps the *entire* router.
        // To apply only to some, we should use .route_layer() or nested routers.
        // For MVP simplicity: we apply to everything and let the middleware filter.
        .layer(middleware::from_fn_with_state(
            state.clone(),
            auth_middleware,
        ))
        // WebSocket
        .route("/ws", get(ws_handler))
        // Static UI
        .fallback_service(ServeDir::new(dist_path))
        .layer(CorsLayer::permissive())
        .with_state(state);

    let addr: SocketAddr = args.bind.parse().expect("Invalid bind address");
    tracing::info!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    tracing::info!("  🧬 LACRIMOSA v0.3.0");
    tracing::info!("  Dashboard:  http://{}", addr);
    tracing::info!("  API:        http://{}/api/status", addr);
    tracing::info!("  WebSocket:  ws://{}/ws", addr);
    tracing::info!("  Journal:    {:?}", args.journals);
    tracing::info!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

// =============================================================================
// Background Metrics Collector (1-second snapshots → history ring)
// =============================================================================

async fn metrics_collector(state: Arc<AppState>) {
    let mut interval = tokio::time::interval(std::time::Duration::from_secs(1));
    let mut prev_events: u64 = 0;
    let mut prev_bytes: u64 = 0;
    let mut prev_tps: f64 = 0.0;
    let mut alert_counter: u64 = 0;

    loop {
        interval.tick().await;

        let events = cz_io::event_loop::EVENTS_PROCESSED.load(Ordering::Relaxed);
        let bytes = cz_io::event_loop::BYTES_PROCESSED.load(Ordering::Relaxed);

        let tps = (events.saturating_sub(prev_events)) as f64;
        let bps = (bytes.saturating_sub(prev_bytes)) as f64;

        // For now, metrics are aggregated or based on the primary (first) journal
        let journals = state.journals.read().await;
        let primary = journals.values().next().unwrap();

        let cursor = primary.cursor.read().await;
        let used = cursor.len();
        let utilization = if INDEX_RING_CAPACITY > 0 {
            (used as f64 / INDEX_RING_CAPACITY as f64) * 100.0
        } else {
            0.0
        };

        let snapshot = MetricsSnapshot {
            timestamp: chrono::Utc::now().to_rfc3339(),
            events,
            bytes,
            tps,
            bps,
            head: cursor.head(),
            tail: cursor.tail(),
            utilization_pct: (used as f64 / INDEX_RING_CAPACITY as f64) * 100.0,
            uptime_seconds: state.start_time.elapsed().as_secs(),
            playback_mode: state.playback.read().await.clone(),
        };

        // Store in history
        {
            let mut history = state.metrics_history.write().await;
            if history.len() >= state.config.server.history_capacity {
                history.pop_front();
            }
            history.push_back(snapshot.clone());
        }

        // Check alert rules
        {
            let rules = state.alert_rules.read().await;
            let mut alerts = state.alerts.write().await;

            for rule in rules.iter().filter(|r| r.enabled) {
                let triggered = match rule.condition.as_str() {
                    "ring_utilization_gt" => utilization > rule.threshold,
                    "tps_drop_gt" => {
                        prev_tps > 0.0 && tps < prev_tps * (1.0 - rule.threshold / 100.0)
                    }
                    _ => false,
                };

                if triggered {
                    alert_counter += 1;
                    alerts.push(Alert {
                        id: alert_counter,
                        severity: rule.severity.clone(),
                        message: format!("{}: threshold {:.1} exceeded", rule.name, rule.threshold),
                        timestamp: chrono::Utc::now().to_rfc3339(),
                        rule_name: rule.name.clone(),
                        resolved: false,
                    });
                    // Keep last 100 alerts
                    let alen = alerts.len();
                    if alen > 100 {
                        alerts.drain(0..alen - 100);
                    }
                }
            }
        }

        prev_events = events;
        prev_bytes = bytes;
        prev_tps = tps;
    }
}

async fn ipc_listener(_state: Arc<AppState>) {
    loop {
        if let Ok(mut stream) = tokio::net::UnixStream::connect("/tmp/cz-io.sock").await {
            tracing::info!("Connected to cz-io real-time push socket");
            let mut buf = [0u8; 4];
            while let Ok(_) = tokio::io::AsyncReadExt::read_exact(&mut stream, &mut buf).await {
                let _slot = u32::from_le_bytes(buf) as usize;
                // V3: Push refresh signal to all WS clients
            }
        }
        tokio::time::sleep(std::time::Duration::from_secs(5)).await;
    }
}

// =============================================================================
// Core API Handlers
// =============================================================================

async fn api_status(State(state): State<Arc<AppState>>) -> Json<SystemStatus> {
    let uptime = state.start_time.elapsed().as_secs();
    let events = cz_io::event_loop::EVENTS_PROCESSED.load(Ordering::Relaxed);
    let bytes = cz_io::event_loop::BYTES_PROCESSED.load(Ordering::Relaxed);

    let (tps, bps) = {
        let history = state.metrics_history.read().await;
        history.back().map(|s| (s.tps, s.bps)).unwrap_or((0.0, 0.0))
    };

    let primary = state.get_journal(None).await.unwrap();
    let journal = primary.journal.read().await;

    Json(SystemStatus {
        version: "0.3.0",
        engine: "io_uring (pipelined, 16-deep)",
        zero_copy: true,
        uptime_seconds: uptime,
        event_size_bytes: CausalEvent::size_bytes(),
        journal_path: primary.path.display().to_string(),
        journal_size_bytes: journal.size(),
        index_ring_capacity: INDEX_RING_CAPACITY,
        index_ring_size_bytes: INDEX_RING_SIZE,
        events_processed: events,
        bytes_processed: bytes,
        current_tps: tps,
        current_bps: bps,
    })
}

async fn api_ring(
    State(state): State<Arc<AppState>>,
    Query(params): Query<HashMap<String, String>>,
) -> Result<Json<RingState>, (StatusCode, Json<ApiError>)> {
    let journal_path = params.get("journal");
    let primary = state.get_journal(journal_path.cloned()).await.ok_or((
        StatusCode::NOT_FOUND,
        Json(ApiError {
            error: "Journal not found".into(),
        }),
    ))?;

    let _journal = primary.journal.read().await;
    let cursor = primary.cursor.read().await;

    let used = cursor.len();
    let utilization = if INDEX_RING_CAPACITY > 0 {
        (used as f64 / INDEX_RING_CAPACITY as f64) * 100.0
    } else {
        0.0
    };

    Ok(Json(RingState {
        head: cursor.head(),
        tail: cursor.tail(),
        capacity: INDEX_RING_CAPACITY,
        used,
        utilization_pct: (utilization * 100.0).round() / 100.0,
        is_full: cursor.is_full(),
        is_empty: cursor.is_empty(),
        bytes_per_slot: CausalEvent::size_bytes(),
        total_bytes: INDEX_RING_SIZE,
    }))
}

async fn api_events(
    State(state): State<Arc<AppState>>,
    Query(params): Query<EventQueryParams>,
) -> Result<Json<EventListResponse>, (StatusCode, Json<ApiError>)> {
    let offset = params.offset.unwrap_or(0);
    let limit = params.limit.unwrap_or(50).min(500);

    let journal_path = params.journal.clone();
    let primary = state.get_journal(journal_path).await.ok_or((
        StatusCode::NOT_FOUND,
        Json(ApiError {
            error: "Journal not found".into(),
        }),
    ))?;

    let journal = primary.journal.read().await;
    let cursor = primary.cursor.read().await;
    let total = cursor.len();

    let mut records = Vec::with_capacity(limit);
    let mut skipped = 0;

    for i in 0..total {
        if records.len() >= limit {
            break;
        }

        let slot = (cursor.tail() + i) % INDEX_RING_CAPACITY;
        let event = unsafe { journal.read_event_at(slot) };

        if is_empty_event(&event) {
            continue;
        }

        // Core filters
        if let Some(nid) = params.node_id {
            if event.node_id != nid {
                continue;
            }
        }
        if let Some(sid) = params.stream_id {
            if event.stream_id != sid {
                continue;
            }
        }
        if let Some(min) = params.ts_min {
            if event.lamport_ts < min {
                continue;
            }
        }
        if let Some(max) = params.ts_max {
            if event.lamport_ts > max {
                continue;
            }
        }

        // DSL query (minimal evaluator for demo/expansion)
        if let Some(ref q) = params.query {
            if !evaluate_dsl(q, &event) {
                continue;
            }
        }

        if skipped < offset {
            skipped += 1;
            continue;
        }

        records.push(EventRecord {
            slot,
            lamport_ts: event.lamport_ts,
            node_id: event.node_id,
            stream_id: event.stream_id,
            payload_offset: event.payload_offset,
            checksum: event.checksum,
            checkpoint: event.is_checkpoint(),
        });
    }

    Ok(Json(EventListResponse {
        events: records,
        total,
        offset,
        limit,
    }))
}

async fn api_event_detail(
    State(state): State<Arc<AppState>>,
    axum::extract::Path(slot): axum::extract::Path<usize>,
) -> Result<Json<EventDetailRecord>, (StatusCode, Json<ApiError>)> {
    let primary = state.get_journal(None).await.unwrap();
    let journal = primary.journal.read().await;

    if slot >= INDEX_RING_CAPACITY {
        return Err((
            StatusCode::NOT_FOUND,
            Json(ApiError {
                error: format!("Slot {} out of range", slot),
            }),
        ));
    }

    let event = unsafe { journal.read_event_at(slot) };
    if is_empty_event(&event) {
        return Err((
            StatusCode::NOT_FOUND,
            Json(ApiError {
                error: format!("Slot {} is empty", slot),
            }),
        ));
    }

    let blob = journal.blob_storage();
    let payload_start = event.payload_offset as usize;
    let payload_end = (payload_start + 256).min(blob.len());
    let payload_slice = if payload_start < blob.len() {
        &blob[payload_start..payload_end]
    } else {
        &[]
    };

    let payload_hex = payload_slice
        .iter()
        .map(|b| format!("{:02x}", b))
        .collect::<Vec<_>>()
        .chunks(16)
        .map(|c| c.join(" "))
        .collect::<Vec<_>>()
        .join("\n");

    let payload_ascii: String = payload_slice
        .iter()
        .map(|&b| {
            if b.is_ascii_graphic() || b == b' ' {
                b as char
            } else {
                '.'
            }
        })
        .collect();

    Ok(Json(EventDetailRecord {
        event: EventRecord {
            slot,
            lamport_ts: event.lamport_ts,
            node_id: event.node_id,
            stream_id: event.stream_id,
            payload_offset: event.payload_offset,
            checksum: event.checksum,
            checkpoint: event.is_checkpoint(),
        },
        payload_hex,
        payload_ascii,
        payload_size: payload_slice.len(),
    }))
}

async fn api_verify(State(_state): State<Arc<AppState>>) -> Json<VerifyResult> {
    let start = Instant::now();
    let timestamp = chrono::Utc::now().to_rfc3339();

    let output = tokio::process::Command::new("cargo")
        .args(["test", "--workspace", "--", "--quiet"])
        .current_dir(".")
        .output()
        .await;

    match output {
        Ok(out) => {
            let stdout = String::from_utf8_lossy(&out.stdout);
            let stderr = String::from_utf8_lossy(&out.stderr);
            let combined = format!("{}\n{}", stdout, stderr);
            Json(VerifyResult {
                success: out.status.success(),
                output: combined.trim().to_string(),
                duration_ms: start.elapsed().as_millis() as u64,
                timestamp,
            })
        }
        Err(e) => Json(VerifyResult {
            success: false,
            output: format!("Failed to execute: {}", e),
            duration_ms: start.elapsed().as_millis() as u64,
            timestamp,
        }),
    }
}

// =============================================================================
// New API Handlers
// =============================================================================

async fn api_simulate(
    State(state): State<Arc<AppState>>,
    Json(params): Json<SimulateParams>,
) -> Result<Json<SimulateResult>, (StatusCode, Json<ApiError>)> {
    let count = params.count.unwrap_or(100).min(10000);
    let base_node = params.node_id.unwrap_or(1);
    let base_stream = params.stream_id.unwrap_or(0);

    let journal_path = params.journal.clone();
    let primary = state.get_journal(journal_path).await.ok_or((
        StatusCode::NOT_FOUND,
        Json(ApiError {
            error: "Journal not found".into(),
        }),
    ))?;

    let mut journal = primary.journal.write().await;
    let mut cursor = primary.cursor.write().await;
    let base_ts = cz_io::event_loop::EVENTS_PROCESSED.load(Ordering::Relaxed);

    let mut created = 0;
    for i in 0..count {
        if cursor.is_full() {
            break;
        }

        let slot = match cursor.advance_head() {
            Some(s) => s,
            None => break,
        };

        let event = CausalEvent::new(
            base_ts + i as u64 + 1,                    // monotonic-ish for simulation
            base_node + (i as u32 % 5),                // node_id: cycle through 5 nodes
            base_stream + (i as u16 % 3),              // stream_id: cycle through 3 streams
            (slot * CausalEvent::size_bytes()) as u64, // payload_offset
            0,                                         // checksum
        );

        unsafe {
            journal.write_event_at(slot, &event);
        }
        created += 1;
    }

    // Update global counters
    cz_io::event_loop::EVENTS_PROCESSED.fetch_add(created as u64, Ordering::Relaxed);
    cz_io::event_loop::BYTES_PROCESSED.fetch_add(
        (created * CausalEvent::size_bytes()) as u64,
        Ordering::Relaxed,
    );

    Ok(Json(SimulateResult {
        events_created: created,
        head_after: cursor.head(),
    }))
}

async fn api_replay(
    State(state): State<Arc<AppState>>,
    Json(params): Json<ReplayParams>,
) -> Result<Json<ReplayResult>, (StatusCode, Json<ApiError>)> {
    let source_primary = state.get_journal(params.journal.clone()).await.ok_or((
        StatusCode::NOT_FOUND,
        Json(ApiError {
            error: "Source journal not found".into(),
        }),
    ))?;

    let target_primary = state
        .get_journal(params.target_journal.clone())
        .await
        .ok_or((
            StatusCode::NOT_FOUND,
            Json(ApiError {
                error: "Target journal not found".into(),
            }),
        ))?;

    let source_journal = source_primary.journal.read().await;
    let mut target_journal = target_primary.journal.write().await;
    let mut target_cursor = target_primary.cursor.write().await;

    let start = params.start_slot.min(INDEX_RING_CAPACITY.saturating_sub(1));
    let end = params.end_slot.min(INDEX_RING_CAPACITY.saturating_sub(1));
    if start > end {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ApiError {
                error: "start_slot must be <= end_slot".into(),
            }),
        ));
    }

    let mut replayed = 0;
    for slot in start..=end {
        if target_cursor.is_full() {
            break;
        }

        let event = unsafe { source_journal.read_event_at(slot) };
        if is_empty_event(&event) {
            continue;
        } // Skip empty slots

        let target_slot = match target_cursor.advance_head() {
            Some(s) => s,
            None => break,
        };

        // We preserve the original event content but it's re-sequenced at the head
        unsafe {
            target_journal.write_event_at(target_slot, &event);
        }
        replayed += 1;
    }

    // Update global counters
    cz_io::event_loop::EVENTS_PROCESSED.fetch_add(replayed as u64, Ordering::Relaxed);
    cz_io::event_loop::BYTES_PROCESSED.fetch_add(
        (replayed * CausalEvent::size_bytes()) as u64,
        Ordering::Relaxed,
    );

    Ok(Json(ReplayResult {
        events_replayed: replayed,
        new_head: target_cursor.head(),
    }))
}

async fn api_topology(
    State(state): State<Arc<AppState>>,
    Query(params): Query<HashMap<String, String>>,
) -> Result<Json<TopologyResponse>, (StatusCode, Json<ApiError>)> {
    let journal_path = params.get("journal");
    let primary = state.get_journal(journal_path.cloned()).await.ok_or((
        StatusCode::NOT_FOUND,
        Json(ApiError {
            error: "Journal not found".into(),
        }),
    ))?;

    let journal = primary.journal.read().await;
    let cursor = primary.cursor.read().await;
    let total = cursor.len();

    let mut node_map: HashMap<u32, (usize, Vec<u16>, u64, u64)> = HashMap::new();

    for i in 0..total.min(50000) {
        let slot = (cursor.tail() + i) % INDEX_RING_CAPACITY;
        let event = unsafe { journal.read_event_at(slot) };
        if is_empty_event(&event) {
            continue;
        }
        let entry = node_map
            .entry(event.node_id)
            .or_insert((0, Vec::new(), u64::MAX, 0));
        entry.0 += 1;
        if !entry.1.contains(&event.stream_id) {
            entry.1.push(event.stream_id);
        }
        entry.2 = entry.2.min(event.lamport_ts);
        entry.3 = entry.3.max(event.lamport_ts);
    }

    let total_streams: usize = node_map.values().map(|v| v.1.len()).sum();
    let nodes: Vec<TopologyNode> = node_map
        .into_iter()
        .map(|(node_id, (count, streams, first, last))| TopologyNode {
            node_id,
            event_count: count,
            streams,
            first_seen_ts: first,
            last_seen_ts: last,
        })
        .collect();

    Ok(Json(TopologyResponse {
        total_nodes: nodes.len(),
        total_streams,
        total_events: total,
        nodes,
    }))
}

async fn api_streams(
    State(state): State<Arc<AppState>>,
    Query(params): Query<HashMap<String, String>>,
) -> Result<Json<StreamsResponse>, (StatusCode, Json<ApiError>)> {
    let journal_path = params.get("journal");
    let primary = state.get_journal(journal_path.cloned()).await.ok_or((
        StatusCode::NOT_FOUND,
        Json(ApiError {
            error: "Journal not found".into(),
        }),
    ))?;

    let journal = primary.journal.read().await;
    let cursor = primary.cursor.read().await;
    let total = cursor.len();

    let mut stream_map: HashMap<u16, (usize, Vec<u32>, u64, u64)> = HashMap::new();

    for i in 0..total.min(50000) {
        let slot = (cursor.tail() + i) % INDEX_RING_CAPACITY;
        let event = unsafe { journal.read_event_at(slot) };
        if is_empty_event(&event) {
            continue;
        }
        let entry = stream_map
            .entry(event.stream_id)
            .or_insert((0, Vec::new(), u64::MAX, 0));
        entry.0 += 1;
        if !entry.1.contains(&event.node_id) {
            entry.1.push(event.node_id);
        }
        entry.2 = entry.2.min(event.lamport_ts);
        entry.3 = entry.3.max(event.lamport_ts);
    }

    let streams: Vec<StreamStat> = stream_map
        .into_iter()
        .map(|(stream_id, (count, nodes, min_ts, max_ts))| StreamStat {
            stream_id,
            event_count: count,
            nodes,
            min_ts,
            max_ts,
        })
        .collect();

    Ok(Json(StreamsResponse {
        total_streams: streams.len(),
        streams,
    }))
}

async fn api_journal_layout(State(state): State<Arc<AppState>>) -> Json<JournalLayout> {
    let primary = state.get_journal(None).await.unwrap();
    let journal = primary.journal.read().await;
    let cursor = primary.cursor.read().await;

    Json(JournalLayout {
        total_size_bytes: journal.size(),
        index_ring_start: 0,
        index_ring_end: INDEX_RING_SIZE,
        index_ring_size_bytes: INDEX_RING_SIZE,
        index_ring_slot_count: INDEX_RING_CAPACITY,
        index_ring_slot_size: CausalEvent::size_bytes(),
        blob_storage_start: INDEX_RING_SIZE,
        blob_storage_end: journal.size(),
        blob_storage_size_bytes: journal.size() - INDEX_RING_SIZE as u64,
        slots_used: cursor.len(),
        slots_free: INDEX_RING_CAPACITY - cursor.len(),
    })
}

async fn api_system(State(state): State<Arc<AppState>>) -> Json<SystemResources> {
    let pid = std::process::id();
    let mut rss = 0u64;
    let mut vms = 0u64;
    let mut threads = 0u64;

    // Read from /proc/self/status
    if let Ok(content) = std::fs::read_to_string("/proc/self/status") {
        for line in content.lines() {
            if let Some(val) = line.strip_prefix("VmRSS:") {
                rss = val
                    .trim()
                    .split_whitespace()
                    .next()
                    .and_then(|v| v.parse().ok())
                    .unwrap_or(0);
            } else if let Some(val) = line.strip_prefix("VmSize:") {
                vms = val
                    .trim()
                    .split_whitespace()
                    .next()
                    .and_then(|v| v.parse().ok())
                    .unwrap_or(0);
            } else if let Some(val) = line.strip_prefix("Threads:") {
                threads = val.trim().parse().unwrap_or(0);
            }
        }
    }

    Json(SystemResources {
        pid,
        memory_rss_kb: rss,
        memory_vms_kb: vms,
        threads,
        uptime_seconds: state.start_time.elapsed().as_secs(),
    })
}

async fn api_metrics_history(
    State(state): State<Arc<AppState>>,
    Query(params): Query<HashMap<String, String>>,
) -> Json<Vec<MetricsSnapshot>> {
    let minutes = params
        .get("minutes")
        .and_then(|v| v.parse::<usize>().ok())
        .unwrap_or(5);

    let count = (minutes * 60).min(3600);
    let history = state.metrics_history.read().await;
    let snapshots: Vec<MetricsSnapshot> = history
        .iter()
        .rev()
        .take(count)
        .cloned()
        .collect::<Vec<_>>()
        .into_iter()
        .rev()
        .collect();

    Json(snapshots)
}

async fn api_alerts_get(State(state): State<Arc<AppState>>) -> Json<Vec<Alert>> {
    let alerts = state.alerts.read().await;
    Json(alerts.iter().rev().take(50).cloned().collect())
}

async fn api_alert_rules_get(State(state): State<Arc<AppState>>) -> Json<Vec<AlertRule>> {
    let rules = state.alert_rules.read().await;
    Json(rules.clone())
}

async fn api_alert_rules_set(
    State(state): State<Arc<AppState>>,
    Json(rules): Json<Vec<AlertRule>>,
) -> Json<Vec<AlertRule>> {
    let mut current = state.alert_rules.write().await;
    *current = rules.clone();
    Json(rules)
}

async fn api_export(
    State(state): State<Arc<AppState>>,
    Query(params): Query<ExportParams>,
) -> impl IntoResponse {
    let format = params.format.unwrap_or_else(|| "json".into());
    let limit = params.limit.unwrap_or(1000).min(50000);

    let journal_path = params.journal.clone();
    let Some(primary) = state.get_journal(journal_path).await else {
        return (
            StatusCode::NOT_FOUND,
            [(header::CONTENT_TYPE, "application/json")],
            r#"{"error":"Journal not found"}"#.to_string(),
        )
            .into_response();
    };

    let journal = primary.journal.read().await;
    let cursor = primary.cursor.read().await;
    let total = cursor.len().min(limit);

    let mut events = Vec::with_capacity(total);
    for i in 0..total {
        let slot = (cursor.tail() + i) % INDEX_RING_CAPACITY;
        let event = unsafe { journal.read_event_at(slot) };
        if is_empty_event(&event) {
            continue;
        }
        events.push(EventRecord {
            slot,
            lamport_ts: event.lamport_ts,
            node_id: event.node_id,
            stream_id: event.stream_id,
            payload_offset: event.payload_offset,
            checksum: event.checksum,
            checkpoint: event.is_checkpoint(),
        });
    }

    match format.as_str() {
        "csv" => {
            let mut csv = String::from(
                "slot,lamport_ts,node_id,stream_id,payload_offset,checksum,checkpoint\n",
            );
            for e in &events {
                csv.push_str(&format!(
                    "{},{},{},{},{},{},{}\n",
                    e.slot,
                    e.lamport_ts,
                    e.node_id,
                    e.stream_id,
                    e.payload_offset,
                    e.checksum,
                    e.checkpoint
                ));
            }
            (
                StatusCode::OK,
                [
                    (header::CONTENT_TYPE, "text/csv"),
                    (
                        header::CONTENT_DISPOSITION,
                        "attachment; filename=\"causal-events.csv\"",
                    ),
                ],
                csv,
            )
                .into_response()
        }
        _ => {
            let json = serde_json::to_string_pretty(&events).unwrap_or_default();
            (
                StatusCode::OK,
                [
                    (header::CONTENT_TYPE, "application/json"),
                    (
                        header::CONTENT_DISPOSITION,
                        "attachment; filename=\"causal-events.json\"",
                    ),
                ],
                json,
            )
                .into_response()
        }
    }
}

// =============================================================================
// WebSocket Handler
// =============================================================================

async fn ws_handler(ws: WebSocketUpgrade, State(state): State<Arc<AppState>>) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_socket(socket, state))
}

async fn handle_socket(mut socket: WebSocket, state: Arc<AppState>) {
    let interval_ms = state.config.server.metrics_interval_ms;
    let mut interval = tokio::time::interval(std::time::Duration::from_millis(interval_ms));
    let mut prev_events: u64 = 0;
    let mut prev_bytes: u64 = 0;
    let mut prev_time = Instant::now();

    loop {
        interval.tick().await;

        let now = Instant::now();
        let dt = now.duration_since(prev_time).as_secs_f64();

        let events = cz_io::event_loop::EVENTS_PROCESSED.load(Ordering::Relaxed);
        let bytes = cz_io::event_loop::BYTES_PROCESSED.load(Ordering::Relaxed);

        let tps = if dt > 0.0 {
            events.saturating_sub(prev_events) as f64 / dt
        } else {
            0.0
        };
        let bps = if dt > 0.0 {
            bytes.saturating_sub(prev_bytes) as f64 / dt
        } else {
            0.0
        };

        prev_events = events;
        prev_bytes = bytes;
        prev_time = now;

        let primary = state.get_journal(None).await.unwrap();
        let cursor = primary.cursor.read().await;
        let used = cursor.len();
        let utilization = if INDEX_RING_CAPACITY > 0 {
            (used as f64 / INDEX_RING_CAPACITY as f64) * 100.0
        } else {
            0.0
        };

        let snapshot = MetricsSnapshot {
            timestamp: chrono::Utc::now().to_rfc3339(),
            events,
            bytes,
            tps: (tps * 100.0).round() / 100.0,
            bps: (bps * 100.0).round() / 100.0,
            head: cursor.head(),
            tail: cursor.tail(),
            utilization_pct: (utilization * 100.0).round() / 100.0,
            uptime_seconds: state.start_time.elapsed().as_secs(),
            playback_mode: state.playback.read().await.clone(),
        };

        let msg = MetricsMessage {
            r#type: "metrics",
            data: snapshot,
        };
        let json = serde_json::to_string(&msg).unwrap_or_default();

        if socket.send(Message::Text(json)).await.is_err() {
            break;
        }
    }
}

async fn api_metrics_prometheus(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let events = cz_io::event_loop::EVENTS_PROCESSED.load(Ordering::Relaxed);
    let bytes = cz_io::event_loop::BYTES_PROCESSED.load(Ordering::Relaxed);

    let mut body = String::new();
    body.push_str("# HELP cz_events_total Total number of events processed\n");
    body.push_str("# TYPE cz_events_total counter\n");
    body.push_str(&format!("cz_events_total {}\n", events));

    body.push_str("# HELP cz_bytes_total Total number of bytes processed\n");
    body.push_str("# TYPE cz_bytes_total counter\n");
    body.push_str(&format!("cz_bytes_total {}\n", bytes));

    let journals = state.journals.read().await;
    for (path, s) in journals.iter() {
        let p_str = path.display().to_string();
        let cursor = s.cursor.read().await;
        body.push_str(&format!(
            "cz_ring_utilization_pct{{journal=\"{}\"}} {}\n",
            p_str,
            (cursor.len() as f64 / INDEX_RING_CAPACITY as f64) * 100.0
        ));
    }

    (
        StatusCode::OK,
        [(header::CONTENT_TYPE, "text/plain; version=0.0.4")],
        body,
    )
}
async fn api_playback_get(State(state): State<Arc<AppState>>) -> Json<PlaybackMode> {
    let mode = state.playback.read().await;
    Json(mode.clone())
}

async fn api_playback_set(
    State(state): State<Arc<AppState>>,
    Json(params): Json<PlaybackSetParams>,
) -> Result<Json<PlaybackMode>, (StatusCode, Json<ApiError>)> {
    let mut mode = state.playback.write().await;
    match params.mode.as_str() {
        "real_time" => {
            *mode = PlaybackMode::RealTime;
        }
        "paused" => {
            let slot = params.slot.unwrap_or(0);
            let ts = params.ts.unwrap_or(0);
            *mode = PlaybackMode::Paused {
                at_slot: slot,
                at_ts: ts,
            };
        }
        _ => {
            return Err((
                StatusCode::BAD_REQUEST,
                Json(ApiError {
                    error: "Invalid playback mode".into(),
                }),
            ))
        }
    }
    Ok(Json(mode.clone()))
}
fn evaluate_dsl(query: &str, event: &CausalEvent) -> bool {
    // Simple mock DSL: "field == value" or "field > value"
    // In a real app, this would use a parser like 'nom' or 'evalexpr'
    let parts: Vec<&str> = query.split_whitespace().collect();
    if parts.len() < 3 {
        return true;
    }

    let field = parts[0];
    let op = parts[1];
    let val_str = parts[2];

    let val = match val_str.parse::<u64>() {
        Ok(v) => v,
        Err(_) => return true,
    };

    match field {
        "node_id" => match op {
            "==" => event.node_id as u64 == val,
            ">" => event.node_id as u64 > val,
            "<" => (event.node_id as u64) < val,
            _ => true,
        },
        "stream_id" => match op {
            "==" => event.stream_id as u64 == val,
            ">" => event.stream_id as u64 > val,
            "<" => (event.stream_id as u64) < val,
            _ => true,
        },
        "ts" | "lamport" => match op {
            "==" => event.lamport_ts == val,
            ">" => event.lamport_ts > val,
            "<" => event.lamport_ts < val,
            _ => true,
        },
        _ => true,
    }
}

// =============================================================================
// Auth Middleware
// =============================================================================

async fn auth_middleware(
    State(state): State<Arc<AppState>>,
    req: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    let path = req.uri().path();
    let method = req.method().clone();

    // Public routes bypass
    if path == "/api/status"
        || path.starts_with("/ws")
        || !path.starts_with("/api")
        || method == Method::OPTIONS
    {
        return Ok(next.run(req).await);
    }

    // Check Authorization header
    let auth_header = req
        .headers()
        .get(axum::http::header::AUTHORIZATION)
        .and_then(|header| header.to_str().ok());

    match auth_header {
        Some(header) if header.starts_with("Bearer ") => {
            let token = &header[7..];
            if let Some(key) = state.auth_layer.validate_token(token).await {
                if let Some(scope) = required_scope(path, &method) {
                    if !state.auth_layer.has_scope(&key, scope) {
                        tracing::warn!("Insufficient scope for {} {}", method, path);
                        return Err(StatusCode::FORBIDDEN);
                    }
                }
                Ok(next.run(req).await)
            } else {
                tracing::warn!("Invalid API Key for {}", path);
                Err(StatusCode::UNAUTHORIZED)
            }
        }
        _ => {
            tracing::warn!("Missing Authorization header for {}", path);
            Err(StatusCode::UNAUTHORIZED)
        }
    }
}

fn required_scope(path: &str, method: &Method) -> Option<auth::Scope> {
    if !path.starts_with("/api") {
        return None;
    }
    if path == "/api/status" {
        return None;
    }
    if path.starts_with("/api/auth") {
        return Some(auth::Scope::Admin);
    }
    match *method {
        Method::GET | Method::HEAD => Some(auth::Scope::Read),
        _ => Some(auth::Scope::Write),
    }
}

fn is_empty_event(event: &CausalEvent) -> bool {
    event.lamport_ts == 0
        && event.node_id == 0
        && event.stream_id == 0
        && event.flags == 0
        && event.payload_offset == 0
        && event.checksum == 0
}
