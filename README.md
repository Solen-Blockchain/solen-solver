# Solen Solver

Intent solver service for the Solen network. Watches for pending user intents, computes optimal solutions, and submits them to earn tips.

## How It Works

1. Users submit **intents** (desired outcomes) to the L1 network
2. The solver polls for pending intents
3. For each intent, the solver tries to produce a **solution** (a set of operations that fulfills the constraints)
4. Solutions are submitted to the L1 intent pool
5. The best solution (highest score, lowest tip claim) wins
6. The solver earns the claimed portion of the intent's tip

## Running

```bash
cargo run --bin solen-solver -- \
    --rpc http://127.0.0.1:19944 \
    --port 4000 \
    --max-tip-pct 50
```

## Dashboard

| Endpoint | Description |
|----------|-------------|
| `GET /status` | Solver status, pending intents, solver address |
| `GET /health` | Health check |

## CLI Options

```
Options:
    --rpc <URL>              L1 RPC endpoint [default: http://127.0.0.1:19944]
    --port <PORT>            Dashboard port [default: 4000]
    --seed <HEX>             Solver keypair seed
    --poll-interval <SECS>   Intent poll interval [default: 2]
    --max-tip-pct <N>        Max tip claim percentage [default: 50]
```

## Solver Strategies

The default solver (`DirectTransferSolver`) handles simple transfer intents by converting `RequireTransfer` constraints directly into transfer operations.

Custom solvers can implement the `IntentSolver` trait:

```rust
pub trait IntentSolver: Send + Sync {
    fn solve(&self, intent: &Intent) -> Option<Solution>;
    fn solver_id(&self) -> AccountId;
}
```
