//! Solen Intent Solver
//!
//! Watches the L1 node for pending intents, computes optimal solutions,
//! and submits them. Solvers compete to fulfill user intents and earn tips.
//!
//! Architecture:
//!   - Watcher: polls the L1 node for new intents
//!   - Solver: runs solver strategies to produce solutions
//!   - Submitter: submits winning solutions as L1 operations
//!   - API: HTTP dashboard for monitoring solver performance

mod solver;
mod watcher;

use std::sync::Arc;

use clap::Parser;
use solen_crypto::Keypair;
use solen_intents::pool::IntentPool;
use tracing::info;
use tracing_subscriber::EnvFilter;

#[derive(Parser)]
#[command(name = "solen-solver", version = "0.1.0")]
struct Cli {
    /// L1 RPC endpoint.
    #[arg(long, default_value = "http://127.0.0.1:19944")]
    rpc: String,

    /// Solver HTTP dashboard port.
    #[arg(long, default_value = "4000")]
    port: u16,

    /// Solver keypair seed (32-byte hex).
    #[arg(long)]
    seed: Option<String>,

    /// Poll interval for new intents (seconds).
    #[arg(long, default_value = "2")]
    poll_interval: u64,

    /// Maximum tip claim percentage (0-100).
    #[arg(long, default_value = "50")]
    max_tip_pct: u64,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .init();

    // Solver keypair.
    let keypair = match &cli.seed {
        Some(hex) => {
            let bytes = hex_decode(hex)?;
            let mut seed = [0u8; 32];
            seed.copy_from_slice(&bytes);
            Keypair::from_seed(&seed)
        }
        None => {
            info!("no --seed provided, generating random solver key");
            Keypair::generate()
        }
    };

    let solver_id = keypair.public_key();
    info!(
        solver = hex_encode(&solver_id),
        rpc = %cli.rpc,
        port = cli.port,
        "=== Solen Solver v0.1.0 ==="
    );

    // Local intent pool for tracking.
    let pool = Arc::new(IntentPool::new(1000));

    // Start intent watcher.
    let watcher_rpc = cli.rpc.clone();
    let watcher_pool = pool.clone();
    let poll_interval = cli.poll_interval;
    tokio::spawn(async move {
        watcher::run_watcher(&watcher_rpc, watcher_pool, poll_interval).await;
    });

    // Start solver loop.
    let solver_pool = pool.clone();
    let solver_rpc = cli.rpc.clone();
    let solver_kp = Arc::new(keypair);
    let max_tip = cli.max_tip_pct;
    tokio::spawn(async move {
        solver::run_solver(solver_pool, &solver_rpc, solver_kp, max_tip).await;
    });

    // Dashboard API.
    let dashboard_pool = pool.clone();
    let app = axum::Router::new()
        .route("/status", axum::routing::get(move || async move {
            let pending = dashboard_pool.pending_count();
            axum::Json(serde_json::json!({
                "solver": hex_encode(&solver_id),
                "pending_intents": pending,
                "status": "running"
            }))
        }))
        .route("/health", axum::routing::get(|| async { "ok" }));

    let addr: std::net::SocketAddr = format!("0.0.0.0:{}", cli.port).parse()?;
    info!(%addr, "solver dashboard started");
    info!("Dashboard: GET /status");

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

fn hex_encode(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

fn hex_decode(s: &str) -> anyhow::Result<Vec<u8>> {
    let s = s.strip_prefix("0x").unwrap_or(s);
    (0..s.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&s[i..i + 2], 16).map_err(Into::into))
        .collect()
}
