# Micro SP (Sequence Planner)

[![Documentation](https://img.shields.io/badge/docs-github--pages-blue)](https://endre90.github.io/micro_sp/)

A minimal Sequence Planner runtime for controlling automation systems.

You describe *what a system can do* as a model of guarded operations; the
runtime works out and executes *what it should do now*. The full system state
lives in Redis, so several processes and any external tool or dashboard ok,can
observe and drive the same system.

## Quick start

Start a Redis instance:

```bash
docker run --name my-redis -p 6379:6379 -d redis
```

Or with persistent storage:

```bash
docker run --name my-redis -p 6379:6379 -d redis redis-server --save 60 1 --loglevel warning
```

Then build a model and hand it to the runners:

```rust
use micro_sp::*;
use std::sync::Arc;

#[tokio::main]
async fn main() {
    let connection_manager = Arc::new(ConnectionManager::new().await);
    main_runner(&"sp".to_string(), model, 3, &connection_manager).await;
}
```

## Concepts

| Concept | What it is |
|---|---|
| `State` | Variable names to `SPValue`s. Any value may also be `UNKNOWN`. |
| `Transition` | A guard plus a set of assignments. |
| `Operation` | Transitions wrapped in a lifecycle: initial → executing → completed, with timeouts, retries and cancellation. |
| `SOPStruct` | Operations sequenced into a procedure (sequence, parallel, alternative). |
| `Model` | The operations, automatic transitions and SOPs a system has. |
| `main_runner` | Spawns the tasks that execute a model. |
| `StateManager` | Reads and writes state in Redis. |
| `TransformsManager` | The same, for 3D coordinate frames. |

Guards and actions are written in a small string DSL, e.g. `var:pos == a` and
`var:pos <- b`. See the `Transition::parse` docs.

## Configuration

| Variable | Effect |
|---|---|
| `REDIS_HOST` / `REDIS_PORT` | Where to reach Redis (default `127.0.0.1:6379`). |
| `MICRO_SP_TICK_INTERVAL_MS` | Overrides the runners' tick period. |
| `MICRO_SP_READ_FULL_STATE` | Runners read the whole keyspace each tick. Debugging escape hatch; slow. |
| `MICRO_SP_ACTIVITY_LOG_DIR` | Enables the on-disk activity log and sets its directory. |
| `MICRO_SP_ACTIVITY_LOG_MAX_MB` | Rotation threshold (default 5). |
| `MICRO_SP_ACTIVITY_LOG_MAX_FILES` | Rotated files to keep (default 10). |
| `RUST_LOG` / `LOG_SHOW_TIME` | Console logging verbosity and timestamps. |

## Activity log

Setting `MICRO_SP_ACTIVITY_LOG_DIR` records everything the system does to a
rotating file — operations, automatic transitions, SOP lifecycle and variable
changes, each with a timestamp:

```
2026-08-17 10:42:00.577 | SOP   | sp_sop_runner | test_sop_MhcX6L      | initial -> executing
2026-08-17 10:40:44.878 | OP    | sp_op_runner  | op_a_to_b_zLmvT0QlAU | initial -> executing  (Starting)
2026-08-17 10:40:44.863 | TRANS | sp_auto_..    | beat                 | taken as 'beat_siBI079q95'
2026-08-17 10:40:44.869 | VAR   | sp_planner    | sp_plan              | [] -> [op_a_to_b, op_b_to_c]
```

The active file is always `micro_sp.log`, so `tail -f` has a stable target; it
rotates at 5 MiB. Each kind sits in its own column, so it greps cleanly:

```bash
grep '| OP '  micro_sp.log            # operation state changes
grep '| VAR ' micro_sp.log | grep pos # one variable's history
```

## Documentation

Full API documentation is published at
**<https://endre90.github.io/micro_sp/>** and rebuilt automatically on every
push to `master`.

To build and read it locally instead:

```bash
cargo doc --open
```

## Tests

The test suite uses [testcontainers](https://crates.io/crates/testcontainers)
to start Redis, so Docker must be running. Tests bind a fixed port and must run
serially:

```bash
cargo test -- --test-threads=1
```

Code coverage:

```bash
cargo tarpaulin --out Html
```

## More on Redis

<https://hub.docker.com/_/redis>
