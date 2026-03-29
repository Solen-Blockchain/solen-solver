//! Intent watcher: polls L1 for pending intents.

use std::sync::Arc;

use solen_intents::pool::IntentPool;
use tracing::debug;

/// Poll the L1 node for new intents and add them to the local pool.
pub async fn run_watcher(
    rpc_url: &str,
    pool: Arc<IntentPool>,
    poll_interval_secs: u64,
) {
    let client = reqwest::Client::new();
    let mut interval =
        tokio::time::interval(tokio::time::Duration::from_secs(poll_interval_secs));

    loop {
        interval.tick().await;

        // Poll the L1 intent system contract for pending intents.
        // In a full implementation, this would call a dedicated RPC method
        // like solen_getPendingIntents and add them to the local pool.
        debug!(pending = pool.pending_count(), "polling for intents");

        // TODO: When the L1 exposes an intent submission RPC,
        // fetch pending intents and add them to the pool here.
    }
}
