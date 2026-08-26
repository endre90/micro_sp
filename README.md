# Micro SP (Sequence Planner)

[![Documentation](https://img.shields.io/badge/docs-github--pages-blue)](https://endre90.github.io/micro_sp/) [![Coverage](https://img.shields.io/badge/coverage-97.98%25-brightgreen)](#tests)

A Sequence Planner runtime for controlling automation systems, in the form of a
library.

You add `micro_sp` as a dependency, describe *what your system can do* as a
model of guarded operations, and call `main_runner`. Your process becomes the
control system: it works out and executes *what the system should do now*. The
full system state lives in Redis, so several processes — and any external tool
or dashboard — observe and drive the same system.

There is no configuration file, no DSL file to load and no server to deploy.
The model is Rust, the state is Redis keys, and the runtime is eight tokio
tasks inside your binary.

**Contents** — [Quick start](#quick-start) · [Core pieces](#core-pieces) ·
[Guards and actions](#guards-and-actions) ·
[Four ways to make something happen](#four-ways-to-make-something-happen) ·
[How micro_sp runs them](#how-micro_sp-runs-them) ·
[Anatomy of an operation](#anatomy-of-an-operation) ·
[Examples](#examples) · [Configuration](#configuration) ·
[Activity log](#activity-log)

## Quick start

Start a Redis instance:

```bash
docker compose up -d
```

or, without the compose file:

```bash
docker run --name my-redis -p 6379:6379 -d redis
```

Then build a model, seed the state and hand it to the runners:

```rust
use micro_sp::*;
use std::sync::Arc;

#[tokio::main]
async fn main() {
    // The domain: one variable saying where the robot is.
    let mut domain = State::new();
    domain.add_mut(
        SPAssignment::new(SPVariable::new("pos", SPValueType::String), "a".to_spvalue()),
        "demo",
    );

    // One operation per hop the robot can make.
    let hop = |name: &str, from: &str, to: &str| {
        Operation::new(
            name,
            Some(10_000), // timeout while executing (ms)
            Some(10_000), // timeout while disabled (ms)
            None,         // failure retries
            None,         // timeout retries
            false,        // may not be bypassed
            vec![Transition::parse(
                "start",
                &format!("var:pos == {from}"),
                "true",
                Vec::<&str>::new(),
                Vec::<&str>::new(),
                &domain,
            )],
            vec![Transition::parse(
                "complete",
                "true",
                "true",
                vec![format!("var:pos <- {to}").as_str()],
                Vec::<&str>::new(),
                &domain,
            )],
            vec![], vec![], vec![], vec![],
        )
    };

    let model = Model::new(
        "sp",
        vec![],                                            // auto transitions
        vec![],                                            // auto operations
        vec![],                                            // mutexed auto operations
        vec![],                                            // SOPs
        vec![hop("a_to_b", "a", "b"), hop("b_to_c", "b", "c")], // planned operations
    );

    // Every key the runners will read has to exist before they start.
    let mut state = generate_runner_state_variables("sp", 1, "demo");
    state.extend_mut(generate_operation_state_variables(&model, false, "demo"), true);
    state.extend_mut(domain, true);

    let connection_manager = Arc::new(ConnectionManager::new().await);
    let mut con = connection_manager.get_connection().await;
    StateManager::set_state(&mut con, &state).await;

    main_runner(&"sp".to_string(), model, 1, &connection_manager).await;

    // The runners are detached tasks; keep the process alive.
    std::future::pending::<()>().await;
}
```

Ask the system for something by posting a goal predicate — `var:pos == c` — to
`sp_incoming_goals`. The planner finds the sequence `a_to_b, b_to_c` and the
plan runner executes it. See [Examples](#examples) for eight complete programs
that do exactly this and more.

## Core pieces

| Concept | What it is |
|---|---|
| `State` | Variable names to `SPValue`s. Any value may also be `UNKNOWN`. |
| `SPValue` | Bool, Int64, Float64, String, Time, Array, Map or Transform — each with an `UNKNOWN` variant. |
| `Predicate` | A boolean expression over the state; the guard half of a transition. |
| `Action` | An assignment to a state variable; the effect half. |
| `Transition` | A guard plus a set of assignments. |
| `Operation` | Transitions wrapped in a lifecycle: initial → executing → completed, with timeouts, retries, bypass and cancellation. |
| `SOP` / `SOPStruct` | Operations arranged into a named tree: sequence, parallel, alternative. |
| `Model` | Everything a system can do: automatic transitions, automatic operations, mutexed automatic operations, SOPs, and the operations the planner may use. |
| `main_runner` | Spawns the tasks that execute a model. |
| `StateManager` | Reads and writes state in Redis. |
| `TransformsManager` | The same, for 3D coordinate frames. |

## Guards and actions

Guards and actions are written in a small string DSL, parsed by
`Transition::parse`:

```rust
Transition::parse(
    "move_to_b",                         // transition id
    "var:pos == a && var:battery > 20",  // guard
    "var:enabled == true",               // runner guard
    vec!["var:pos <- b"],                // actions
    vec!["var:moves += 1"],              // runner actions
    &state,                              // the vars above have to be in the state
)
```

| Syntax | Meaning |
|---|---|
| `var:name` | A state variable. Resolved against the state passed to `parse`. |
| `==` `!=` `<` `<=` `>` `>=` | Comparison. Either side may be a variable or a literal. |
| `&&` `\|\|` `!` `( )` | Boolean composition. |
| `true` / `false` / `TRUE` / `FALSE` | Boolean literals. `FALSE &&  …` is a handy way to disable a branch. |
| `<-` | Assign. `var:pos <- b` |
| `+=` `-=` | Assign in place. `var:n += 2` |
| `UNKNOWN_int`, `UNKNOWN_string`, `UNKNOWN_bool`, … | The typed unknown values. |
| `"quoted string"`, `[a, b, c]`, `1.5`, `-3` | Literals. |

Two sharp edges worth knowing up front:

* **`var:` names are resolved at parse time.** The variable must already exist
  in the state you pass to `Transition::parse`, or it panics. A model that
  writes runner variables (`var:{sp_id}_sop_enabled`, `var:{sp_id}_plan`) has to
  be built against a state that already contains them — see
  `generate_runner_state_variables`.
* **A guard that fails to parse becomes `FALSE`,** and a bad action becomes a
  no-op. Both are logged as errors rather than panicking, so a typo shows up as
  an operation that mysteriously never starts. Check the log.

The full grammar is `pred_parser` in `src/modelling/parser.rs`.

## Four ways to make something happen

This is the part worth reading twice. `micro_sp` gives you four mechanisms, and
choosing between them is most of modelling.

| | Who decides the order | What starts it | Has a lifecycle? |
|---|---|---|---|
| **Automatic transition** | Nobody — it is a rule | Its guard holding | No |
| **Automatic operation** | Nobody — it self-starts | Its precondition holding | Yes |
| **SOP** | You, at modelling time | `{sp_id}_sop_enabled` being set | Yes, per operation and per tree |
| **Plan** | The planner, at runtime | A goal predicate being posted | Yes, per operation and per plan |

In one line: **a transition is a rule, an operation is a task, a SOP is a
script, a plan is a search result.**

### Automatic transitions

`Model::auto_transitions`. A guard and a set of assignments, taken by the
automatic transition runner the instant the guard holds — every tick, forever.
There is no timeout, no retry, no failure branch and nothing to schedule it.

Use them to react to measurements, derive state from other state, latch alarms,
or emit heartbeats. Because a transition completes within a single tick, it can
never wait for anything: if the thing you are modelling takes time, you want an
operation.

### Automatic operations

`Model::auto_operations` and `Model::mutexed_auto_operations`. A full
`Operation` — precondition, postcondition, deadline, retries, failure branches —
that nobody plans for. The automatic operation runner starts it whenever a
precondition holds, and it goes through the same lifecycle a planned operation
does.

Use them for background behaviour that still has to survive going wrong:
polling a sensor, keeping a resource warm, reacting to a fault. The
`mutexed_auto_operations` variant is identical except that only one of them may
execute at a time — the way to model a shared resource.

### SOPs

`Model::sops`. A **S**tandard **O**perating **P**rocedure is a *tree* of
operations with the order fixed when you write the model:

```rust
SOPStruct {
    id: "pick_and_place".to_string(),
    sop: SOP::Sequence(vec![
        SOP::Operation(Box::new(approach)),
        SOP::Parallel(vec![
            SOP::Operation(Box::new(close_gripper)),
            SOP::Operation(Box::new(start_conveyor)),
        ]),
        SOP::Alternative(vec![
            SOP::Operation(Box::new(place_in_bin_a)),
            SOP::Operation(Box::new(place_in_bin_b)),
        ]),
    ]),
}
```

* `Sequence` — children run one after another.
* `Parallel` — children run concurrently; the node completes when all have.
* `Alternative` — the node completes as soon as any one child has.

Use a SOP when the *route* is the requirement, not just the destination: a
recipe, a startup procedure, a certified sequence someone signed off on. A
plan cannot express "go to a, then b, then a again", because a goal predicate
can only describe where you end up. A SOP can.

A SOP does not start itself. The SOP runner executes whichever SOP
`{sp_id}_sop_id` names, once `{sp_id}_sop_enabled` is set — ordinary state
variables, so a dashboard, another process, or an automatic operation can all
enable one.

### Plans

`Model::operations`. These are the operations the *planner* may sequence. You
post a goal predicate and the planner searches for an order of operations that
reaches it:

```rust
StateManager::set_sp_value(
    &mut con,
    "sp_incoming_goals",
    &vec![goal_string_to_sp_value("", &"var:pos == c".to_string(), GoalPriority::Normal)]
        .to_spvalue(),
).await;
```

Use planning when you want to state the destination and let the system work out
the route — and especially when the route depends on state you do not know when
you write the model. If a plan turns out to be based on a wrong assumption, set
`{sp_id}_replan_for_same_goal` and the planner tries again from what is now
known to be true.

### So which one?

* It is a rule that never waits → **automatic transition**.
* It has to happen whenever conditions allow, and can go wrong → **automatic
  operation**.
* You know the exact order and it matters → **SOP**.
* You know the destination and want the system to find the route → **plan**.

Note that automatic operations, SOP steps and planned operations are all the
*same* `Operation` type. Which field of the `Model` an operation sits in is the
entire difference between "this runs on its own", "this is step three of a
procedure" and "the planner may use this".

## How micro_sp runs them

`main_runner` spawns eight detached tokio tasks:

| Runner | Job |
|---|---|
| `planner_ticker` | Searches `model.operations` for a plan to the current goal. |
| `planned_operation_runner` | Walks `{sp_id}_plan`, driving one operation at a time. |
| `sop_runner` | Walks the enabled SOP tree, driving its operations. |
| `auto_transition_runner` | Takes every automatic transition whose guard holds. |
| `auto_operation_runner` | Drives automatic and mutexed automatic operations. |
| `goal_runner` | Admits goals, orders them by priority, promotes one at a time. |
| `time_interface_runner` | Drives the `{sp_id}_timer_N_*` timers. |
| `tf_interface` | Serves 3D transform lookups and inserts. |

**They never call each other.** Every handover is a key in Redis. Each runner
loops: wait one tick, read only the key set it cares about, compute a new
`State`, and write back the diff. An idle stack therefore writes nothing at all.

That also means anything else that can reach Redis is a first-class participant.
A dashboard reads the same keys; a separate process can host a driver, post
goals, or enable a SOP, without linking against your binary.

### The goal path, end to end

```text
                    something posts a goal predicate
                                  |
                                  v
  {sp_id}_incoming_goals   -->  goal_runner admits it, assigns an id
                                  |
                                  v
  {sp_id}_scheduled_goals  -->  priority-ordered queue; one is promoted
                                  |
                                  v
  {sp_id}_current_goal_predicate
  {sp_id}_replan_trigger   -->  planner_ticker searches
                                  |
                                  v
  {sp_id}_plan                    ["op_a_to_b", "op_b_to_c"]
  {sp_id}_planner_state           found | not_found | ready
                                  |
                                  v
  {sp_id}_plan_current_step -->  planned_operation_runner drives step N:
                                  initial -> executing -> completed -> terminated_completed
                                  |
                                  v
  {sp_id}_plan_state == completed
                                  |
                                  v
                          goal_runner releases the goal, takes the next
```

### The SOP path

Set `{sp_id}_sop_id` and `{sp_id}_sop_enabled`. The SOP runner uniquifies the
tree's operations (so the same operation can appear twice without the two
occurrences being confused), walks it, and drives each operation through the
same machinery the plan runner uses. The tree's state is derived bottom-up from
its operations, and reported in `{sp_id}_sop_state`.

### The automatic paths

No trigger at all. Both automatic runners evaluate their whole set on every
tick, and act on whatever is enabled.

### Ticks

The default tick period is 5 ms and the floor is 1 ms; override it with
`MICRO_SP_TICK_INTERVAL_MS`. Since every hop between runners costs at least one
tick, the period is the latency floor for anything that has to travel through
Redis. The measured latency and CPU trade-offs are tabulated in the
[`running::tick`](https://endre90.github.io/micro_sp/micro_sp/running/tick/index.html)
module docs.

## Anatomy of an operation

An operation is the unit both the planner and the SOP runner schedule, and the
only place where going wrong is modelled. Here is one with every field spelled
out:

```rust
let move_robot = Operation::new(
    "robot_move_to_b",
    Some(5_000),   // timeout_executing_ms: give up after 5s in `executing`
    Some(10_000),  // timeout_disabled_ms:  give up after 10s never becoming enabled
    Some(2),       // failure_retries:      two more goes after a failure
    Some(1),       // timeout_retries:      one more go after a timeout
    true,          // can_be_bypassed:      may be waved through instead of going fatal

    // preconditions — the first one whose guard holds starts the operation
    vec![Transition::parse(
        "start_robot_move_to_b",
        "var:robot_request_state == initial \
         && var:robot_position_estimated != b",   // guard: the model's condition
        "true",                                    // runner_guard: operator permission
        vec![
            "var:robot_command_command <- move",
            "var:robot_position_command <- b",
            "var:robot_request_trigger <- true",
        ],
        Vec::<&str>::new(),                        // runner_actions
        &state,
    )],

    // postconditions — the first one whose guard holds completes it
    vec![Transition::parse(
        "complete_robot_move_to_b",
        "true",
        "var:robot_request_state == succeeded",    // wait for the hardware
        vec![
            "var:robot_request_trigger <- false",
            "var:robot_request_state <- initial",
            "var:robot_position_estimated <- b",
        ],
        Vec::<&str>::new(),
        &state,
    )],

    // failure_transitions — fire while executing; each one spends a retry
    vec![Transition::parse(
        "failed_robot_move_to_b",
        "true",
        "var:robot_request_state == failed",
        vec![
            "var:robot_request_trigger <- false",  // leave the hardware ready
            "var:robot_request_state <- initial",  // for the retry
        ],
        Vec::<&str>::new(),
        &state,
    )],

    vec![],  // timeout_transitions — empty: the timeout is unconditional
    vec![],  // bypass_transitions  — empty: bypass is unconditional
    vec![],  // cancel_transitions  — extra assignments on cancellation
);
```

### The fields

| Field | Meaning | Worth knowing |
|---|---|---|
| `name` | Unique, and also the state variable holding the lifecycle. | `Model::new` prefixes it with `op_`. |
| `timeout_executing_ms` | Deadline in `executing`. | `None` means `MAX_ALLOWED_OPERATION_DURATION_MS` (10 minutes), **not** "no timeout". |
| `timeout_disabled_ms` | Deadline in `disabled`, i.e. never becoming enabled. | Same `None` default. |
| `failure_retries` | Retries after a failure. | `None` → 0. Only ever spent if a `failure_transitions` guard actually fires. |
| `timeout_retries` | Retries after a timeout. | `None` → 0. |
| `can_be_bypassed` | Whether an exhausted operation is waved through. | `false` → it goes `Fatal` instead. |
| `preconditions` | Guards that start it. | The first one that holds is taken. |
| `postconditions` | Guards that complete it. | The first one that holds is taken. Several branches is how one operation models several outcomes. |
| `failure_transitions` | Guards that fail it while executing. | Their actions should leave the hardware ready for a retry. |
| `timeout_transitions` | Guards checked before timing out. | **All-or-nothing:** declare none and the timeout is unconditional; declare some and one must hold or the operation *cannot time out at all*. |
| `bypass_transitions` | Guards checked before bypassing. | Same all-or-nothing rule. |
| `cancel_transitions` | Extra assignments on cancellation. | |

Each transition has a **guard** and a **runner guard**, and both must hold. The
guard is the model's condition — the planner reasons about it. The runner guard
is the runtime's: operator permission, or waiting on hardware that no planner
can predict. Likewise `actions` are the modelled effects the planner searches
over, while `runner_actions` are applied only when actually running.

### The lifecycle

[`process_operation`](src/running/process_operation.rs) is called once per tick
for every active operation. It matches on the operation's current state and takes
the **first** branch whose guard holds, so the order matters as much as the arrows
do — see the precedence table below.

```text
                           initial
                              │
      ┌───────────────────────┴─────────────────────────────────┐
      │ a precondition holds                                    │ otherwise
      │                                                         v
      │                                                     disabled
      │                                                     │         │
      │                                a precondition holds │         │
      ├─────────────────────────────────────────────────────┘         |
      |                                                               |
      v                                     past timeout_disabled_ms  │
  executing                                                           │
      │                                                               │
      ├───────────────────────┬─────────────────────┐                 │
      │                       │                     │                 │
      │ a postcondition       │ a failure_          │ past timeout_   │
      │ holds                 │ transition holds    │ executing_ms    │
      │                       │                     │                 │
      v                       v                     v                 │
  completed                failed               timedout   <──────────┘
      │                       │                     │
      │                       └──────────┬──────────┘
      │                                  │
      │       ┌──────────────────────────┼────────────────────────┐
      │       │ a retry is left          │ out of retries,        │ out of retries,
      │       │ (counter += 1)           │ can_be_bypassed        │ otherwise
      │       v                          v                        v
      │    initial                   bypassed                   fatal
      │                                  |                        |
      |                                  |                        |
      |                                  v                        |
      └───────────────────────────>  terminated  <────────────────┘
```

Two things the picture leaves out:

* **`UNKNOWN` → `initial`.** A state value that does not parse as any of the
  above — a variable that was never initialised, or a garbled one — is forced
  back to `initial` rather than treated as an error.
* **`cancelled`.** A `stop` dashboard command cancels the operation, but only
  from `initial`, `disabled`, `executing`, `failed` or `timedout`. It is *not*
  reachable from `completed`, `bypassed`, `fatal` or `terminated_*`.

The `timedout` and `bypassed` edges also need their all-or-nothing
`timeout_transitions` / `bypass_transitions` to permit them, per the field table
above; declaring one whose guard never holds is how you forbid a timeout or a
bypass outright.

### What each state checks, in order

| In state | Checked in this order |
|---|---|
| `initial` | cancel → a precondition holds → **`executing`**; otherwise → **`disabled`** |
| `disabled` | cancel → past `timeout_disabled_ms` → **`timedout`**; a precondition holds → **`executing`**; otherwise wait |
| `executing` | cancel → a `failure_transition` holds → **`failed`**; past `timeout_executing_ms` → **`timedout`**; a postcondition holds → **`completed`**; otherwise wait |
| `failed` / `timedout` | cancel → a retry is left → **`initial`**; `can_be_bypassed` → **`bypassed`**; otherwise → **`fatal`** |

The precedences are load-bearing. A failure beats a timeout that came due on the
same tick, and both beat a postcondition that also holds, so a modelled failure
is never quietly completed. In `disabled`, the deadline is checked before the
precondition, so an operation whose guard first holds on the very tick its
disabled deadline passes times out instead of starting.

### Termination, and two sharp edges

All three end states terminate, each carrying the reason it got there:
`completed` → `terminated_completed`, `bypassed` → `terminated_bypassed`,
`fatal` → `terminated_fatal`. A `terminated_*` operation is inert — further ticks
leave it exactly where it is — and it is what a plan or SOP runner waits for
before taking its next step. `SOP::get_state` reads the reason back:
`Completed` and `Bypassed` both count as a finished step, `Fatal` fails the SOP.

Each of the three also does its own bookkeeping on the way out. `completed`
clears both retry counters and advances `plan_current_step` for a planned
operation; `bypassed` advances the cursor too; `fatal` sets `plan_state` to
`failed`.

**The dotted edges in the diagram are not wired up yet.**
[`Operation::terminate`](src/modelling/operation.rs) only implements
`TerminationReason::Completed`; its `_ => state.clone()` arm makes the other
three reasons silent no-ops. So today `bypassed`, `fatal` and `cancelled` stay
put, and `terminated_bypassed`, `terminated_fatal` and `terminated_cancelled` are
parseable but never actually written.

It bites hardest inside a SOP. `SOP::get_state` maps `bypassed` to
`SOPState::Executing` and only `terminated_bypassed` to `SOPState::Completed`, so
a bypassed operation leaves its branch reporting `Executing` forever and the SOP
never finishes. The plan runner escapes it: it advances the cursor on the way
past, so the stuck operation is simply never looked at again. `fatal` and
`cancelled` are read directly by `SOP::get_state` as well, so those two still
propagate. `a_bypassed_operation_never_lets_its_sop_finish`,
`a_bypassed_operation_advances_the_plan_but_never_terminates` and
`a_fatal_operation_stays_fatal_and_settles` pin all of it, so the gap cannot be
mistaken for working.

A retry also puts the operation back to `initial` **without** resetting
`{name}_elapsed_executing_ms` / `{name}_elapsed_disabled_ms`. Those counters are
already past the deadline that caused the timeout, so a timeout retry times out
again on the tick after it restarts — the pinned sequence is `executing →
timedout → initial → executing → timedout → fatal`. `timeout_retries` therefore
buys extra attempts, not extra time.

## Examples

Eight runnable programs live in [`examples/`](examples). Each one seeds its own
state, boots the full runner stack against emulated hardware, runs a single
scenario to completion, prints what happened, and exits — 0 on success,
non-zero on timeout.

Start Redis first:

```bash
docker compose up -d
```

Then:

```bash
cargo run --example sop_sequence
RUST_LOG=info cargo run --example planning_goals   # with runner logging
```

| Example | What it shows |
|---|---|
| [`auto_transitions`](examples/auto_transitions.rs) | Rules with no lifecycle. Two transitions blink a light three times and the model goes quiet on its own. No hardware. |
| [`auto_operations`](examples/auto_operations.rs) | Operations that start themselves. Two of them bounce the robot between `a` and `b`, waiting on hardware each time — something a transition cannot do. |
| [`planning_goals`](examples/planning_goals.rs) | Six goal predicates posted to `sp_incoming_goals`. The planner picks the operations; nothing writes an order down. |
| [`replanning`](examples/replanning.rs) | A plan built on a wrong assumption. The robot turns out to hold a different tool, `sp_replan_for_same_goal` is set, and a seven-step recovery plan appears without the goal being re-posted. |
| [`sop_sequence`](examples/sop_sequence.rs) | `SOP::Sequence`. A five-step a→b→a→b→a route — a *destination* goal could never express it. |
| [`sop_parallel`](examples/sop_parallel.rs) | `SOP::Parallel`. Robot and gantry move at the same time; the node waits for both. A plan is a list, so it cannot do this. |
| [`sop_alternative`](examples/sop_alternative.rs) | `SOP::Alternative`. Three routes, the first one blocked; whichever live branch wins closes the node. |
| [`failure_handling`](examples/failure_handling.rs) | The off-nominal lifecycle, three times over: retry then bypass, fail then fatal, timeout then fatal. |

Or run the lot:

```bash
for e in auto_transitions auto_operations planning_goals replanning \
         sop_sequence sop_parallel sop_alternative failure_handling; do
  cargo run --example "$e" || echo "FAILED: $e"
done
```

### The shared scaffolding

[`examples/common/`](examples/common) holds what the examples have in common:

* [`state.rs`](examples/common/state.rs) — the domain: a robot and a gantry,
  each with command variables, a `request_trigger` / `request_state` handshake,
  and `*_estimated` variables for what the model believes.
* [`emulators/`](examples/common/emulators) — a robot and a gantry emulator.
  Each is a tick loop with exactly the shape a real driver has, so swapping in
  real hardware means replacing the loop, not the model.
* [`mod.rs`](examples/common/mod.rs) — booting, waiting and printing.

How long the emulators take and whether they fail is itself state
(`*_emulate_execution_time`, `*_emulate_failure_rate`,
`*_emulate_failure_cause`), which is how `failure_handling` provokes failures on
demand.

### Watching one run

```bash
MICRO_SP_ACTIVITY_LOG_DIR=/tmp/micro_sp RUST_LOG=info \
  cargo run --example sop_sequence
tail -f /tmp/micro_sp/micro_sp.log
```

## Configuration

| Variable | Effect |
|---|---|
| `REDIS_HOST` / `REDIS_PORT` | Where to reach Redis (default `127.0.0.1:6379`). |
| `MICRO_SP_TICK_INTERVAL_MS` | Overrides the runners' tick period (default 5, floor 1). |
| `MICRO_SP_READ_FULL_STATE` | Runners read the whole keyspace each tick. Debugging escape hatch; slow. |
| `MICRO_SP_ACTIVITY_LOG` / `MICRO_SP_ACTIVITY_LOG_DIR` | Enables the on-disk activity log; the latter also sets its directory. |
| `MICRO_SP_ACTIVITY_LOG_MAX_MB` | Rotation threshold (default 5). |
| `MICRO_SP_ACTIVITY_LOG_MAX_FILES` | Rotated files to keep (default 10). |
| `MICRO_SP_ACTIVITY_LOG_SKIP` | Comma-separated variable-name suffixes to leave out of `VAR` lines. Defaults to the per-tick elapsed counters. |
| `RUST_LOG` / `LOG_SHOW_TIME` | Console logging verbosity and timestamps. |

## Activity log

Setting `MICRO_SP_ACTIVITY_LOG_DIR` records everything the system does to a
rotating file — operations, automatic transitions, SOP lifecycle and variable
changes, plus every console log line, each with a timestamp:

```
2026-08-17 10:42:00.577 | SOP   | sp_sop_runner | test_sop_MhcX6L      | initial -> executing
2026-08-17 10:40:44.878 | OP    | sp_op_runner  | op_a_to_b_zLmvT0QlAU | initial -> executing  (Starting)
2026-08-17 10:40:44.863 | TRANS | sp_auto_..    | beat                 | taken as 'beat_siBI079q95'
2026-08-17 10:40:44.869 | VAR   | sp_planner    | sp_plan              | [] -> [op_a_to_b, op_b_to_c]
2026-08-17 10:40:44.871 | INFO  | sp_planner    | running/plan_..rs:71 | Planning to reach: var == 1.
2026-08-17 10:40:44.884 | ERR   | sp_op_runner  | running/proce..rs:214| Operation timed out.
```

The last two are the same lines `RUST_LOG` prints to the console, mirrored here so
the prose saying *why* something happened sits next to the records saying *what*
happened. Their kind column is the level (`ERR`, `WARN`, `INFO`, `DEBUG`,
`TRACE`), their source column is the log target and their subject column is the
`file:line` that emitted them. `RUST_LOG` governs both views identically — a line
the console suppresses never reaches the file.

The active file is always `micro_sp.log`, so `tail -f` has a stable target; it
rotates at 5 MiB. Each kind sits in its own column, so it greps cleanly:

```bash
grep '| OP '  micro_sp.log            # operation state changes
grep '| VAR ' micro_sp.log | grep pos # one variable's history
grep '| ERR ' micro_sp.log            # just the errors
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
