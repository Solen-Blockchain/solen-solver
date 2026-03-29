//! Solver engine: processes intents and produces solutions.

use std::sync::Arc;

use solen_crypto::Keypair;
use solen_intents::pool::IntentPool;
use solen_intents::solver::{DirectTransferSolver, IntentSolver};
use tracing::{debug, info};

/// Run the solver loop: check for pending intents and submit solutions.
pub async fn run_solver(
    pool: Arc<IntentPool>,
    rpc_url: &str,
    keypair: Arc<Keypair>,
    max_tip_pct: u64,
) {
    let solver = DirectTransferSolver {
        id: keypair.public_key(),
    };

    let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(2));

    loop {
        interval.tick().await;

        let intents = pool.pending_intents();
        if intents.is_empty() {
            continue;
        }

        debug!(count = intents.len(), "processing pending intents");

        for intent in &intents {
            // Try to solve the intent.
            if let Some(mut solution) = solver.solve(intent) {
                // Cap our tip claim.
                let max_claim = intent.tip * max_tip_pct as u128 / 100;
                solution.claimed_tip = solution.claimed_tip.min(max_claim);

                // Submit solution to the pool.
                match pool.submit_solution(solution) {
                    Ok(()) => {
                        info!(intent_id = intent.id, "solution submitted");
                    }
                    Err(e) => {
                        debug!(intent_id = intent.id, error = %e, "solution rejected");
                    }
                }
            }
        }
    }
}
