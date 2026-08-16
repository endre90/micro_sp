use crate::{
    running::process_operation::{OperationProcessingType, process_operation},
    *,
};
use chrono::Utc;
// use rand::seq::IndexedRandom;
use std::sync::Arc;
use tokio::sync::mpsc;

// Add automatic operations here as well that finish immediatelly, god for setting some values, triggering robot moves etc.

// DONE (correctness + PERF): this used to take `&State` and write its own
// effects straight to Redis, inside the caller's `for t in
// &model.auto_transitions` loop. Two problems, one fix:
//   - every transition in the loop was evaluated against the *same* snapshot,
//     so a transition whose guard depends on a variable an earlier transition
//     had just written did not see it until the next tick. A chain of N auto
//     transitions therefore took N ticks - 50 ms each - instead of one. Worse,
//     two transitions writing the same variable both decided what to write from
//     the same stale read, so the later one silently overwrote the earlier.
//   - each firing transition issued its own `set_state`, so three transitions
//     firing on one tick meant three sequential MSETs.
// It now applies its actions to a single `State` threaded through the loop,
// which the caller diffs and writes once. Transitions see each other's effects
// within the tick, in model order.
// DONE: PERF: `transition.to_owned().eval(..)` and the two `transition.clone()`
// calls used to deep-copy the whole transition (guard predicate tree, runner
// guard, both action vectors) three times per evaluation - and the first copy
// happened even for transitions whose guard is false, which is the
// overwhelmingly common case. `Predicate::eval`/`Transition::eval` take `&self`
// and the take path uses `take_mut`, so the common no-fire path now allocates
// nothing at all.
// PERF: `nanoid!` and the `format!` for the unique name are computed before the
// guard result is used elsewhere; they are only needed when the transition
// actually fires, which is already the case here - but the `TransitionMsg` with
// its `format!("Executed.")` and `Utc::now()` is built even when nobody
// consumes the log. Consider gating on `log::log_enabled!`.
fn process_transition(
    transition: &Transition,
    state: &mut State,
    // logging_tx: mpsc::Sender<LogMsg>,
    log_target: &str,
) {
    if !transition.eval(state, &log_target) {
        return;
    }

    // DONE: PERF: this used to clone the whole transition twice - once to rename
    // it for the log line, once more for the by-value `take` - even though the
    // rename only ever feeds a `format!`. The name is now built as a plain
    // `String` and the actions are applied in place on one state copy.
    let unique_id = nanoid::nanoid!(10, &NANOID_ALPHABET);
    let unique_name = format!("{}_{}", transition.name, unique_id);

    transition.take_mut(state, &log_target);
    log::info!(target: &log_target, "Executed auto transition: '{}'.", unique_name);

    // let transition_msg = TransitionMsg {
    //     transition_name: transition.name.clone(),
    //     timestamp: Utc::now(),
    //     severity: log::Level::Info,
    //     log: format!("Executed."),
    // };
    // let log_msg = LogMsg::TransitionMsg(transition_msg);
    // match logging_tx.send(log_msg).await {
    //     Ok(()) => (),
    //     Err(e) => log::error!(target: &log_target, "Failed to send logging with: {e}."),
    // }
}

// PERF: this runner is already doing the right thing on the read side - it
// precomputes `keys` once and uses `get_state_for_keys`.
// DONE: PERF: the `keys` vector was not deduplicated, so transitions sharing
// variables made the per-tick `MGET` send the same key several times.
//
// DONE: PERF: `let model = model.clone()` deep-copied the entire model into the
// task on top of the copy `main_runner` already made for it. `main_runner` now
// holds one `Arc<Model>` and this borrows from it.
// PERF: at 50 ms this is the fastest-ticking runner and therefore the biggest
// contributor to idle CPU. If the auto transitions only react to variables
// written by other runners, a keyspace-notification subscription on `keys`
// would let it sleep entirely when nothing changes.
pub async fn auto_transition_runner(
    name: &str,
    model: &Model,
    connection_manager: &Arc<ConnectionManager>,
    // logging_tx: mpsc::Sender<LogMsg>,
) -> Result<(), Box<dyn std::error::Error>> {
    initialize_env_logger();
    let mut interval = runner_interval();
    let log_target = format!("{}_auto_transition_runner", name);
    let keys: Vec<String> = normalize_keys(
        model
            .auto_transitions
            .iter()
            .flat_map(|t| t.get_all_var_keys())
            .collect(),
    );

    log::info!(target: &log_target, "Online.");

    // PERF: one long-lived connection handle for the whole runner instead of
    // re-fetching one every tick, and no pre-flight PING before the real work.
    // `SPConnection` is cheap to clone, multiplexed and self-healing, so this
    // handle stays valid across reconnects; a dropped socket now surfaces as an
    // error on the command itself, which the callee already logs and skips.
    let mut con = connection_manager.get_connection().await;

    loop {
        interval.tick().await;
        let state = match StateManager::get_state_for_keys(&mut con, &keys, &log_target).await {
            Some(s) => s,
            None => continue,
        };

        // One `State` threaded through every transition, so each sees what the
        // ones before it did, and one write for the whole tick.
        let mut new_state = state.clone();
        for t in &model.auto_transitions {
            process_transition(t, &mut new_state, &log_target);
        }

        let modified_state = state.get_diff_partial_state(&new_state);
        if !modified_state.state.is_empty() {
            StateManager::set_state(&mut con, &modified_state).await;
        }
    }
}

// PERF: same shape as `sop_runner`. Specifically:
//   1. DONE: `get_full_state` (a blocking `KEYS *` + `MGET` of the whole
//      database every 200 ms) is replaced with `get_state_for_keys`. The key
//      set is `model.auto_operations`/`mutexed_auto_operations`
//      `get_all_var_keys()` plus their template name variables, unioned with
//      the bookkeeping keys of whatever is currently in
//      `active_auto_ops`/`active_mutexed_op` - see `running::runner_keys`. It
//      is rebuilt only when the active set changes.
//   2. `for op in &model.auto_operations { if op.eval(&state, ..) }` evaluates
//      every auto operation's preconditions every tick. `Operation::eval` first
//      checks `state.get_value(&self.name, ..)` and only then the guards - but
//      because `get_value` clones the whole map (see `State::get_value`), even
//      the cheap rejection path is expensive. Fixing `get_value` makes this
//      loop nearly free; beyond that, indexing operations by the variables
//      their guards read would let you skip operations whose inputs did not
//      change since the last tick.
//   3. `active_auto_ops` is consumed and rebuilt (`next_active_auto_ops`) every
//      tick, cloning each `Operation` in and out. `Vec::retain` with the
//      state check, or holding `Arc<Operation>`, avoids the copying.
//   4. `format!("{}", current_active_op.name)` is an allocating no-op copy of a
//      `String` that is already owned - pass `&current_active_op.name`.
//   5. The tail does `set_state` then `remove_sp_values` twice: three
//      sequential round trips that should be one pipeline.
//   6. DONE: `let model = model.clone()` - see the note on
//      `auto_transition_runner`; `main_runner` holds one `Arc<Model>` now.
pub async fn auto_operation_runner(
    sp_id: &str,
    model: &Model,
    // logging_tx: mpsc::Sender<LogMsg>,
    connection_manager: &Arc<ConnectionManager>,
) -> Result<(), Box<dyn std::error::Error>> {
    initialize_env_logger();
    let mut interval = runner_interval();
    let log_target = format!("{}_operation_runner", sp_id);

    let mut active_auto_ops: Vec<Operation> = vec![];
    let mut active_mutexed_op: Option<Operation> = None;
    let mut terminated_operations: Vec<String> = vec![];

    // See the note on `sop_runner`: the static part comes from the model, the
    // dynamic part is the bookkeeping variables of the operations currently in
    // `active_auto_ops` / `active_mutexed_op`, whose names only exist once they
    // have been activated with a `nanoid` suffix.
    let static_keys = auto_operation_runner_static_keys(sp_id, &model);
    let mut keys = static_keys.clone();
    let read_full_state = read_full_state_enabled();
    if read_full_state {
        log::warn!(target: &log_target, "MICRO_SP_READ_FULL_STATE is set: reading the whole keyspace every tick.");
    }

    // PERF: one long-lived connection handle for the whole runner instead of
    // re-fetching one every tick, and no pre-flight PING before the real work.
    // `SPConnection` is cheap to clone, multiplexed and self-healing, so this
    // handle stays valid across reconnects; a dropped socket now surfaces as an
    // error on the command itself, which the callee already logs and skips.
    let mut con = connection_manager.get_connection().await;

    // Real time between ticks; see the note in `process_operation`.
    let mut tick_clock = TickClock::new();

    loop {
        interval.tick().await;
        let tick_elapsed_ms = tick_clock.elapsed_ms();

        let read = match read_full_state {
            true => StateManager::get_full_state(&mut con).await,
            false => StateManager::get_state_for_keys(&mut con, &keys, &log_target).await,
        };
        let state = match read {
            Some(s) => s,
            None => continue,
        };

        let mut new_state = state.clone();
        let mut new_op_ids = vec![];

        for op in &model.auto_operations {
            if op.eval(&state, &log_target) {
                let prefix = format!("{}_", op.name);
                if !active_auto_ops.iter().any(|a| a.name.starts_with(&prefix)) {
                    let unique_id = nanoid::nanoid!(10, &NANOID_ALPHABET);
                    let unique_op_id = format!("{}{}", prefix, unique_id);
                    let mut op_mut = op.clone();
                    op_mut.name = unique_op_id.clone();
                    active_auto_ops.push(op_mut);
                    new_op_ids.push(unique_op_id);
                }
            }
        }

        if active_mutexed_op.is_none() {
            for op in &model.mutexed_auto_operations {
                if op.eval(&state, &log_target) {
                    let prefix = format!("{}_", op.name);
                    let unique_id = nanoid::nanoid!(10, &NANOID_ALPHABET);
                    let unique_op_id = format!("{}{}", prefix, unique_id);
                    let mut op_mut = op.clone();
                    op_mut.name = unique_op_id.clone();
                    
                    active_mutexed_op = Some(op_mut);
                    new_op_ids.push(unique_op_id);
                    
                    break;
                }
            }
        }

        if !new_op_ids.is_empty() {
            new_state =
                add_operation_meta_tracking_variables(&new_op_ids, &new_state, false, &log_target);
            new_state = add_operation_state_tracking_variable(&new_op_ids, &new_state, &log_target);
        }

        let mut next_active_auto_ops = vec![];
        for current_active_op in active_auto_ops {
            new_state = process_operation(
                &sp_id,
                new_state,
                &current_active_op,
                OperationProcessingType::Automatic,
                None,
                None,
                // logging_tx.clone(),
                tick_elapsed_ms,
                &log_target,
            )
            .await;

            let operation_state = new_state.get_string_or_default_to_unknown(
                &format!("{}", current_active_op.name),
                &log_target,
            );

            match OperationState::from_str(&operation_state) {
                OperationState::Terminated(_) => {
                    terminated_operations.push(current_active_op.name.clone());
                }
                _ => next_active_auto_ops.push(current_active_op),
            };
        }
        active_auto_ops = next_active_auto_ops;

        let mut next_active_mutexed_op = None;
        if let Some(current_active_op) = active_mutexed_op {
            new_state = process_operation(
                &sp_id,
                new_state,
                &current_active_op,
                OperationProcessingType::Automatic,
                None,
                None,
                // logging_tx.clone(),
                tick_elapsed_ms,
                &log_target,
            )
            .await;

            let operation_state = new_state.get_string_or_default_to_unknown(
                &format!("{}", current_active_op.name),
                &log_target,
            );

            match OperationState::from_str(&operation_state) {
                OperationState::Terminated(_) => {
                    terminated_operations.push(current_active_op.name.clone());
                }
                _ => {
                    next_active_mutexed_op = Some(current_active_op);
                }
            };
        }
        active_mutexed_op = next_active_mutexed_op;

        let modified_state = state.get_diff_partial_state_and_add_missing(&new_state);
        let active_set_changed = !new_op_ids.is_empty() || !terminated_operations.is_empty();

        // DONE: PERF: this was `set_state` then two `remove_sp_values`, three
        // sequential round trips each awaited before the next could be sent.
        // One pipeline now.
        let mut terminated_operations_meta = vec![];
        for op in &terminated_operations {
            terminated_operations_meta.push(format!("{}_information", op));
            terminated_operations_meta.push(format!("{}_failure_retry_counter", op));
            terminated_operations_meta.push(format!("{}_timeout_retry_counter", op));
            terminated_operations_meta.push(format!("{}_elapsed_executing_ms", op));
            terminated_operations_meta.push(format!("{}_elapsed_disabled_ms", op));
        }
        StateManager::apply(
            &mut con,
            &modified_state,
            &[&terminated_operations, &terminated_operations_meta],
        )
        .await;
        terminated_operations.clear();

        // Operations were activated and/or terminated this tick, so the set of
        // bookkeeping variables to read from the next tick on has changed.
        if active_set_changed {
            let active: Vec<String> = active_auto_ops
                .iter()
                .chain(active_mutexed_op.iter())
                .map(|op| op.name.clone())
                .collect();
            keys = keys_with_active_operations(&static_keys, &active);
        }
    }
}
/// The two automatic runners, driven end to end against a real Redis.
///
/// `auto_transition_runner` fires guard-less state rewrites; `auto_operation_runner`
/// activates an operation instance whenever a template's guard becomes true,
/// drives it, and deletes its bookkeeping variables once it terminates. Neither
/// is reachable without Redis, and the activation/teardown bookkeeping - which
/// is where the "operations accumulate in the keyspace" class of bug lives - had
/// no coverage at all.
#[cfg(test)]
mod runner_tests {
    use super::*;
    use serial_test::serial;
    use std::time::Duration;
    use testcontainers::{ContainerAsync, ImageExt, core::ContainerPort, runners::AsyncRunner};
    use testcontainers_modules::redis::Redis;

    const SP: &str = "sp";
    const TARGET: &str = "test";

    async fn redis() -> (ContainerAsync<Redis>, Arc<ConnectionManager>) {
        let container = Redis::default()
            .with_mapped_port(6379, ContainerPort::Tcp(6379))
            .start()
            .await
            .unwrap();
        let manager = Arc::new(ConnectionManager::new().await);
        let mut con = manager.get_connection().await;
        StateManager::flush_state(&mut con).await;
        (container, manager)
    }

    fn flags(names: &[&str]) -> State {
        let mut state = State::new();
        for name in names {
            state.add_mut(
                SPAssignment::new(SPVariable::new(name, SPValueType::Bool), false.to_spvalue()),
                TARGET,
            );
        }
        state
    }

    /// An operation that sets `flag` on start and completes once it is set.
    fn auto_op(name: &str, flag: &str, state: &State) -> Operation {
        Operation::new(
            name,
            Some(10_000),
            Some(10_000),
            None,
            None,
            false,
            vec![Transition::parse(
                "start",
                &format!("var:{flag} == false"),
                "true",
                vec![format!("var:{flag} <- true").as_str()],
                Vec::<&str>::new(),
                state,
            )],
            vec![Transition::parse(
                "complete",
                &format!("var:{flag} == true"),
                "true",
                Vec::<&str>::new(),
                Vec::<&str>::new(),
                state,
            )],
            vec![],
            vec![],
            vec![],
            vec![],
        )
    }

    async fn deploy(manager: &Arc<ConnectionManager>, model: &Model, domain: State) {
        let mut state = generate_runner_state_variables(SP, 0, TARGET);
        state.extend_mut(generate_operation_state_variables(model, false, TARGET), true);
        state.extend_mut(domain, true);
        state.add_mut(
            SPAssignment::new(
                SPVariable::new(&format!("{SP}_dashboard_command"), SPValueType::String),
                "none".to_spvalue(),
            ),
            TARGET,
        );
        let mut con = manager.get_connection().await;
        StateManager::set_state(&mut con, &state).await;
    }

    fn spawn_transitions(
        manager: &Arc<ConnectionManager>,
        model: Model,
    ) -> tokio::task::JoinHandle<()> {
        let manager = Arc::clone(manager);
        tokio::spawn(async move {
            let _ = auto_transition_runner(SP, &model, &manager).await;
        })
    }

    fn spawn_operations(
        manager: &Arc<ConnectionManager>,
        model: Model,
    ) -> tokio::task::JoinHandle<()> {
        let manager = Arc::clone(manager);
        tokio::spawn(async move {
            let _ = auto_operation_runner(SP, &model, &manager).await;
        })
    }

    async fn wait_true(con: &mut SPConnection, key: &str, ms: u64) -> bool {
        let deadline = std::time::Instant::now() + Duration::from_millis(ms);
        while std::time::Instant::now() < deadline {
            if StateManager::get_sp_value(con, key).await == Some(true.to_spvalue()) {
                return true;
            }
            tokio::time::sleep(Duration::from_millis(2)).await;
        }
        false
    }

    /// The chain-in-one-tick property, this time through Redis rather than
    /// against `process_transition` directly: three dependent transitions settle
    /// without costing three ticks.
    #[tokio::test]
    #[serial]
    async fn a_chain_of_auto_transitions_settles() {
        let (_container, manager) = redis().await;
        let mut con = manager.get_connection().await;
        let domain = flags(&["a", "b", "c"]);

        let link = |name: &str, needs: Option<&str>, sets: &str| {
            let guard = match needs {
                Some(needs) => format!("var:{needs} == true && var:{sets} == false"),
                None => format!("var:{sets} == false"),
            };
            Transition::parse(
                name,
                &guard,
                "true",
                vec![format!("var:{sets} <- true").as_str()],
                Vec::<&str>::new(),
                &domain,
            )
        };

        let model = Model::new(
            SP,
            vec![
                link("first", None, "a"),
                link("second", Some("a"), "b"),
                link("third", Some("b"), "c"),
            ],
            vec![],
            vec![],
            vec![],
            vec![],
        );
        deploy(&manager, &model, domain).await;

        let runner = spawn_transitions(&manager, model);
        let settled = wait_true(&mut con, "c", 3000).await;
        runner.abort();

        assert!(settled, "the whole chain should have fired");
        for flag in ["a", "b", "c"] {
            assert_eq!(
                StateManager::get_sp_value(&mut con, flag).await,
                Some(true.to_spvalue())
            );
        }
    }

    /// Once every guard is false the runner has nothing to do, and must stop
    /// writing. An auto transition runner that keeps writing is the worst case
    /// for idle load, since it is the fastest-ticking runner.
    #[tokio::test]
    #[serial]
    async fn a_settled_auto_transition_runner_writes_nothing() {
        let (_container, manager) = redis().await;
        let mut con = manager.get_connection().await;
        let domain = flags(&["a"]);

        let model = Model::new(
            SP,
            vec![Transition::parse(
                "once",
                "var:a == false",
                "true",
                vec!["var:a <- true"],
                Vec::<&str>::new(),
                &domain,
            )],
            vec![],
            vec![],
            vec![],
            vec![],
        );
        deploy(&manager, &model, domain).await;

        let runner = spawn_transitions(&manager, model);
        assert!(wait_true(&mut con, "a", 3000).await);

        let before = StateManager::get_full_state(&mut con).await.unwrap();
        tokio::time::sleep(Duration::from_millis(300)).await;
        let after = StateManager::get_full_state(&mut con).await.unwrap();

        assert!(!runner.is_finished());
        runner.abort();
        assert!(
            before.get_diff_partial_state(&after).state.is_empty(),
            "a settled auto transition runner must not write"
        );
    }

    /// An auto operation activates on its own, runs, terminates, and has its
    /// bookkeeping variables deleted again - so a system that runs the same auto
    /// operation a thousand times does not grow a thousand key sets.
    #[tokio::test]
    #[serial]
    async fn an_auto_operation_activates_runs_and_cleans_up_after_itself() {
        let (_container, manager) = redis().await;
        let mut con = manager.get_connection().await;
        let domain = flags(&["a"]);

        let model = Model::new(
            SP,
            vec![],
            vec![auto_op("do_it", "a", &domain)],
            vec![],
            vec![],
            vec![],
        );
        deploy(&manager, &model, domain).await;

        let before: i64 = redis::cmd("DBSIZE").query_async(&mut con).await.unwrap();

        let runner = spawn_operations(&manager, model);
        assert!(
            wait_true(&mut con, "a", 3000).await,
            "the operation should have started on its own"
        );

        // Wait for the keyspace to come back to where it started, which only
        // happens once the operation has terminated and been cleaned up.
        let deadline = std::time::Instant::now() + Duration::from_millis(3000);
        let mut after = before + 1;
        while std::time::Instant::now() < deadline {
            after = redis::cmd("DBSIZE").query_async(&mut con).await.unwrap();
            if after == before {
                break;
            }
            tokio::time::sleep(Duration::from_millis(5)).await;
        }
        assert!(!runner.is_finished());
        runner.abort();

        assert_eq!(
            after, before,
            "the terminated operation's variables should have been deleted"
        );
    }

    /// Every auto operation whose guard holds runs; they are not mutually
    /// exclusive.
    #[tokio::test]
    #[serial]
    async fn auto_operations_run_concurrently() {
        let (_container, manager) = redis().await;
        let mut con = manager.get_connection().await;
        let domain = flags(&["a", "b"]);

        let model = Model::new(
            SP,
            vec![],
            vec![auto_op("one", "a", &domain), auto_op("two", "b", &domain)],
            vec![],
            vec![],
            vec![],
        );
        deploy(&manager, &model, domain).await;

        let runner = spawn_operations(&manager, model);
        assert!(wait_true(&mut con, "a", 3000).await);
        assert!(wait_true(&mut con, "b", 3000).await);
        runner.abort();
    }

    /// The mutexed set is the opposite: at most one of them runs at a time.
    #[tokio::test]
    #[serial]
    async fn only_one_mutexed_auto_operation_runs_at_a_time() {
        let (_container, manager) = redis().await;
        let mut con = manager.get_connection().await;
        let domain = flags(&["a", "b"]);

        // Neither can ever complete, so whichever starts holds the mutex.
        let stuck = |name: &str, flag: &str| {
            Operation::new(
                name,
                Some(10_000),
                Some(10_000),
                None,
                None,
                false,
                vec![Transition::parse(
                    "start",
                    &format!("var:{flag} == false"),
                    "true",
                    vec![format!("var:{flag} <- true").as_str()],
                    Vec::<&str>::new(),
                    &domain,
                )],
                vec![Transition::parse(
                    "never",
                    "false",
                    "true",
                    Vec::<&str>::new(),
                    Vec::<&str>::new(),
                    &domain,
                )],
                vec![],
                vec![],
                vec![],
                vec![],
            )
        };

        let model = Model::new(
            SP,
            vec![],
            vec![],
            vec![stuck("one", "a"), stuck("two", "b")],
            vec![],
            vec![],
        );
        deploy(&manager, &model, domain).await;

        let runner = spawn_operations(&manager, model);
        assert!(wait_true(&mut con, "a", 3000).await, "the first should start");
        tokio::time::sleep(Duration::from_millis(300)).await;
        runner.abort();

        assert_eq!(
            StateManager::get_sp_value(&mut con, "b").await,
            Some(false.to_spvalue()),
            "the second mutexed operation must wait for the first to finish"
        );
    }

    /// An auto operation whose guard never holds costs nothing.
    #[tokio::test]
    #[serial]
    async fn an_auto_operation_runner_with_nothing_to_do_writes_nothing() {
        let (_container, manager) = redis().await;
        let mut con = manager.get_connection().await;
        let mut domain = flags(&["a"]);
        // Already true, so the guard `var:a == false` never holds.
        domain = domain.update("a", true.to_spvalue());

        let model = Model::new(
            SP,
            vec![],
            vec![auto_op("never", "a", &domain)],
            vec![],
            vec![],
            vec![],
        );
        deploy(&manager, &model, domain).await;

        let runner = spawn_operations(&manager, model);
        tokio::time::sleep(Duration::from_millis(100)).await;

        let before = StateManager::get_full_state(&mut con).await.unwrap();
        tokio::time::sleep(Duration::from_millis(300)).await;
        let after = StateManager::get_full_state(&mut con).await.unwrap();

        assert!(!runner.is_finished());
        runner.abort();
        assert!(
            before.get_diff_partial_state(&after).state.is_empty(),
            "an idle auto operation runner must not write: {:?}",
            before.get_diff_partial_state(&after)
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const TARGET: &str = "test";

    fn state() -> State {
        let mut state = State::new();
        for name in ["a", "b", "c"] {
            state.add_mut(
                SPAssignment::new(SPVariable::new(name, SPValueType::Bool), false.to_spvalue()),
                TARGET,
            );
        }
        state
    }

    fn link(name: &str, needs: Option<&str>, sets: &str, state: &State) -> Transition {
        let guard = match needs {
            Some(needs) => format!("var:{} == true && var:{} == false", needs, sets),
            None => format!("var:{} == false", sets),
        };
        Transition::parse(
            name,
            &guard,
            "true",
            vec![format!("var:{} <- true", sets).as_str()],
            Vec::<&str>::new(),
            state,
        )
    }

    /// The bug: every transition was evaluated against the same snapshot, so a
    /// chain of three took three ticks - one link per tick - instead of one.
    #[test]
    fn a_chain_of_transitions_completes_within_one_tick() {
        let state = state();
        let transitions = vec![
            link("first", None, "a", &state),
            link("second", Some("a"), "b", &state),
            link("third", Some("b"), "c", &state),
        ];

        let mut new_state = state.clone();
        for t in &transitions {
            process_transition(t, &mut new_state, TARGET);
        }

        for name in ["a", "b", "c"] {
            assert_eq!(
                new_state.get_value(name, TARGET),
                Some(true.to_spvalue()),
                "'{name}' should have been set in the same tick"
            );
        }
    }

    /// Order still matters within a tick: a link whose predecessor comes later
    /// in the model does not fire until the next tick, as before.
    #[test]
    fn a_chain_listed_backwards_still_advances_one_link_per_tick() {
        let state = state();
        let transitions = vec![
            link("third", Some("b"), "c", &state),
            link("second", Some("a"), "b", &state),
            link("first", None, "a", &state),
        ];

        let mut new_state = state.clone();
        for t in &transitions {
            process_transition(t, &mut new_state, TARGET);
        }
        assert_eq!(new_state.get_value("a", TARGET), Some(true.to_spvalue()));
        assert_eq!(new_state.get_value("b", TARGET), Some(false.to_spvalue()));
        assert_eq!(new_state.get_value("c", TARGET), Some(false.to_spvalue()));
    }

    /// A transition whose guard is false must leave the state untouched.
    #[test]
    fn a_transition_that_does_not_fire_changes_nothing() {
        let state = state();
        let blocked = link("blocked", Some("a"), "b", &state);

        let mut new_state = state.clone();
        process_transition(&blocked, &mut new_state, TARGET);

        assert_eq!(new_state, state);
        assert!(state.get_diff_partial_state(&new_state).state.is_empty());
    }
}
