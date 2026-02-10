//! # cz-cli — The "Moat" of LACRIMOSA
//!
//! Minimal CLI interface for the distributed sequencer.
//!
//! - `cz start --journal <path>` — Boot the io_uring event loop.
//! - `cz verify` — Run Kani proofs.
//! - `cz status` — Report system metrics.

use std::path::PathBuf;
use std::process::Command;

use clap::{Parser, Subcommand};

use cz_io::cursor::Cursor;
use cz_io::event_loop::{EventLoop, EventLoopConfig};
use cz_io::journal::Journal;

/// 🧬 LACRIMOSA — A hyper-efficient, formally verified distributed sequencer.
#[derive(Parser)]
#[command(name = "cz", version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Boot the sequencer event loop.
    Start {
        /// Path to the journal file (or block device).
        #[arg(long)]
        journal: PathBuf,

        /// Journal size in GiB (default: 100).
        #[arg(long, default_value_t = 100)]
        size_gib: u64,

        /// UDP bind address (default: 0.0.0.0:9000).
        #[arg(long, default_value = "0.0.0.0:9000")]
        bind: String,
    },

    /// Run Kani formal verification proofs.
    Verify,

    /// Report system status as JSON.
    Status,

    /// Launch the LACRIMOSA Control Center.
    Hub {
        /// Path to the journal file(s) (supports multiple).
        #[arg(long, default_value = "journal.db")]
        journals: Vec<PathBuf>,

        /// Server bind address.
        #[arg(long, default_value = "127.0.0.1:3000")]
        bind: String,
    },

    /// 🖤 LACRIMOSA — One-click total system ignition (Sequencer + Control Center).
    Lacrimosa {
        /// Path to the journal file.
        #[arg(long, default_value = "journal.db")]
        journal: PathBuf,

        /// Server bind address for the Hub.
        #[arg(long, default_value = "127.0.0.1:3000")]
        hub_bind: String,

        /// UDP bind address for the sequencer.
        #[arg(long, default_value = "0.0.0.0:9000")]
        seq_bind: String,
    },

    /// Manage connectors (list, add, remove).
    Connectors {
        #[command(subcommand)]
        action: ConnectorCmd,
    },

    /// Run a Causal Query Language (CQL) query.
    Query { query: String },

    /// Live tail a stream.
    Tail { stream: String },

    /// List active incidents.
    Incidents,

    /// Search traces.
    Traces {
        #[arg(long)]
        service: Option<String>,
        #[arg(long)]
        limit: Option<usize>,
    },
}

#[derive(Subcommand)]
enum ConnectorCmd {
    List,
    Add {
        kind: String, // kafka, webhook, nats
        #[arg(long)]
        config: String, // JSON config string
    },
    Remove {
        id: String,
    },
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Start {
            journal: journal_path,
            size_gib,
            bind,
        } => {
            eprintln!("🧬 LACRIMOSA: Booting sequencer...");
            eprintln!("   Journal: {}", journal_path.display());
            eprintln!("   Size:    {} GiB", size_gib);
            eprintln!("   Bind:    {}", bind);

            let size = size_gib * 1024 * 1024 * 1024;

            let mut journal = Journal::open(&journal_path, size).expect("Failed to open journal");

            let mut cursor = Cursor::for_index_ring();

            let config = EventLoopConfig {
                bind_addr: bind,
                ring_depth: 256,
            };

            let mut event_loop =
                EventLoop::new(&config).expect("Failed to create io_uring event loop");

            eprintln!("🧬 LACRIMOSA: Sequencer running. Press Ctrl+C to stop.");

            event_loop
                .run(&mut journal, &mut cursor)
                .expect("Event loop failed");
        }

        Commands::Verify => {
            eprintln!("🧬 LACRIMOSA: Running formal verification...");
            eprintln!("   Tool: Kani Model Checker");
            eprintln!("   Targets: cz-verify (ordering proofs), cz-io (ring invariants)");
            eprintln!();

            // Run Kani on cz-verify
            let verify_status = Command::new("cargo")
                .args(["kani", "--package", "cz-verify"])
                .status();

            let verify_passed = match verify_status {
                Ok(status) => {
                    if status.success() {
                        eprintln!("   ✅ cz-verify: ALL PROOFS PASSED");
                        true
                    } else {
                        eprintln!("   ❌ cz-verify: PROOF FAILURE");
                        false
                    }
                }
                Err(e) => {
                    eprintln!("   ⚠️  Kani not found: {}", e);
                    eprintln!("   Install with: cargo install kani-verifier && cargo kani setup");
                    false
                }
            };

            // Run Kani on cz-io
            let io_status = Command::new("cargo")
                .args(["kani", "--package", "cz-io"])
                .status();

            let io_passed = match io_status {
                Ok(status) => {
                    if status.success() {
                        eprintln!("   ✅ cz-io: ALL PROOFS PASSED");
                        true
                    } else {
                        eprintln!("   ❌ cz-io: PROOF FAILURE");
                        false
                    }
                }
                Err(e) => {
                    eprintln!("   ⚠️  Kani not found: {}", e);
                    false
                }
            };

            if verify_passed && io_passed {
                eprintln!();
                eprintln!("🧬 VERIFICATION COMPLETE: Mathematical Safety Confirmed.");
                std::process::exit(0);
            } else {
                eprintln!();
                eprintln!("🧬 VERIFICATION INCOMPLETE: One or more proofs failed.");
                std::process::exit(1);
            }
        }

        Commands::Status => {
            // Check verification status
            let kani_check = Command::new("cargo")
                .args(["kani", "--package", "cz-verify"])
                .output();

            let verified = match kani_check {
                Ok(output) => output.status.success(),
                Err(_) => false,
            };

            // Read live metrics from cz-io
            use std::sync::atomic::Ordering;
            let events = cz_io::event_loop::EVENTS_PROCESSED.load(Ordering::Relaxed);
            let bytes = cz_io::event_loop::BYTES_PROCESSED.load(Ordering::Relaxed);

            let status = serde_json::json!({
                "events_processed": events,
                "bytes_processed": bytes,
                "verified": verified,
                "engine": "io_uring (pipelined)",
                "zero_copy": "True (NIC -> mmap direct)",
                "event_size_bytes": cz_core::CausalEvent::size_bytes(),
            });

            println!("{}", serde_json::to_string_pretty(&status).unwrap());
        }

        Commands::Hub { journals, bind } => {
            eprintln!("🧬 LACRIMOSA: Launching Control Center...");
            eprintln!("   Journals: {:?}", journals);
            eprintln!("   Bind:    {}", bind);
            eprintln!();

            let mut args = vec!["run", "-p", "cz-hub", "--", "--bind", &bind];

            for j in &journals {
                args.push("--journals");
                args.push(j.to_str().unwrap());
            }

            let status = Command::new("cargo").args(&args).status();

            match status {
                Ok(s) if s.success() => {}
                Ok(s) => {
                    eprintln!("Hub exited with: {}", s);
                    std::process::exit(1);
                }
                Err(e) => {
                    eprintln!("Failed to launch hub: {}", e);
                    std::process::exit(1);
                }
            }
        }

        Commands::Lacrimosa {
            journal,
            hub_bind,
            seq_bind,
        } => {
            eprintln!("🖤 LACRIMOSA: Total System Ignition...");
            eprintln!("   Journal: {}", journal.display());
            eprintln!("   Seq Bind: {}", seq_bind);
            eprintln!("   Hub Bind: {}", hub_bind);
            eprintln!();

            // 1. Start Sequencer in a background thread
            let j_path = journal.clone();
            let s_bind = seq_bind.clone();
            std::thread::spawn(move || {
                let size = 100 * 1024 * 1024 * 1024; // Default 100GB
                let mut journal = Journal::open(&j_path, size).expect("Failed to open journal");
                let mut cursor = Cursor::for_index_ring();
                let config = EventLoopConfig {
                    bind_addr: s_bind,
                    ring_depth: 256,
                };
                let mut event_loop = EventLoop::new(&config).expect("Failed to create event loop");
                event_loop
                    .run(&mut journal, &mut cursor)
                    .expect("Sequencer failed");
            });

            // Wait a moment for the sequencer to bind/create journal if needed
            std::thread::sleep(std::time::Duration::from_millis(500));

            // 2. Launch Hub (Foreground)
            let status = Command::new("cargo")
                .args([
                    "run",
                    "-p",
                    "cz-hub",
                    "--",
                    "--journals",
                    &journal.display().to_string(),
                    "--bind",
                    &hub_bind,
                ])
                .status();

            match status {
                Ok(s) if s.success() => {}
                Ok(s) => {
                    eprintln!("Hub exited with: {}", s);
                    std::process::exit(1);
                }
                Err(e) => {
                    eprintln!("Failed to launch hub: {}", e);
                    std::process::exit(1);
                }
            }
        }

        // Async Commands
        cmd => {
            let rt = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .expect("Failed to build tokio runtime");

            rt.block_on(async_main(cmd));
        }
    }
}

async fn async_main(cmd: Commands) {
    let client = reqwest::Client::new();
    let base_url =
        std::env::var("CZ_BASE_URL").unwrap_or_else(|_| "http://127.0.0.1:3000".to_string());

    // Check if implicit key exists in environment
    // For MVP we assume NO AUTH in CLI or we need to pass headers.
    // Ideally we load from ~/.cz/config or env CZ_API_KEY.
    let api_key = std::env::var("CZ_API_KEY").ok();

    match cmd {
        Commands::Connectors { action } => match action {
            ConnectorCmd::List => {
                let url = format!("{}/api/connectors", base_url);
                match get_request(&client, &url, api_key.as_deref()).await {
                    Ok(resp) => {
                        // Parse and print table
                        if let Ok(json) = resp.json::<serde_json::Value>().await {
                            println!("{}", serde_json::to_string_pretty(&json).unwrap());
                        }
                    }
                    Err(e) => eprintln!("Error: {}", e),
                }
            }
            ConnectorCmd::Add { kind, config } => {
                let url = format!("{}/api/connectors", base_url);
                let raw_config = serde_json::from_str::<serde_json::Value>(&config)
                    .unwrap_or_else(|_| serde_json::json!({}));
                let params = raw_config
                    .as_object()
                    .map(|map| {
                        map.iter()
                            .map(|(k, v)| {
                                let value = v
                                    .as_str()
                                    .map(ToString::to_string)
                                    .unwrap_or_else(|| v.to_string());
                                (k.clone(), value)
                            })
                            .collect::<std::collections::HashMap<String, String>>()
                    })
                    .unwrap_or_default();
                let payload = serde_json::json!({
                    "name": format!("{}-{}", kind, uuid::Uuid::new_v4().as_simple()),
                    "kind": kind,
                    "params": params,
                });

                match post_request(&client, &url, api_key.as_deref(), &payload).await {
                    Ok(resp) => println!("Connector created: {}", resp.status()),
                    Err(e) => eprintln!("Error: {}", e),
                }
            }
            ConnectorCmd::Remove { id } => {
                let url = format!("{}/api/connectors/{}", base_url, id);
                let mut request = client.delete(&url);
                if let Some(k) = api_key.as_deref() {
                    request = request.header("Authorization", format!("Bearer {}", k));
                }
                match request.send().await {
                    Ok(resp) => println!("Connector removed: {}", resp.status()),
                    Err(e) => eprintln!("Error: {}", e),
                }
            }
        },

        Commands::Query { query } => {
            let url = format!("{}/api/query", base_url);
            let payload = serde_json::json!({ "query": query });
            match post_request(&client, &url, api_key.as_deref(), &payload).await {
                Ok(resp) => {
                    if let Ok(json) = resp.json::<serde_json::Value>().await {
                        // TODO: Pretty print table
                        println!("{}", serde_json::to_string_pretty(&json).unwrap());
                    }
                }
                Err(e) => eprintln!("Error: {}", e),
            }
        }

        Commands::Tail { stream } => {
            // Polling tail
            let mut offset = 0;
            let stream_id_filter = stream.parse::<u16>().ok();
            loop {
                let url = if let Some(stream_id) = stream_id_filter {
                    format!(
                        "{}/api/events?limit=100&offset={}&stream_id={}",
                        base_url, offset, stream_id
                    )
                } else {
                    format!("{}/api/events?limit=100&offset={}", base_url, offset)
                };

                match get_request(&client, &url, api_key.as_deref()).await {
                    Ok(resp) => {
                        if let Ok(json) = resp.json::<serde_json::Value>().await {
                            if let Some(events) = json.get("events").and_then(|e| e.as_array()) {
                                if events.is_empty() {
                                    tokio::time::sleep(tokio::time::Duration::from_millis(500))
                                        .await;
                                    continue;
                                }
                                for event in events {
                                    println!("{}", serde_json::to_string(event).unwrap());
                                }
                                offset += events.len();
                            }
                        }
                    }
                    Err(_) => tokio::time::sleep(tokio::time::Duration::from_secs(1)).await,
                }
            }
        }

        Commands::Incidents => {
            let url = format!("{}/api/alerts/incidents", base_url);
            match get_request(&client, &url, api_key.as_deref()).await {
                Ok(resp) => {
                    if let Ok(json) = resp.json::<serde_json::Value>().await {
                        println!("{}", serde_json::to_string_pretty(&json).unwrap());
                    }
                }
                Err(e) => eprintln!("Error: {}", e),
            }
        }

        Commands::Traces { service, limit } => {
            let mut params = vec![];
            if let Some(svc) = service {
                params.push(format!("service={}", svc));
            }
            if let Some(lim) = limit {
                params.push(format!("limit={}", lim));
            }
            let url = if params.is_empty() {
                format!("{}/api/traces", base_url)
            } else {
                format!("{}/api/traces?{}", base_url, params.join("&"))
            };
            match get_request(&client, &url, api_key.as_deref()).await {
                Ok(resp) => {
                    if let Ok(json) = resp.json::<serde_json::Value>().await {
                        println!("{}", serde_json::to_string_pretty(&json).unwrap());
                    }
                }
                Err(e) => eprintln!("Error: {}", e),
            }
        }

        _ => {}
    }
}

async fn get_request(
    client: &reqwest::Client,
    url: &str,
    key: Option<&str>,
) -> Result<reqwest::Response, reqwest::Error> {
    let mut req = client.get(url);
    if let Some(k) = key {
        req = req.header("Authorization", format!("Bearer {}", k));
    }
    req.send().await
}

async fn post_request(
    client: &reqwest::Client,
    url: &str,
    key: Option<&str>,
    json: &serde_json::Value,
) -> Result<reqwest::Response, reqwest::Error> {
    let mut req = client.post(url).json(json);
    if let Some(k) = key {
        req = req.header("Authorization", format!("Bearer {}", k));
    }
    req.send().await
}
