//! Executing [`SOPStruct`](crate::SOPStruct)s - the scripted procedures a model ships with.
//!
//! The SOP runner picks up whichever SOP the state points at, walks its
//! sequence/parallel/alternative tree, and drives the operations it contains
//! through [`process_operation`](crate::running::process_operation) until the
//! whole procedure completes, fails or is cancelled.

use crate::*;
use log::Level;
use crate::SPConnection;
use std::sync::Arc;

/// Runs the SOP executor until the process ends.
///
/// On every tick it reads the SOP keys for `sp_id` from Redis, starts the SOP
/// named in `{sp_id}_sop_id` once `{sp_id}_sop_enabled` is set, advances its
/// operations, and writes back `{sp_id}_sop_state`, `{sp_id}_sop_current_step`
/// and the per-operation state. `model` supplies the SOPs to look up,
/// `connection_manager` the shared Redis connection; log output goes to the
/// `{sp_id}_sop_runner` target.
pub async fn sop_runner(
    sp_id: &str,
    model: &Model,
    connection_manager: &Arc<ConnectionManager>,
) -> Result<(), Box<dyn std::error::Error>> {
    initialize_env_logger();
    activity_log::init_from_env();
    let mut interval = runner_interval();
    let log_target = &format!("{}_sop_runner", sp_id);

    log::info!(target: log_target, "Online.");

    let mut active_unique_sop_id: Option<String> = None;
    let mut active_unique_sop_state: SOPState = SOPState::Initial;
    let mut active_sop_container: Option<SOP> = None;

    // The variables read every tick no matter what is running, and the set
    // actually requested from Redis. The latter grows with the bookkeeping
    // variables of a SOP's operations while that SOP is active - their names
    // only exist once `uniquify_sop_operations` has run, which is why the set
    // is rebuilt there rather than computed once here.
    let static_keys = sop_runner_static_keys(sp_id, model);
    let mut keys = static_keys.clone();
    let read_full_state = read_full_state_enabled();
    if read_full_state {
        log::warn!(target: log_target, "MICRO_SP_READ_FULL_STATE is set: reading the whole keyspace every tick.");
    }

    let mut con = connection_manager.get_connection().await;

    // Real time between ticks. `process_operation` advances the elapsed
    // counters by this rather than by a compile-time constant, which is what
    // made SOP operations - driven at 100 ms by a constant of 200 - time out at
    // half their configured deadline.
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
        let mut sop_state =
            state.get_string_or_default_to_unknown(&format!("{}_sop_state", sp_id), &log_target);

        let sop_enabled =
            state.get_bool_or_default_to_false(&format!("{}_sop_enabled", sp_id), &log_target);

        let sop_id =
            state.get_string_or_default_to_unknown(&format!("{}_sop_id", sp_id), &log_target);

        let Some(sop_template) = model.sops.iter().find(|s| s.id == sop_id) else {
            log::debug!(target: &log_target, "SOP with id '{}' not found in model. Skipping evaluation.", sop_id);
            continue;
        };

        let old_sop_information = new_state
            .get_string_or_default_to_unknown(&format!("{}_sop_information", sop_id), &log_target);

        let mut new_sop_info: String; // = old_sop_information.clone();
        let mut sop_info_level: Level = log::Level::Info;


        // Snapshotted for the file log: the arms below drive
        // `active_unique_sop_state` forward, and comparing against this after
        // the match is what turns "the runner is in state X" into "the SOP
        // moved from X to Y", which is the thing worth a line.
        let sop_state_before = active_unique_sop_state.clone();
        let sop_id_before = active_unique_sop_id.clone();

        // Check first if there is an active unique SOP already running
        match active_unique_sop_id {
            None => {
                if sop_enabled {
                    let unique_sop_id = nanoid::nanoid!(10, &NANOID_ALPHABET);
                    active_unique_sop_id = Some(format!("{}_{}", sop_template.id, unique_sop_id));

                    let unique_sop = uniquify_sop_operations(sop_template.sop.clone());
                    active_sop_container = Some(unique_sop.clone());
                    let ops_in_sop = get_all_operations_from_sop(&unique_sop);
                    let op_names: Vec<String> =
                        ops_in_sop.iter().map(|x| x.name.clone()).collect();

                    // The bookkeeping variables of these operations are created
                    // in `new_state` below and written out by the diff at the
                    // end of this tick; from the next tick on they have to be
                    // read back, so the key set grows with them here.
                    keys = keys_with_active_operations(&static_keys, &op_names);

                    new_state = add_operation_meta_tracking_variables(
                        &op_names,
                        &new_state,
                        false,
                        &log_target,
                    );
                    new_state =
                        add_operation_state_tracking_variable(&op_names, &new_state, &log_target);
                    new_sop_info = format!("SOP '{sop_id}' is enabled, starting execution.");
                    sop_info_level = log::Level::Info;
                    new_state =
                        new_state.update(&format!("{}_sop_enabled", sp_id), false.to_spvalue());
                } else {
                    continue;
                }
            }
            Some(ref active_sop) => match active_unique_sop_state {
                SOPState::Initial => {
                    active_unique_sop_state = SOPState::Executing;
                    new_sop_info = format!(
                        "Initializing a new SOP '{}':\n{}",
                        active_sop,
                        visualize_sop(active_sop_container.as_ref().unwrap())
                    );
                }
                SOPState::Executing => {
                    // Inform the operation that the sop is executing
                    sop_state = SOPState::Executing.to_string();
                    let con_clone = con.clone();
                    new_sop_info = format!("Executing SOP '{active_sop}'.");
                    sop_info_level = log::Level::Info;
                    new_state = process_sop_node_tick(
                        sp_id,
                        new_state,
                        active_sop_container.as_ref().unwrap(),
                        con_clone,
                        tick_elapsed_ms,
                        &log_target,
                    )
                    .await;

                    let calculated_root_state = active_sop_container
                        .as_ref()
                        .unwrap()
                        .get_state(&new_state, &log_target);

                    if calculated_root_state != SOPState::Executing {
                        new_sop_info = format!("Completing SOP '{active_sop}'.");
                        sop_info_level = log::Level::Info;

                        active_unique_sop_state = calculated_root_state;
                    }
                }
                SOPState::Fatal => {
                    new_sop_info = format!("Fataled SOP '{active_sop}'.");
                    sop_info_level = log::Level::Error;
                    active_unique_sop_state = SOPState::Initial;
                    // Inform the operation that the sop has failed:
                    sop_state = SOPState::Fatal.to_string();

                    if let Some(unique_sop) = active_sop_container {
                        let con_clone = con.clone();
                        remove_operations_from_state(active_sop, &unique_sop, con_clone).await;
                    }

                    active_sop_container = None;
                    active_unique_sop_id = None;
                    // Those operation variables have just been deleted from
                    // Redis, so stop asking for them.
                    keys = static_keys.clone();
                }
                SOPState::Completed => {
                    new_sop_info = format!("Completed SOP '{active_sop}'.");
                    sop_info_level = log::Level::Info;
                    active_unique_sop_state = SOPState::Initial;
                    // Inform the operation that the sop has completed:
                    sop_state = SOPState::Completed.to_string();

                    if let Some(unique_sop) = active_sop_container {
                        let con_clone = con.clone();
                        remove_operations_from_state(active_sop, &unique_sop, con_clone).await;
                    }

                    active_sop_container = None;
                    active_unique_sop_id = None;
                    // Those operation variables have just been deleted from
                    // Redis, so stop asking for them.
                    keys = static_keys.clone();
                }
                SOPState::Cancelled => {
                    new_sop_info = format!("Cancelled SOP '{active_sop}'.");
                    sop_info_level = log::Level::Warn;
                    active_unique_sop_state = SOPState::Initial;
                    // Inform the operation that the sop has ben cancelled:
                    sop_state = SOPState::Cancelled.to_string();

                    if let Some(unique_sop) = active_sop_container {
                        let con_clone = con.clone();
                        remove_operations_from_state(active_sop, &unique_sop, con_clone).await;
                    }

                    active_sop_container = None;
                    active_unique_sop_id = None;
                    // Those operation variables have just been deleted from
                    // Redis, so stop asking for them.
                    keys = static_keys.clone();
                }
                SOPState::UNKNOWN => {
                    new_sop_info = format!("SOP '{active_sop}' state id UNKNOWN.");
                    sop_info_level = log::Level::Info;
                    active_unique_sop_state = SOPState::Initial;
                    active_sop_container = None;
                    active_unique_sop_id = None;
                    keys = static_keys.clone();
                }
            },
        }

        if new_sop_info != old_sop_information {
            match sop_info_level {
                log::Level::Info => log::info!(target: &log_target, "{}", new_sop_info),
                log::Level::Warn => log::warn!(target: &log_target, "{}", new_sop_info),
                log::Level::Error => log::error!(target: &log_target, "{}", new_sop_info),
                _ => (),
            }
        }

        // A `SOP` line whenever the runner actually moved: either the tracked
        // state changed, or a new unique SOP was activated / an old one torn
        // down (both of which change `active_unique_sop_id` while the state can
        // stay put). Guarding on a real change is what keeps this off the
        // per-tick path - `Executing` re-enters its arm every 100 ms.
        if sop_state_before != active_unique_sop_state || sop_id_before != active_unique_sop_id {
            let subject = active_unique_sop_id
                .as_deref()
                .or(sop_id_before.as_deref())
                .unwrap_or(&sop_id);
            // Activation and teardown both leave the tracked state untouched
            // (a SOP is activated *into* `Initial`, and released back *to* it),
            // so without this note those two lines would read as a meaningless
            // "initial -> initial" and "completed -> initial".
            let note = match (sop_id_before.is_some(), active_unique_sop_id.is_some()) {
                (false, true) => "activated",
                (true, false) => "released",
                _ => "",
            };
            activity_log::log_sop(
                &log_target,
                subject,
                &sop_state_before.to_string(),
                &active_unique_sop_state.to_string(),
                note,
            );
        }

        new_state = new_state
            .update(
                &format!("{}_sop_information", sop_id),
                new_sop_info.to_spvalue(),
            )
            .update(&format!("{}_sop_state", sp_id), sop_state.to_spvalue());

        let modified_state = state.get_diff_partial_state_and_add_missing(&new_state);

        if !modified_state.state.is_empty() {
            activity_log::log_state_diff(&log_target, &state, &modified_state);
            StateManager::set_state(&mut con, &modified_state).await;
        }

    }
}

async fn remove_operations_from_state(sop_id: &str, unique_sop: &SOP, mut con: SPConnection) {
    let ops_in_sop = get_all_operations_from_sop(&unique_sop);
    let mut op_ids_meta = vec![];
    let sop_id = format!("op_{}", sop_id);
    log::debug!(target: "sop_runner", "Removing operation variables for '{}'.", sop_id);
    let mut op_ids = ops_in_sop
        .iter()
        .map(|x| x.name.to_string())
        .collect::<Vec<String>>();
    op_ids.push(sop_id.clone());
    for op in &op_ids {
        op_ids_meta.push(format!("{}_information", op));
        op_ids_meta.push(format!("{}_failure_retry_counter", op));
        op_ids_meta.push(format!("{}_timeout_retry_counter", op));
        op_ids_meta.push(format!("{}_elapsed_executing_ms", op));
        op_ids_meta.push(format!("{}_elapsed_disabled_ms", op));
    }

    StateManager::apply(&mut con, &State::new(), &[&op_ids, &op_ids_meta]).await;
}

async fn process_sop_node_tick(
    sp_id: &str,
    mut state: State,
    sop: &SOP,
    con: crate::SPConnection,
    tick_elapsed_ms: i64,
    log_target: &str,
) -> State {
    match sop {
        SOP::Operation(operation) => {
            state = running::process_operation::process_operation(
                &sp_id,
                state,
                operation,
                running::process_operation::OperationProcessingType::SOP,
                None,
                None,
                tick_elapsed_ms,
                log_target,
            )
            .await;
        }

        SOP::Sequence(sops) => {
            let active_child = sops
                .iter()
                .find(|child| child.get_state(&state, &log_target) != SOPState::Completed);

            if let Some(child) = active_child {
                state = Box::pin(process_sop_node_tick(
                    sp_id, state, child, con, tick_elapsed_ms, log_target,
                ))
                .await;
            }
        }

        SOP::Parallel(sops) => {
            for child in sops {
                state = Box::pin(process_sop_node_tick(
                    sp_id,
                    state,
                    child,
                    con.clone(),
                    tick_elapsed_ms,
                    log_target,
                ))
                .await;
            }
        }

        SOP::Alternative(sops) => {
            let active_child = sops.iter().find(|child| {
                let child_state = child.get_state(&state, &log_target);
                child_state != SOPState::Initial && child_state != SOPState::Completed
            });

            // If a path is active, keep processing it
            if let Some(child) = active_child {
                state = Box::pin(process_sop_node_tick(
                    sp_id, state, child, con, tick_elapsed_ms, log_target,
                ))
                .await;
            } else {
                // If no path is active, find the first one that can start
                if let Some(path_to_start) = sops
                    .iter()
                    .find(|child| can_sop_start(sp_id, child, &state, log_target))
                {
                    log::info!(target: log_target, "Found valid alternative path to start.");
                    state = Box::pin(process_sop_node_tick(
                        sp_id,
                        state,
                        path_to_start,
                        con,
                        tick_elapsed_ms,
                        log_target,
                    ))
                    .await;
                }
            }
        }
    }

    state
}

fn can_sop_start(sp_id: &str, sop: &SOP, state: &State, log_target: &str) -> bool {
    match sop {
        SOP::Operation(operation) => {
            let current_state = sop.get_state(&state, &log_target);
            current_state == SOPState::Initial && operation.eval(state, log_target)
        }
        SOP::Sequence(sops) => sops.first().map_or(false, |first| {
            can_sop_start(sp_id, first, state, log_target)
        }),
        SOP::Parallel(sops) => sops
            .iter()
            .all(|child| can_sop_start(sp_id, child, state, log_target)),
        SOP::Alternative(sops) => sops
            .iter()
            .any(|child| can_sop_start(sp_id, child, state, log_target)),
    }
}

/// Renames every operation in a SOP tree to `op_{name}_{nanoid}`.
///
/// Each run of a SOP gets its own operation instances, so two runs of the same
/// procedure never share lifecycle variables.
pub fn uniquify_sop_operations(sop: SOP) -> SOP {
    match sop {
        SOP::Operation(op) => {
            let unique_id = nanoid::nanoid!(10, &NANOID_ALPHABET); // 64^10 unique ids
            let new_name = format!("op_{}_{}", op.name, unique_id);
            SOP::Operation(Box::new(Operation {
                name: new_name,
                ..*op
            }))
        }
        SOP::Sequence(sops) => {
            SOP::Sequence(sops.into_iter().map(uniquify_sop_operations).collect())
        }
        SOP::Parallel(sops) => {
            SOP::Parallel(sops.into_iter().map(uniquify_sop_operations).collect())
        }
        SOP::Alternative(sops) => {
            SOP::Alternative(sops.into_iter().map(uniquify_sop_operations).collect())
        }
    }
}

/// The SOP runner, driven end to end against a real Redis.
///
/// This is the runner with the most moving parts and it had no tests at all:
/// activation (uniquifying the tree, creating every operation's bookkeeping
/// variables, growing the read key set), the per-tick tree walk, and the three
/// teardown paths (completed / fatal / cancelled) that delete those variables
/// again. None of it is reachable without Redis, because the walk carries a
/// connection and the teardown deletes keys.
///
/// What the tests below are really checking is that a SOP *finishes* - that the
/// root node's state converges - and that the runner cleans up after itself, so
/// running the same SOP twice does not accumulate keys.
#[cfg(test)]
mod tests {
    use super::*;
    use serial_test::serial;
    use std::time::Duration;
    use testcontainers::{ContainerAsync, ImageExt, core::ContainerPort, runners::AsyncRunner};
    use testcontainers_modules::redis::Redis;

    const SP: &str = "sp";
    const TARGET: &str = "test";
    const SOP_ID: &str = "test_sop";

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

    /// The domain the SOP operates on: one boolean per step.
    fn domain(flags: &[&str]) -> State {
        let mut state = State::new();
        for flag in flags {
            state.add_mut(
                SPAssignment::new(SPVariable::new(flag, SPValueType::Bool), false.to_spvalue()),
                TARGET,
            );
        }
        state
    }

    /// An operation that sets `flag` when it starts and completes once it is
    /// set - i.e. one that always runs to completion in two ticks.
    fn step(name: &str, flag: &str, state: &State) -> SOP {
        SOP::Operation(Box::new(Operation::new(
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
        )))
    }

    /// An operation that starts but never satisfies its postcondition, times
    /// out quickly, and is allowed to be bypassed.
    fn bypassing_step(name: &str, flag: &str, state: &State) -> SOP {
        SOP::Operation(Box::new(Operation::new(
            name,
            Some(20),
            Some(10_000),
            None,
            None,
            true,
            vec![Transition::parse(
                "start",
                &format!("var:{flag} == false"),
                "true",
                vec![format!("var:{flag} <- true").as_str()],
                Vec::<&str>::new(),
                state,
            )],
            vec![Transition::parse(
                "never",
                "false",
                "true",
                Vec::<&str>::new(),
                Vec::<&str>::new(),
                state,
            )],
            vec![],
            vec![],
            vec![],
            vec![],
        )))
    }

    /// Build the model plus the full initial state the runner needs in Redis.
    async fn deploy(sop: SOP, flags: &[&str], manager: &Arc<ConnectionManager>) -> Model {
        let model = Model::new(
            SP,
            vec![],
            vec![],
            vec![],
            vec![SOPStruct {
                id: SOP_ID.to_string(),
                sop,
            }],
            vec![],
        );

        let mut state = generate_runner_state_variables(SP, 0, TARGET);
        state.extend_mut(generate_operation_state_variables(&model, false, TARGET), true);
        state.extend_mut(domain(flags), true);
        state.add_mut(
            SPAssignment::new(
                SPVariable::new(&format!("{SP}_dashboard_command"), SPValueType::String),
                "none".to_spvalue(),
            ),
            TARGET,
        );
        state = state.update(&format!("{SP}_sop_id"), SOP_ID.to_spvalue());

        let mut con = manager.get_connection().await;
        StateManager::set_state(&mut con, &state).await;
        model
    }

    fn spawn_runner(
        manager: &Arc<ConnectionManager>,
        model: Model,
    ) -> tokio::task::JoinHandle<()> {
        let manager = Arc::clone(manager);
        tokio::spawn(async move {
            let _ = sop_runner(SP, &model, &manager).await;
        })
    }

    async fn value(con: &mut SPConnection, key: &str) -> String {
        match StateManager::get_sp_value(con, key).await {
            Some(SPValue::String(StringOrUnknown::String(s))) => s,
            other => format!("{other:?}"),
        }
    }

    async fn wait_for(con: &mut SPConnection, key: &str, expected: &str, timeout_ms: u64) -> String {
        let deadline = std::time::Instant::now() + Duration::from_millis(timeout_ms);
        let mut last = String::new();
        while std::time::Instant::now() < deadline {
            last = value(con, key).await;
            if last == expected {
                return last;
            }
            tokio::time::sleep(Duration::from_millis(5)).await;
        }
        last
    }

    async fn enable_sop(con: &mut SPConnection) {
        StateManager::set_sp_value(con, &format!("{SP}_sop_enabled"), &true.to_spvalue()).await;
    }

    /// The whole point of the runner: a sequence of operations runs to
    /// completion and the SOP reports `completed`.
    #[tokio::test]
    #[serial]
    async fn a_sequence_runs_to_completion() {
        let (_container, manager) = redis().await;
        let mut con = manager.get_connection().await;

        let state = domain(&["a", "b"]);
        let sop = SOP::Sequence(vec![step("one", "a", &state), step("two", "b", &state)]);
        let model = deploy(sop, &["a", "b"], &manager).await;

        let runner = spawn_runner(&manager, model);
        enable_sop(&mut con).await;

        let sop_state = wait_for(&mut con, &format!("{SP}_sop_state"), "completed", 5000).await;
        runner.abort();

        assert_eq!(sop_state, "completed");
        assert_eq!(
            StateManager::get_sp_value(&mut con, "a").await,
            Some(true.to_spvalue()),
            "the first step's action should have run"
        );
        assert_eq!(
            StateManager::get_sp_value(&mut con, "b").await,
            Some(true.to_spvalue()),
            "the second step's action should have run"
        );
    }

    /// A `Sequence` is ordered: the second step must not run before the first
    /// has finished. Without that the whole abstraction is pointless, and the
    /// walk's "first child that is not Completed" rule is what enforces it.
    #[tokio::test]
    #[serial]
    async fn a_sequence_does_not_start_its_second_step_early() {
        let (_container, manager) = redis().await;
        let mut con = manager.get_connection().await;

        let state = domain(&["a", "b"]);
        // The first step can never complete, so the second must never run.
        let blocked = SOP::Operation(Box::new(Operation::new(
            "blocked",
            Some(10_000),
            Some(10_000),
            None,
            None,
            false,
            vec![Transition::parse(
                "start",
                "var:a == false",
                "true",
                vec!["var:a <- true"],
                Vec::<&str>::new(),
                &state,
            )],
            vec![Transition::parse(
                "never",
                "false",
                "true",
                Vec::<&str>::new(),
                Vec::<&str>::new(),
                &state,
            )],
            vec![],
            vec![],
            vec![],
            vec![],
        )));
        let sop = SOP::Sequence(vec![blocked, step("two", "b", &state)]);
        let model = deploy(sop, &["a", "b"], &manager).await;

        let runner = spawn_runner(&manager, model);
        enable_sop(&mut con).await;

        assert_eq!(
            wait_for(&mut con, "a", "true", 3000).await,
            "Some(Bool(Bool(true)))",
            "the first step should have started"
        );
        tokio::time::sleep(Duration::from_millis(300)).await;
        runner.abort();

        assert_eq!(
            StateManager::get_sp_value(&mut con, "b").await,
            Some(false.to_spvalue()),
            "the second step ran while the first was still executing"
        );
    }

    /// `Parallel` runs every branch on the same tick, so both flags are set
    /// without either branch waiting for the other.
    #[tokio::test]
    #[serial]
    async fn parallel_branches_all_run() {
        let (_container, manager) = redis().await;
        let mut con = manager.get_connection().await;

        let state = domain(&["a", "b", "c"]);
        let sop = SOP::Parallel(vec![
            step("one", "a", &state),
            step("two", "b", &state),
            step("three", "c", &state),
        ]);
        let model = deploy(sop, &["a", "b", "c"], &manager).await;

        let runner = spawn_runner(&manager, model);
        enable_sop(&mut con).await;

        let sop_state = wait_for(&mut con, &format!("{SP}_sop_state"), "completed", 5000).await;
        runner.abort();

        assert_eq!(sop_state, "completed");
        for flag in ["a", "b", "c"] {
            assert_eq!(
                StateManager::get_sp_value(&mut con, flag).await,
                Some(true.to_spvalue()),
                "branch '{flag}' did not run"
            );
        }
    }

    /// `Alternative` picks exactly one branch - the first one that can start -
    /// and leaves the others alone.
    #[tokio::test]
    #[serial]
    async fn an_alternative_takes_exactly_one_branch() {
        let (_container, manager) = redis().await;
        let mut con = manager.get_connection().await;

        let state = domain(&["a", "b"]);
        let sop = SOP::Alternative(vec![step("one", "a", &state), step("two", "b", &state)]);
        let model = deploy(sop, &["a", "b"], &manager).await;

        let runner = spawn_runner(&manager, model);
        enable_sop(&mut con).await;

        let sop_state = wait_for(&mut con, &format!("{SP}_sop_state"), "completed", 5000).await;
        runner.abort();

        assert_eq!(sop_state, "completed");
        let a = StateManager::get_sp_value(&mut con, "a").await;
        let b = StateManager::get_sp_value(&mut con, "b").await;
        assert_eq!(a, Some(true.to_spvalue()), "the first viable branch runs");
        assert_eq!(
            b,
            Some(false.to_spvalue()),
            "the second branch must not also run"
        );
    }

    /// Teardown: when the SOP finishes, the per-operation bookkeeping variables
    /// it created on activation are deleted again. Without this every SOP run
    /// would leave a permanent residue in the keyspace.
    #[tokio::test]
    #[serial]
    async fn finishing_a_sop_removes_the_operation_variables_it_created() {
        let (_container, manager) = redis().await;
        let mut con = manager.get_connection().await;

        let state = domain(&["a"]);
        let sop = SOP::Sequence(vec![step("one", "a", &state)]);
        let model = deploy(sop, &["a"], &manager).await;

        let before: i64 = redis::cmd("DBSIZE").query_async(&mut con).await.unwrap();

        let runner = spawn_runner(&manager, model);
        enable_sop(&mut con).await;
        assert_eq!(
            wait_for(&mut con, &format!("{SP}_sop_state"), "completed", 5000).await,
            "completed"
        );

        // Give the teardown tick a moment to land.
        tokio::time::sleep(Duration::from_millis(200)).await;
        runner.abort();

        let after: i64 = redis::cmd("DBSIZE").query_async(&mut con).await.unwrap();
        assert_eq!(
            before, after,
            "the uniquified operation's variables should have been deleted again"
        );
    }

    /// The enable flag is consumed on activation, so a SOP runs once per
    /// request rather than restarting forever.
    #[tokio::test]
    #[serial]
    async fn the_enable_flag_is_consumed() {
        let (_container, manager) = redis().await;
        let mut con = manager.get_connection().await;

        let state = domain(&["a"]);
        let sop = SOP::Sequence(vec![step("one", "a", &state)]);
        let model = deploy(sop, &["a"], &manager).await;

        let runner = spawn_runner(&manager, model);
        enable_sop(&mut con).await;
        assert_eq!(
            wait_for(&mut con, &format!("{SP}_sop_state"), "completed", 5000).await,
            "completed"
        );
        runner.abort();

        assert_eq!(
            StateManager::get_sp_value(&mut con, &format!("{SP}_sop_enabled")).await,
            Some(false.to_spvalue())
        );
    }

    /// A `sop_id` that is not in the model is skipped with a debug line rather
    /// than crashing the runner - a dashboard can write anything into that key.
    #[tokio::test]
    #[serial]
    async fn an_unknown_sop_id_is_ignored_without_killing_the_runner() {
        let (_container, manager) = redis().await;
        let mut con = manager.get_connection().await;

        let state = domain(&["a"]);
        let sop = SOP::Sequence(vec![step("one", "a", &state)]);
        let model = deploy(sop, &["a"], &manager).await;

        let runner = spawn_runner(&manager, model);
        StateManager::set_sp_value(&mut con, &format!("{SP}_sop_id"), &"nope".to_spvalue()).await;
        enable_sop(&mut con).await;
        tokio::time::sleep(Duration::from_millis(300)).await;

        assert!(!runner.is_finished(), "the runner must survive an unknown id");
        assert_eq!(
            StateManager::get_sp_value(&mut con, "a").await,
            Some(false.to_spvalue()),
            "and must not have run anything"
        );

        // Point it at the real SOP and it picks up from there.
        StateManager::set_sp_value(&mut con, &format!("{SP}_sop_id"), &SOP_ID.to_spvalue()).await;
        assert_eq!(
            wait_for(&mut con, &format!("{SP}_sop_state"), "completed", 5000).await,
            "completed"
        );
        runner.abort();
    }

    /// An idle runner with nothing enabled must not write.
    #[tokio::test]
    #[serial]
    async fn an_idle_runner_writes_nothing() {
        let (_container, manager) = redis().await;
        let mut con = manager.get_connection().await;

        let state = domain(&["a"]);
        let sop = SOP::Sequence(vec![step("one", "a", &state)]);
        let model = deploy(sop, &["a"], &manager).await;

        let runner = spawn_runner(&manager, model);
        tokio::time::sleep(Duration::from_millis(100)).await;

        let before = StateManager::get_full_state(&mut con).await.unwrap();
        tokio::time::sleep(Duration::from_millis(300)).await;
        let after = StateManager::get_full_state(&mut con).await.unwrap();

        assert!(
            before.get_diff_partial_state(&after).state.is_empty(),
            "a disabled SOP runner must not write"
        );
        assert!(!runner.is_finished());
        runner.abort();
    }

    /// A SOP whose operation dies takes the whole SOP to `fatal`, and the
    /// runner tears it down - deleting the operation variables and going back
    /// to idle so a new SOP can be started.
    #[tokio::test]
    #[serial]
    async fn a_fatal_operation_fatals_and_tears_down_the_sop() {
        let (_container, manager) = redis().await;
        let mut con = manager.get_connection().await;

        let state = domain(&["a"]);
        // Starts, never completes, times out fast, no retries, no bypass.
        let doomed = SOP::Operation(Box::new(Operation::new(
            "doomed",
            Some(20),
            Some(10_000),
            None,
            None,
            false,
            vec![Transition::parse(
                "start",
                "var:a == false",
                "true",
                vec!["var:a <- true"],
                Vec::<&str>::new(),
                &state,
            )],
            vec![Transition::parse(
                "never",
                "false",
                "true",
                Vec::<&str>::new(),
                Vec::<&str>::new(),
                &state,
            )],
            vec![],
            vec![],
            vec![],
            vec![],
        )));
        let model = deploy(SOP::Sequence(vec![doomed]), &["a"], &manager).await;

        let before: i64 = redis::cmd("DBSIZE").query_async(&mut con).await.unwrap();

        let runner = spawn_runner(&manager, model);
        enable_sop(&mut con).await;

        let sop_state = wait_for(&mut con, &format!("{SP}_sop_state"), "fatal", 5000).await;
        assert_eq!(sop_state, "fatal");

        // And it tears down, so the runner is free for the next SOP.
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
        assert_eq!(after, before, "a fatal SOP must clean up after itself too");
    }

    /// Pressing stop cancels the running operation, which cancels the SOP, and
    /// the runner tears it down through the third of its three teardown paths.
    #[tokio::test]
    #[serial]
    async fn stop_cancels_a_running_sop_and_tears_it_down() {
        let (_container, manager) = redis().await;
        let mut con = manager.get_connection().await;

        let state = domain(&["a"]);
        // Starts and then sits in Executing indefinitely.
        let long_running = SOP::Operation(Box::new(Operation::new(
            "long",
            Some(10_000),
            Some(10_000),
            None,
            None,
            false,
            vec![Transition::parse(
                "start",
                "var:a == false",
                "true",
                vec!["var:a <- true"],
                Vec::<&str>::new(),
                &state,
            )],
            vec![Transition::parse(
                "never",
                "false",
                "true",
                Vec::<&str>::new(),
                Vec::<&str>::new(),
                &state,
            )],
            vec![],
            vec![],
            vec![],
            vec![],
        )));
        let model = deploy(SOP::Sequence(vec![long_running]), &["a"], &manager).await;

        let before: i64 = redis::cmd("DBSIZE").query_async(&mut con).await.unwrap();

        let runner = spawn_runner(&manager, model);
        enable_sop(&mut con).await;
        assert_eq!(
            wait_for(&mut con, &format!("{SP}_sop_state"), "executing", 5000).await,
            "executing",
            "the SOP should be running before we stop it"
        );

        StateManager::set_sp_value(
            &mut con,
            &format!("{SP}_dashboard_command"),
            &"stop".to_spvalue(),
        )
        .await;

        let sop_state = wait_for(&mut con, &format!("{SP}_sop_state"), "cancelled", 5000).await;
        assert_eq!(sop_state, "cancelled");

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
        assert_eq!(after, before, "a cancelled SOP must clean up after itself");
    }

    /// The escape hatch: with `MICRO_SP_READ_FULL_STATE` set, the runner reads
    /// the whole keyspace every tick with `get_full_state` instead of the
    /// precomputed key set - and still drives a SOP to completion.
    #[tokio::test]
    #[serial]
    async fn read_full_state_env_var_still_drives_a_sop() {
        // SAFETY: serialized with the rest of this crate's Redis tests via
        // `#[serial]`, which uses a single global lock, so no other test
        // observes this env var while it is set.
        unsafe {
            std::env::set_var("MICRO_SP_READ_FULL_STATE", "1");
        }
        let (_container, manager) = redis().await;
        let mut con = manager.get_connection().await;

        let state = domain(&["a"]);
        let sop = SOP::Sequence(vec![step("one", "a", &state)]);
        let model = deploy(sop, &["a"], &manager).await;

        let runner = spawn_runner(&manager, model);
        enable_sop(&mut con).await;

        let sop_state = wait_for(&mut con, &format!("{SP}_sop_state"), "completed", 5000).await;
        runner.abort();
        unsafe {
            std::env::remove_var("MICRO_SP_READ_FULL_STATE");
        }

        assert_eq!(
            sop_state, "completed",
            "the SOP must still run to completion when reading the whole keyspace every tick"
        );
        assert_eq!(
            StateManager::get_sp_value(&mut con, "a").await,
            Some(true.to_spvalue())
        );
    }

    /// `can_sop_start` recurses through every branch type while looking for an
    /// `Alternative` path to start, not just bare operations. A top-level
    /// `Alternative` whose branches are themselves a `Sequence`, a nested
    /// `Alternative` and a `Parallel` exercises all three recursive arms: the
    /// first two can never start (their operation's guard is unsatisfiable),
    /// so the search has to walk past both before it reaches the `Parallel`
    /// branch, finds every one of its children startable, and picks it.
    #[tokio::test]
    #[serial]
    async fn an_alternative_finds_a_startable_path_through_nested_branch_types() {
        let (_container, manager) = redis().await;
        let mut con = manager.get_connection().await;

        let state = domain(&["d", "e"]);

        // Never satisfiable, so `can_sop_start` must reject both of these.
        let never_startable = |name: &str| {
            SOP::Operation(Box::new(Operation::new(
                name,
                Some(10_000),
                Some(10_000),
                None,
                None,
                false,
                vec![Transition::parse(
                    "start",
                    "false",
                    "true",
                    Vec::<&str>::new(),
                    Vec::<&str>::new(),
                    &state,
                )],
                vec![Transition::parse(
                    "complete",
                    "true",
                    "true",
                    Vec::<&str>::new(),
                    Vec::<&str>::new(),
                    &state,
                )],
                vec![],
                vec![],
                vec![],
                vec![],
            )))
        };

        let sop = SOP::Alternative(vec![
            SOP::Sequence(vec![never_startable("seq_op")]),
            SOP::Alternative(vec![never_startable("nested_op")]),
            SOP::Parallel(vec![step("par1", "d", &state), step("par2", "e", &state)]),
        ]);
        let model = deploy(sop, &["d", "e"], &manager).await;

        let runner = spawn_runner(&manager, model);
        enable_sop(&mut con).await;

        let sop_state = wait_for(&mut con, &format!("{SP}_sop_state"), "completed", 5000).await;
        runner.abort();

        assert_eq!(sop_state, "completed");
        assert_eq!(
            StateManager::get_sp_value(&mut con, "d").await,
            Some(true.to_spvalue()),
            "the Parallel branch is the only startable one and must have run"
        );
        assert_eq!(
            StateManager::get_sp_value(&mut con, "e").await,
            Some(true.to_spvalue())
        );
    }

    /// The `op_lone_*` keys that are an operation's own state variable, with
    /// its five bookkeeping siblings filtered out.
    async fn operation_state_keys(con: &mut crate::SPConnection) -> Vec<String> {
        let keys: Vec<String> = redis::cmd("KEYS")
            .arg("op_lone_*")
            .query_async(con)
            .await
            .unwrap();
        let mut own: Vec<String> = keys
            .into_iter()
            .filter(|k| {
                !(k.ends_with("_information")
                    || k.ends_with("_elapsed_executing_ms")
                    || k.ends_with("_elapsed_disabled_ms")
                    || k.ends_with("_failure_retry_counter")
                    || k.ends_with("_timeout_retry_counter"))
            })
            .collect();
        own.sort();
        own
    }

    /// A corrupted operation state does **not** reach the runner's
    /// `SOPState::UNKNOWN` arm, and this pins why - because the obvious reading
    /// of the code says it should.
    ///
    /// `SOP::get_state` does map an unparseable operation state to
    /// `SOPState::UNKNOWN`, and the runner has an arm for it that resets its
    /// bookkeeping. But the `Executing` arm calls `process_sop_node_tick`
    /// *before* it calls `get_state`, and `process_operation`'s own
    /// `OperationState::UNKNOWN` arm repairs the variable back to `initial` in
    /// that same tick. By the time the root state is computed the corruption is
    /// gone, so `get_state` returns `Initial`, never `UNKNOWN`.
    ///
    /// Two consequences worth having written down:
    ///   - the runner's `SOPState::UNKNOWN` arm is unreachable in this path,
    ///     which is why it shows as uncovered;
    ///   - the SOP does not tear down and does not release its unique id. It
    ///     falls back to `Initial` and re-initialises *the same* unique SOP,
    ///     whose operation can no longer satisfy its guard (the SOP already ran
    ///     and its postcondition holds), so it sits there Disabled. A fresh SOP
    ///     therefore cannot be started, and the operation variables are never
    ///     cleaned up.
    #[tokio::test]
    #[serial]
    async fn a_corrupted_operation_state_is_repaired_before_the_sop_can_see_it() {
        let (_container, manager) = redis().await;
        let mut con = manager.get_connection().await;

        let state = domain(&["x"]);
        // A bare top-level `Operation`, not wrapped in a `Sequence` - so the
        // root's own state *is* this operation's state.
        let sop = step("lone", "x", &state);
        let model = deploy(sop, &["x"], &manager).await;

        let runner = spawn_runner(&manager, model);
        enable_sop(&mut con).await;

        assert_eq!(
            wait_for(&mut con, &format!("{SP}_sop_state"), "executing", 5000).await,
            "executing",
            "the operation should be running before we corrupt its state"
        );

        let before = operation_state_keys(&mut con).await;
        let op_key = before
            .first()
            .expect("the operation's own state key must exist")
            .clone();

        StateManager::set_sp_value(&mut con, &op_key, &"totally_bogus".to_spvalue()).await;

        // Several ticks, so the repair and the re-initialisation both happen.
        tokio::time::sleep(Duration::from_millis(600)).await;

        // The corruption is gone: `process_operation` initialised it back
        // rather than leaving a value nothing can parse.
        let repaired = StateManager::get_sp_value(&mut con, &op_key).await;
        assert_ne!(
            repaired,
            Some("totally_bogus".to_spvalue()),
            "process_operation must repair an unparseable operation state"
        );

        // Re-enabling does not start a fresh SOP: the old unique id was never
        // released, so no second operation instance is ever created.
        enable_sop(&mut con).await;
        tokio::time::sleep(Duration::from_millis(600)).await;
        runner.abort();

        let after = operation_state_keys(&mut con).await;
        assert_eq!(
            after, before,
            "no new operation instance appears - the wedged SOP keeps its own"
        );
        assert!(
            StateManager::get_sp_value(&mut con, &op_key).await.is_some(),
            "and the original operation's variables are never cleaned up"
        );
    }

    /// BUG (consequence of `Operation::terminate` ignoring every reason except
    /// `Completed`, see `process_operation`): a bypassed operation never
    /// reaches `terminated_bypassed`, and `SOP::get_state` maps plain
    /// `Bypassed` to `Executing`. So the branch containing it reports
    /// `Executing` forever and the SOP never finishes - the runner sits there
    /// re-walking a tree that can no longer make progress.
    ///
    /// `can_be_bypassed` exists precisely so that a non-critical step can time
    /// out without killing the procedure, so this defeats the feature.
    #[tokio::test]
    #[serial]
    async fn a_bypassed_operation_never_lets_its_sop_finish() {
        let (_container, manager) = redis().await;
        let mut con = manager.get_connection().await;

        let state = domain(&["a", "b"]);
        let sop = SOP::Sequence(vec![
            bypassing_step("flaky", "a", &state),
            step("after", "b", &state),
        ]);
        let model = deploy(sop, &["a", "b"], &manager).await;

        let runner = spawn_runner(&manager, model);
        enable_sop(&mut con).await;

        // Long enough for it to start, time out and bypass several times over.
        tokio::time::sleep(Duration::from_millis(1500)).await;
        let sop_state = value(&mut con, &format!("{SP}_sop_state")).await;
        let after_ran = StateManager::get_sp_value(&mut con, "b").await;
        runner.abort();

        assert_eq!(
            sop_state, "executing",
            "if this now reads 'completed' the terminate() bug is fixed"
        );
        assert_eq!(
            after_ran,
            Some(false.to_spvalue()),
            "and the step after the bypassed one is never reached"
        );
    }
}
