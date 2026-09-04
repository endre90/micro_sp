//! Executing several [`SOPStruct`](crate::SOPStruct)s at the same time.
//!
//! [`sop_runner`](crate::sop_runner) can only ever have one procedure in flight:
//! it holds the running SOP in three scalar slots, and both its request channel
//! (`{sp_id}_sop_id` + `{sp_id}_sop_enabled`) and its status channel
//! (`{sp_id}_sop_state`) are single-valued per `sp_id`. Naming a second SOP means
//! overwriting the first one's id.
//!
//! This runner drops the `{sp_id}_sop_id` indirection - which is the thing that
//! makes it single-valued - and namespaces everything by the SOP's *own* id
//! instead. Every [`SOPStruct`](crate::SOPStruct) in the model gets its own
//! request and status variables, so a model can start and wait on each procedure
//! independently and any number of them can run at once:
//!
//! | Key | Type | Direction | Meaning |
//! |---|---|---|---|
//! | `{sop.id}_sop_enabled` | Bool | model -> runner | start this SOP; consumed (set `false`) on activation |
//! | `{sop.id}_sop_state` | String | runner -> model | `initial` / `executing` / `completed` / `fatal` / `cancelled` |
//! | `{sop.id}_sop_information` | String | runner -> model | human-readable progress |
//!
//! So the wrapper-operation idiom the examples use becomes one wrapper per SOP,
//! with no shared key between them:
//!
//! ```text
//! start action:         var:sop_move_robot_sop_enabled <- true
//! postcondition guard:  var:sop_move_robot_sop_state == completed
//! postcondition action: var:sop_move_robot_sop_state <- initial
//! ```
//!
//! `{sop.id}_sop_information` already exists for every SOP in the model -
//! [`generate_operation_state_variables`](crate::generate_operation_state_variables)
//! creates it. The other two do not: seed them with
//! [`generate_multi_sop_state_variables`](crate::generate_multi_sop_state_variables),
//! and seed them *before* building the model, because `Transition::parse`
//! resolves `var:` names at parse time and panics on a name that is not in the
//! state it is given.
//!
//! One live instance per [`SOPStruct`](crate::SOPStruct): enabling a SOP that is
//! already running is ignored (with a debug line) rather than queued.
//! Cancellation is global, as it is for every other runner -
//! `{sp_id}_dashboard_command == "stop"` is what
//! [`Operation::can_be_cancelled`](crate::Operation::can_be_cancelled) reads, so
//! it cancels every active SOP at once.
//!
//! [`main_runner`](crate::main_runner) spawns this runner; the single-SOP
//! [`sop_runner`](crate::sop_runner) is the one commented out there. Do not spawn
//! a second copy by hand - each copy keeps its own list of active SOPs, so two of
//! them would each start their own instance of the same
//! [`SOPStruct`](crate::SOPStruct) and drive the same hardware twice.
//!
//! It touches none of `{sp_id}_sop_id`, `{sp_id}_sop_enabled`,
//! `{sp_id}_sop_state`, `{sp_id}_sop_current_step` or `{sp_id}_sop_stack`, so it
//! can run alongside the single-SOP runner - as long as the two are not pointed
//! at the same SOP, since they would then fight over its operations' variables.
//!
//! # Inherited sharp edges
//!
//! These live in the machinery below this runner and are not fixed here, but they
//! are visible through it:
//!
//! * `Operation::terminate` only implements `TerminationReason::Completed`, so a
//!   **bypassed** operation never reaches `terminated_bypassed`.
//!   [`SOP::get_state`](crate::SOP::get_state) maps plain `Bypassed` to
//!   `Executing`, so a SOP containing a bypassed operation never finishes.
//! * A retry does not reset `{op}_elapsed_executing_ms` / `_elapsed_disabled_ms`,
//!   so a timeout retry times out again on the next tick. `timeout_retries` buys
//!   extra attempts, not extra time.
//! * [`SOPState::Cancelled`](crate::SOPState::Cancelled) renders as `"cancelled"`
//!   but `SOPState::from_str` has no arm for it. This runner tracks each
//!   instance's state in memory and never parses that key back, so it is
//!   unaffected - and a model guard written as
//!   `var:{sop_id}_sop_state == cancelled` still works, being a plain string
//!   comparison.

use crate::running::sop_runner::process_sop_node_tick;
use crate::*;
use log::Level;
use std::sync::Arc;

/// One SOP currently in flight.
struct ActiveSop {
    /// The [`SOPStruct::id`] this instance came from - the key namespace it
    /// reports under, and what makes "one instance per template" decidable.
    template_id: String,
    /// `{template_id}_{nanoid}`. Only ever used for log and activity-log lines:
    /// unlike the single-SOP runner, nothing in the state is keyed by it.
    unique_id: String,
    /// The runner's own view of how far this instance has got, which is not the
    /// same thing as the tree's derived state - see the `Executing` arm.
    tracked_state: SOPState,
    /// The tree, with every operation renamed to `op_{name}_{nanoid}`, so two
    /// SOPs containing the same operation cannot share lifecycle variables.
    sop: SOP,
    /// The uniquified operation names, kept for the read key set and for
    /// teardown - recomputing them from the tree on every tick would be waste.
    op_names: Vec<String>,
    /// The last line logged for this instance, so `Executing` - which re-enters
    /// its arm every tick - is printed only when it actually changes.
    ///
    /// Kept here rather than read back from `{template_id}_sop_information`:
    /// that key is per template while the message is per instance, so anything
    /// else writing it - a second copy of this runner, a dashboard - would make
    /// every tick look like a change and log at the tick rate. Same shape as
    /// `goal_info_old` in `goal_runner`.
    last_logged_info: String,
}

/// The per-SOP request and status variables [`sop_multi_runner`] reads and
/// writes: `{sop.id}_sop_enabled` (`false`) and `{sop.id}_sop_state`
/// (`"initial"`) for every SOP given.
///
/// Takes the SOPs rather than the whole [`Model`] on purpose: the variables have
/// to exist *before* the model is built, because `Transition::parse` resolves
/// `var:` names at parse time and panics on one it cannot find, and a model that
/// enables a SOP writes `var:{sop_id}_sop_enabled`. So the order is: build the
/// `Vec<SOPStruct>`, call this, merge the result into the state you parse the
/// model against, and merge it into the state you seed into Redis.
///
/// `{sop.id}_sop_information` is deliberately not included here -
/// [`generate_operation_state_variables`] already creates it for every SOP in
/// the model.
///
/// ```
/// use micro_sp::*;
///
/// let sops = vec![SOPStruct {
///     id: "sop_move_robot".to_string(),
///     sop: SOP::Operation(Box::new(Operation {
///         name: "move".to_string(),
///         ..Default::default()
///     })),
/// }];
///
/// let state = generate_multi_sop_state_variables(&sops, "docs");
/// assert_eq!(
///     state.get_value("sop_move_robot_sop_enabled", "docs"),
///     Some(false.to_spvalue())
/// );
/// assert_eq!(
///     state.get_value("sop_move_robot_sop_state", "docs"),
///     Some("initial".to_spvalue())
/// );
/// ```
pub fn generate_multi_sop_state_variables(sops: &[SOPStruct], log_target: &str) -> State {
    let mut state = State::new();
    for sop in sops {
        state.add_mut(
            SPAssignment::new(
                SPVariable::new(&format!("{}_sop_enabled", sop.id), SPValueType::Bool),
                false.to_spvalue(),
            ),
            log_target,
        );
        state.add_mut(
            SPAssignment::new(
                SPVariable::new(&format!("{}_sop_state", sop.id), SPValueType::String),
                SOPState::Initial.to_spvalue(),
            ),
            log_target,
        );
    }
    state
}

/// Keys [`sop_multi_runner`] reads on every tick regardless of what is running.
///
/// The per-operation bookkeeping variables of the running SOPs are added on
/// activation via [`keys_with_active_operations`], once the operations have been
/// uniquified and their names are known.
pub fn sop_multi_runner_static_keys(sp_id: &str, model: &Model) -> Vec<String> {
    let mut keys = vec![
        // read by `Operation::can_be_cancelled` for every operation processed
        format!("{}_dashboard_command", sp_id),
        format!("{}_sop_runner_information", sp_id),
    ];
    for sop in &model.sops {
        keys.push(format!("{}_sop_enabled", sop.id));
        keys.push(format!("{}_sop_state", sop.id));
    }
    // Already contributes `{sop.id}_sop_information` and every guard and action
    // variable of every operation in the model.
    keys.extend(model_variable_keys(model));
    normalize_keys(keys)
}

/// Set a variable, creating it if this is the first time it is written.
///
/// `State::update` panics on a variable that is not there, and the per-SOP status
/// variables are the runner's own output - a model that forgot to seed them
/// should get them created rather than take the runner's task down. The
/// `_and_add_missing` diff at the end of the tick writes the new ones out.
fn publish(state: &mut State, key: &str, value: SPValue, log_target: &str) {
    if state.contains(key) {
        state.update_mut(key, value);
    } else {
        let variable = SPVariable::new(key, value.has_type());
        state.add_mut(SPAssignment::new(variable, value), log_target);
    }
}

/// Runs the multi-SOP executor until the process ends.
///
/// On every tick it reads the per-SOP keys for `model.sops` from Redis, starts
/// every SOP whose `{sop.id}_sop_enabled` is set, advances the operations of all
/// of them, and writes back `{sop.id}_sop_state`, `{sop.id}_sop_information`, the
/// per-operation state and a summary in `{sp_id}_sop_runner_information`. `model`
/// supplies the SOPs to look up, `connection_manager` the shared Redis
/// connection; log output goes to the `{sp_id}_sop_multi_runner` target.
///
/// The active SOPs are advanced one after another within a tick, threading one
/// `State` through all of them, so a SOP started later in the list sees what the
/// earlier ones did on the same tick. That is the same model `SOP::Parallel` uses
/// for its branches and `auto_operation_runner` uses for its active operations,
/// and it means the whole tick is still a single read and a single write.
pub async fn sop_multi_runner(
    sp_id: &str,
    model: &Model,
    connection_manager: &Arc<ConnectionManager>,
) -> Result<(), Box<dyn std::error::Error>> {
    initialize_env_logger();
    activity_log::init_from_env();
    let mut interval = runner_interval();
    let log_target = &format!("{}_sop_multi_runner", sp_id);

    log::info!(target: log_target, "Online.");

    let mut active: Vec<ActiveSop> = vec![];

    // The variables read every tick no matter what is running, and the set
    // actually requested from Redis. The latter grows with the bookkeeping
    // variables of every active SOP's operations - their names only exist once
    // `uniquify_sop_operations` has run, which is why the set is rebuilt on
    // activation and teardown rather than computed once here.
    let static_keys = sop_multi_runner_static_keys(sp_id, model);
    let mut keys = static_keys.clone();
    let read_full_state = read_full_state_enabled();
    if read_full_state {
        log::warn!(target: log_target, "MICRO_SP_READ_FULL_STATE is set: reading the whole keyspace every tick.");
    }

    // A model that never seeded its `{sop.id}_sop_enabled` variables can never
    // enable anything, and would otherwise fail silently. Said once, not once a
    // tick.
    let mut missing_reported = false;

    let mut con = connection_manager.get_connection().await;

    // Real time between ticks. `process_operation` advances the elapsed counters
    // by this rather than by a compile-time constant.
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

        // --- Activation: every SOP whose request flag is up and that is not
        // already running gets a fresh instance.
        let mut activated: Vec<String> = vec![];
        for template in &model.sops {
            let enabled_key = format!("{}_sop_enabled", template.id);

            if !state.contains(&enabled_key) {
                if !missing_reported {
                    log::error!(target: log_target,
                        "'{}' is not in the state, so SOP '{}' can never be enabled. \
                         Seed the per-SOP variables with `generate_multi_sop_state_variables` \
                         before starting this runner.",
                        enabled_key, template.id);
                    missing_reported = true;
                }
                continue;
            }

            if !state.get_bool_or_default_to_false(&enabled_key, &log_target) {
                continue;
            }

            // The request is consumed either way: an ignored one must not sit
            // there and fire the instant the current run finishes.
            new_state = new_state.update(&enabled_key, false.to_spvalue());

            if active.iter().any(|a| a.template_id == template.id) {
                log::debug!(target: log_target,
                    "SOP '{}' is already running; enable request ignored.", template.id);
                continue;
            }

            let unique_id = format!(
                "{}_{}",
                template.id,
                nanoid::nanoid!(10, &NANOID_ALPHABET)
            );
            let sop = uniquify_sop_operations(template.sop.clone());
            let op_names: Vec<String> = get_all_operations_from_sop(&sop)
                .iter()
                .map(|op| op.name.clone())
                .collect();

            // Created here and written out by the diff at the end of this tick;
            // from the next tick on they have to be read back, which is why the
            // key set is rebuilt below.
            new_state =
                add_operation_meta_tracking_variables(&op_names, &new_state, false, &log_target);
            new_state = add_operation_state_tracking_variable(&op_names, &new_state, &log_target);

            log::info!(target: log_target, "SOP '{}' is enabled, starting execution.", template.id);
            activity_log::log_sop(
                log_target,
                &unique_id,
                &SOPState::Initial.to_string(),
                &SOPState::Initial.to_string(),
                "activated",
            );

            activated.push(template.id.clone());
            active.push(ActiveSop {
                template_id: template.id.clone(),
                unique_id,
                tracked_state: SOPState::Initial,
                sop,
                op_names,
                last_logged_info: String::new(),
            });
        }

        // Nothing running and nothing requested - the common case. Falling
        // through here would rewrite the summary variable every tick; the
        // activation pass above cannot have touched `new_state` either, since it
        // only writes when a flag was up, and a flag that was up means either a
        // new instance or an already-running one.
        if active.is_empty() && activated.is_empty() {
            continue;
        }

        // --- Processing: advance every active SOP, threading one state through
        // all of them.
        let mut still_active: Vec<ActiveSop> = Vec::with_capacity(active.len());
        let mut deletes: Vec<String> = vec![];

        for mut instance in active.into_iter() {
            let state_before = instance.tracked_state.clone();
            let mut keep = true;
            let info: String;
            let mut level = Level::Info;
            // What to write into `{template_id}_sop_state` this tick, if
            // anything. `Initial` publishes nothing: the model has just set that
            // key itself, and the tree has not been walked yet.
            let published: Option<SOPState>;

            match instance.tracked_state.clone() {
                SOPState::Initial => {
                    instance.tracked_state = SOPState::Executing;
                    published = None;
                    info = format!(
                        "Initializing a new SOP '{}':\n{}",
                        instance.unique_id,
                        visualize_sop(&instance.sop)
                    );
                }
                SOPState::Executing => {
                    // Published as executing even on the tick the tree finishes:
                    // the terminal value goes out on the next tick, together
                    // with the teardown, which gives the walk's own writes a
                    // tick to land first. This is the single-SOP runner's
                    // handshake, kept as is.
                    published = Some(SOPState::Executing);
                    new_state = process_sop_node_tick(
                        sp_id,
                        new_state,
                        &instance.sop,
                        con.clone(),
                        tick_elapsed_ms,
                        &log_target,
                    )
                    .await;

                    let root = instance.sop.get_state(&new_state, &log_target);
                    if root != SOPState::Executing {
                        info = format!("Completing SOP '{}'.", instance.unique_id);
                        instance.tracked_state = root;
                    } else {
                        info = format!("Executing SOP '{}'.", instance.unique_id);
                    }
                }
                terminal => {
                    // Every terminal state tears the instance down the same way:
                    // publish it, delete the operation variables it created, and
                    // free the template so it can be enabled again. Unlike the
                    // single-SOP runner, `UNKNOWN` cleans up too rather than
                    // leaking its operations' keys.
                    published = Some(terminal.clone());
                    let verb = match terminal {
                        SOPState::Completed => "Completed",
                        SOPState::Fatal => "Fataled",
                        SOPState::Cancelled => "Cancelled",
                        _ => "Abandoned (state UNKNOWN)",
                    };
                    level = match terminal {
                        SOPState::Completed => Level::Info,
                        SOPState::Cancelled => Level::Warn,
                        _ => Level::Error,
                    };
                    info = format!("{} SOP '{}'.", verb, instance.unique_id);

                    for op_name in &instance.op_names {
                        push_operation_keys(&mut deletes, op_name);
                    }
                    keep = false;
                }
            }

            // Logged only when it actually changes - `Executing` re-enters its
            // arm every tick. Compared against this instance's own last line
            // rather than against the state, so a foreign writer of the shared
            // `{template_id}_sop_information` key cannot turn this into a log
            // line per tick. The key itself is still published below: that is
            // the model-facing channel and its contract is unchanged.
            if instance.last_logged_info != info {
                match level {
                    Level::Warn => log::warn!(target: log_target, "{}", info),
                    Level::Error => log::error!(target: log_target, "{}", info),
                    _ => log::info!(target: log_target, "{}", info),
                }
                instance.last_logged_info = info.clone();
            }
            publish(
                &mut new_state,
                &format!("{}_sop_information", instance.template_id),
                info.to_spvalue(),
                &log_target,
            );

            if let Some(sop_state) = published {
                publish(
                    &mut new_state,
                    &format!("{}_sop_state", instance.template_id),
                    sop_state.to_spvalue(),
                    &log_target,
                );
            }

            // A `SOP` line whenever this instance actually moved: either its
            // tracked state changed, or it was torn down (which leaves the state
            // put - a SOP is released back *to* the value it reported).
            if state_before != instance.tracked_state || !keep {
                activity_log::log_sop(
                    log_target,
                    &instance.unique_id,
                    &state_before.to_string(),
                    &instance.tracked_state.to_string(),
                    if keep { "" } else { "released" },
                );
            }

            if keep {
                still_active.push(instance);
            }
        }

        let active_set_changed = !activated.is_empty() || !deletes.is_empty();
        active = still_active;

        let summary = match active.len() {
            0 => "No SOPs active.".to_string(),
            n => format!(
                "{} SOP(s) active: {}.",
                n,
                active
                    .iter()
                    .map(|a| a.template_id.as_str())
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
        };
        publish(
            &mut new_state,
            &format!("{}_sop_runner_information", sp_id),
            summary.to_spvalue(),
            &log_target,
        );

        // Operations were activated and/or torn down this tick, so the set of
        // bookkeeping variables to read from the next tick on has changed.
        if active_set_changed {
            let op_names: Vec<String> = active
                .iter()
                .flat_map(|a| a.op_names.iter().cloned())
                .collect();
            keys = keys_with_active_operations(&static_keys, &op_names);
        }

        // `_and_add_missing` is required: activation adds variables that were not
        // in the state that was read.
        let modified_state = state.get_diff_partial_state_and_add_missing(&new_state);

        if !modified_state.state.is_empty() {
            activity_log::log_state_diff(&log_target, &state, &modified_state);
        }
        // MSET before DEL in one pipeline, so a torn-down SOP's variables are
        // gone even if this tick also wrote them.
        StateManager::apply(&mut con, &modified_state, &[&deletes]).await;
    }
}

/// The multi-SOP runner, driven end to end against a real Redis.
///
/// The property under test throughout is the one the single-SOP runner cannot
/// provide: two procedures in flight at the same time, each with its own request
/// and status keys, each tearing down independently of the other. Nothing here is
/// reachable without Redis - the tree walk carries a connection and the teardown
/// deletes keys.
#[cfg(test)]
mod tests {
    use super::*;
    use serial_test::serial;
    use std::time::Duration;
    use testcontainers::{ContainerAsync, ImageExt, core::ContainerPort, runners::AsyncRunner};
    use testcontainers_modules::redis::Redis;

    const SP: &str = "sp";
    const TARGET: &str = "test";

    /// The host port to bind the test container to. `ConnectionManager::new`
    /// reads the same variable, so one setting moves both ends - which is what
    /// lets these tests run on a machine where something else already holds
    /// 6379. Defaults to 6379, as every other test module in the crate does.
    fn redis_port() -> u16 {
        std::env::var("REDIS_PORT")
            .ok()
            .and_then(|port| port.parse().ok())
            .unwrap_or(6379)
    }

    async fn redis() -> (ContainerAsync<Redis>, Arc<ConnectionManager>) {
        let container = Redis::default()
            .with_mapped_port(redis_port(), ContainerPort::Tcp(6379))
            .start()
            .await
            .unwrap();
        let manager = Arc::new(ConnectionManager::new().await);
        let mut con = manager.get_connection().await;
        StateManager::flush_state(&mut con).await;
        (container, manager)
    }

    /// The domain the SOPs operate on: one boolean per step.
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

    /// Starts, sets `flag`, then sits in `Executing` indefinitely.
    fn long_step(name: &str, flag: &str, state: &State) -> SOP {
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

    /// Starts, never completes, times out in 20 ms with no retries and no
    /// bypass - so it goes fatal, and takes its SOP with it.
    fn doomed_step(name: &str, flag: &str, state: &State) -> SOP {
        SOP::Operation(Box::new(Operation::new(
            name,
            Some(20),
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

    fn sop(id: &str, tree: SOP) -> SOPStruct {
        SOPStruct {
            id: id.to_string(),
            sop: tree,
        }
    }

    /// Build the model plus the full initial state the runner needs in Redis.
    async fn deploy(
        sops: Vec<SOPStruct>,
        flags: &[&str],
        manager: &Arc<ConnectionManager>,
    ) -> Model {
        deploy_with(sops, flags, true, manager).await
    }

    /// `seed_multi_variables = false` leaves out
    /// `generate_multi_sop_state_variables`, i.e. simulates a caller who forgot.
    async fn deploy_with(
        sops: Vec<SOPStruct>,
        flags: &[&str],
        seed_multi_variables: bool,
        manager: &Arc<ConnectionManager>,
    ) -> Model {
        let model = Model::new(SP, vec![], vec![], vec![], sops, vec![]);

        let mut state = generate_runner_state_variables(SP, 0, TARGET);
        state.extend_mut(
            generate_operation_state_variables(&model, false, TARGET),
            true,
        );
        if seed_multi_variables {
            state.extend_mut(generate_multi_sop_state_variables(&model.sops, TARGET), true);
        }
        state.extend_mut(domain(flags), true);
        state.add_mut(
            SPAssignment::new(
                SPVariable::new(&format!("{SP}_dashboard_command"), SPValueType::String),
                "none".to_spvalue(),
            ),
            TARGET,
        );

        let mut con = manager.get_connection().await;
        StateManager::set_state(&mut con, &state).await;
        model
    }

    fn spawn_runner(manager: &Arc<ConnectionManager>, model: Model) -> tokio::task::JoinHandle<()> {
        let manager = Arc::clone(manager);
        tokio::spawn(async move {
            let _ = sop_multi_runner(SP, &model, &manager).await;
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

    async fn enable(con: &mut SPConnection, sop_id: &str) {
        StateManager::set_sp_value(con, &format!("{sop_id}_sop_enabled"), &true.to_spvalue()).await;
    }

    async fn keys_matching(con: &mut SPConnection, pattern: &str) -> Vec<String> {
        let mut keys: Vec<String> = redis::cmd("KEYS")
            .arg(pattern)
            .query_async(con)
            .await
            .unwrap();
        keys.sort();
        keys
    }

    /// The headline: two different SOPs enabled at the same time both run to
    /// completion, each reporting under its own id. This is the case the
    /// single-SOP runner cannot express at all - there is only one
    /// `{sp_id}_sop_id` to point at.
    #[tokio::test]
    #[serial]
    async fn two_sops_enabled_at_once_both_run_to_completion() {
        let (_container, manager) = redis().await;
        let mut con = manager.get_connection().await;

        let state = domain(&["a", "b", "c", "d"]);
        let model = deploy(
            vec![
                sop(
                    "sop_alpha",
                    SOP::Sequence(vec![step("alpha_one", "a", &state), step("alpha_two", "b", &state)]),
                ),
                sop(
                    "sop_beta",
                    SOP::Sequence(vec![step("beta_one", "c", &state), step("beta_two", "d", &state)]),
                ),
            ],
            &["a", "b", "c", "d"],
            &manager,
        )
        .await;

        let runner = spawn_runner(&manager, model);
        enable(&mut con, "sop_alpha").await;
        enable(&mut con, "sop_beta").await;

        let alpha = wait_for(&mut con, "sop_alpha_sop_state", "completed", 5000).await;
        let beta = wait_for(&mut con, "sop_beta_sop_state", "completed", 5000).await;
        runner.abort();

        assert_eq!(alpha, "completed");
        assert_eq!(beta, "completed");
        for flag in ["a", "b", "c", "d"] {
            assert_eq!(
                StateManager::get_sp_value(&mut con, flag).await,
                Some(true.to_spvalue()),
                "step '{flag}' did not run"
            );
        }
    }

    /// And they really do overlap rather than queue: both report `executing` at
    /// the same instant.
    #[tokio::test]
    #[serial]
    async fn two_sops_are_executing_at_the_same_time() {
        let (_container, manager) = redis().await;
        let mut con = manager.get_connection().await;

        let state = domain(&["a", "b"]);
        let model = deploy(
            vec![
                sop("sop_alpha", SOP::Sequence(vec![long_step("alpha", "a", &state)])),
                sop("sop_beta", SOP::Sequence(vec![long_step("beta", "b", &state)])),
            ],
            &["a", "b"],
            &manager,
        )
        .await;

        let runner = spawn_runner(&manager, model);
        enable(&mut con, "sop_alpha").await;
        enable(&mut con, "sop_beta").await;

        assert_eq!(
            wait_for(&mut con, "sop_alpha_sop_state", "executing", 5000).await,
            "executing"
        );
        // Read beta *while* alpha is still executing - neither can ever finish.
        let beta = wait_for(&mut con, "sop_beta_sop_state", "executing", 5000).await;
        let alpha_still = value(&mut con, "sop_alpha_sop_state").await;
        runner.abort();

        assert_eq!(beta, "executing");
        assert_eq!(
            alpha_still, "executing",
            "the second SOP must not have had to wait for the first"
        );
        assert_eq!(
            StateManager::get_sp_value(&mut con, "a").await,
            Some(true.to_spvalue())
        );
        assert_eq!(
            StateManager::get_sp_value(&mut con, "b").await,
            Some(true.to_spvalue())
        );
    }

    /// Status keys are per SOP: one finishing says nothing about the other.
    #[tokio::test]
    #[serial]
    async fn finishing_one_sop_leaves_the_others_status_alone() {
        let (_container, manager) = redis().await;
        let mut con = manager.get_connection().await;

        let state = domain(&["a", "b"]);
        let model = deploy(
            vec![
                sop("sop_quick", SOP::Sequence(vec![step("alpha", "a", &state)])),
                sop("sop_slow", SOP::Sequence(vec![long_step("beta", "b", &state)])),
            ],
            &["a", "b"],
            &manager,
        )
        .await;

        let runner = spawn_runner(&manager, model);
        enable(&mut con, "sop_quick").await;
        enable(&mut con, "sop_slow").await;

        assert_eq!(
            wait_for(&mut con, "sop_quick_sop_state", "completed", 5000).await,
            "completed"
        );
        let slow = value(&mut con, "sop_slow_sop_state").await;
        runner.abort();

        assert_eq!(
            slow, "executing",
            "the still-running SOP must not inherit the finished one's state"
        );
    }

    /// Teardown is per instance: the finished SOP's operation variables are
    /// deleted while the still-running SOP keeps every one of its own.
    #[tokio::test]
    #[serial]
    async fn teardown_removes_only_the_finished_sops_operation_variables() {
        let (_container, manager) = redis().await;
        let mut con = manager.get_connection().await;

        let state = domain(&["a", "b"]);
        let model = deploy(
            vec![
                sop("sop_quick", SOP::Sequence(vec![step("alpha", "a", &state)])),
                sop("sop_slow", SOP::Sequence(vec![long_step("beta", "b", &state)])),
            ],
            &["a", "b"],
            &manager,
        )
        .await;

        let runner = spawn_runner(&manager, model);
        enable(&mut con, "sop_quick").await;
        enable(&mut con, "sop_slow").await;

        assert_eq!(
            wait_for(&mut con, "sop_quick_sop_state", "completed", 5000).await,
            "completed"
        );

        // Give the teardown tick a moment to land.
        let deadline = std::time::Instant::now() + Duration::from_millis(3000);
        let mut alpha_keys = keys_matching(&mut con, "op_alpha_*").await;
        while !alpha_keys.is_empty() && std::time::Instant::now() < deadline {
            tokio::time::sleep(Duration::from_millis(5)).await;
            alpha_keys = keys_matching(&mut con, "op_alpha_*").await;
        }
        let beta_keys = keys_matching(&mut con, "op_beta_*").await;
        runner.abort();

        assert!(
            alpha_keys.is_empty(),
            "the finished SOP's operation variables should be gone: {alpha_keys:?}"
        );
        assert_eq!(
            beta_keys.len(),
            1 + OPERATION_META_SUFFIXES.len(),
            "the running SOP must keep its state variable and its five \
             bookkeeping siblings: {beta_keys:?}"
        );
    }

    /// One live instance per SOP: re-enabling a running one is ignored, and the
    /// request is consumed rather than left to fire the moment it finishes.
    #[tokio::test]
    #[serial]
    async fn re_enabling_a_running_sop_is_ignored() {
        let (_container, manager) = redis().await;
        let mut con = manager.get_connection().await;

        let state = domain(&["a"]);
        let model = deploy(
            vec![sop(
                "sop_alpha",
                SOP::Sequence(vec![long_step("alpha", "a", &state)]),
            )],
            &["a"],
            &manager,
        )
        .await;

        let runner = spawn_runner(&manager, model);
        enable(&mut con, "sop_alpha").await;
        assert_eq!(
            wait_for(&mut con, "sop_alpha_sop_state", "executing", 5000).await,
            "executing"
        );
        let before = keys_matching(&mut con, "op_alpha_*").await;

        enable(&mut con, "sop_alpha").await;
        tokio::time::sleep(Duration::from_millis(300)).await;
        let after = keys_matching(&mut con, "op_alpha_*").await;
        let flag = StateManager::get_sp_value(&mut con, "sop_alpha_sop_enabled").await;
        runner.abort();

        assert_eq!(
            after, before,
            "no second instance may be created for a SOP that is already running"
        );
        assert_eq!(
            flag,
            Some(false.to_spvalue()),
            "the ignored request must still be consumed"
        );
    }

    /// A SOP that goes fatal is torn down on its own; the others carry on. On
    /// the single-SOP runner there is nothing to carry on.
    #[tokio::test]
    #[serial]
    async fn a_fatal_sop_does_not_take_the_others_down() {
        let (_container, manager) = redis().await;
        let mut con = manager.get_connection().await;

        let state = domain(&["a", "b"]);
        let model = deploy(
            vec![
                sop("sop_doomed", SOP::Sequence(vec![doomed_step("alpha", "a", &state)])),
                sop("sop_healthy", SOP::Sequence(vec![step("beta", "b", &state)])),
            ],
            &["a", "b"],
            &manager,
        )
        .await;

        let runner = spawn_runner(&manager, model);
        enable(&mut con, "sop_doomed").await;
        enable(&mut con, "sop_healthy").await;

        assert_eq!(
            wait_for(&mut con, "sop_doomed_sop_state", "fatal", 5000).await,
            "fatal"
        );
        assert_eq!(
            wait_for(&mut con, "sop_healthy_sop_state", "completed", 5000).await,
            "completed",
            "a fatal SOP must not stop the others"
        );

        // And the fatal one cleaned up after itself.
        let deadline = std::time::Instant::now() + Duration::from_millis(3000);
        let mut alpha_keys = keys_matching(&mut con, "op_alpha_*").await;
        while !alpha_keys.is_empty() && std::time::Instant::now() < deadline {
            tokio::time::sleep(Duration::from_millis(5)).await;
            alpha_keys = keys_matching(&mut con, "op_alpha_*").await;
        }
        assert!(!runner.is_finished());
        runner.abort();
        assert!(
            alpha_keys.is_empty(),
            "a fatal SOP must clean up after itself too: {alpha_keys:?}"
        );
    }

    /// Cancellation is global by design: one stop command cancels every SOP in
    /// flight and tears them all down.
    #[tokio::test]
    #[serial]
    async fn stop_cancels_every_active_sop() {
        let (_container, manager) = redis().await;
        let mut con = manager.get_connection().await;

        let state = domain(&["a", "b"]);
        let model = deploy(
            vec![
                sop("sop_alpha", SOP::Sequence(vec![long_step("alpha", "a", &state)])),
                sop("sop_beta", SOP::Sequence(vec![long_step("beta", "b", &state)])),
            ],
            &["a", "b"],
            &manager,
        )
        .await;

        let runner = spawn_runner(&manager, model);
        enable(&mut con, "sop_alpha").await;
        enable(&mut con, "sop_beta").await;
        assert_eq!(
            wait_for(&mut con, "sop_alpha_sop_state", "executing", 5000).await,
            "executing"
        );
        assert_eq!(
            wait_for(&mut con, "sop_beta_sop_state", "executing", 5000).await,
            "executing"
        );

        StateManager::set_sp_value(
            &mut con,
            &format!("{SP}_dashboard_command"),
            &"stop".to_spvalue(),
        )
        .await;

        assert_eq!(
            wait_for(&mut con, "sop_alpha_sop_state", "cancelled", 5000).await,
            "cancelled"
        );
        assert_eq!(
            wait_for(&mut con, "sop_beta_sop_state", "cancelled", 5000).await,
            "cancelled"
        );

        let deadline = std::time::Instant::now() + Duration::from_millis(3000);
        let mut remaining = keys_matching(&mut con, "op_*").await;
        while !remaining.is_empty() && std::time::Instant::now() < deadline {
            tokio::time::sleep(Duration::from_millis(5)).await;
            remaining = keys_matching(&mut con, "op_*").await;
        }
        assert!(!runner.is_finished());
        runner.abort();
        assert!(
            remaining.is_empty(),
            "every cancelled SOP must clean up after itself: {remaining:?}"
        );
    }

    /// An idle runner with nothing enabled must not write - including the
    /// summary variable, which is the one thing this runner writes that the
    /// single-SOP runner does not.
    #[tokio::test]
    #[serial]
    async fn an_idle_runner_writes_nothing() {
        let (_container, manager) = redis().await;
        let mut con = manager.get_connection().await;

        let state = domain(&["a"]);
        let model = deploy(
            vec![sop("sop_alpha", SOP::Sequence(vec![step("alpha", "a", &state)]))],
            &["a"],
            &manager,
        )
        .await;

        let runner = spawn_runner(&manager, model);
        tokio::time::sleep(Duration::from_millis(100)).await;

        let before = StateManager::get_full_state(&mut con).await.unwrap();
        tokio::time::sleep(Duration::from_millis(300)).await;
        let after = StateManager::get_full_state(&mut con).await.unwrap();

        assert!(
            before.get_diff_partial_state(&after).state.is_empty(),
            "an idle multi-SOP runner must not write"
        );
        assert!(!runner.is_finished());
        runner.abort();
    }

    /// The summary variable tracks the active set, and comes back to "none" once
    /// the last SOP is done - the only reason the idle fast path is placed after
    /// the activation pass rather than before it.
    #[tokio::test]
    #[serial]
    async fn the_summary_variable_names_the_active_sops_and_is_reset() {
        let (_container, manager) = redis().await;
        let mut con = manager.get_connection().await;

        let state = domain(&["a", "b"]);
        let model = deploy(
            vec![
                sop("sop_quick", SOP::Sequence(vec![step("alpha", "a", &state)])),
                sop("sop_slow", SOP::Sequence(vec![long_step("beta", "b", &state)])),
            ],
            &["a", "b"],
            &manager,
        )
        .await;

        let runner = spawn_runner(&manager, model);
        enable(&mut con, "sop_quick").await;
        enable(&mut con, "sop_slow").await;

        let summary_key = format!("{SP}_sop_runner_information");
        let deadline = std::time::Instant::now() + Duration::from_millis(5000);
        let mut summary = String::new();
        while std::time::Instant::now() < deadline {
            summary = value(&mut con, &summary_key).await;
            if summary.starts_with("2 SOP(s) active") {
                break;
            }
            tokio::time::sleep(Duration::from_millis(5)).await;
        }
        assert!(
            summary.contains("sop_quick") && summary.contains("sop_slow"),
            "both SOPs should be named while both are running, got {summary:?}"
        );

        // The quick one finishes; the summary shrinks to just the slow one.
        assert_eq!(
            wait_for(&mut con, "sop_quick_sop_state", "completed", 5000).await,
            "completed"
        );
        let deadline = std::time::Instant::now() + Duration::from_millis(3000);
        while std::time::Instant::now() < deadline {
            summary = value(&mut con, &summary_key).await;
            if summary.starts_with("1 SOP(s) active") {
                break;
            }
            tokio::time::sleep(Duration::from_millis(5)).await;
        }
        runner.abort();
        assert_eq!(summary, "1 SOP(s) active: sop_slow.");
    }

    /// A caller who never seeded the per-SOP variables gets a loud error rather
    /// than a task that panicked on a missing key and died in silence. Reading a
    /// variable that is not in the state panics (see `State::get_value`), so this
    /// is a real hazard for a brand-new key layout.
    #[tokio::test]
    #[serial]
    async fn a_missing_enable_variable_does_not_kill_the_runner() {
        let (_container, manager) = redis().await;
        let mut con = manager.get_connection().await;

        let state = domain(&["a"]);
        let model = deploy_with(
            vec![sop("sop_alpha", SOP::Sequence(vec![step("alpha", "a", &state)]))],
            &["a"],
            false,
            &manager,
        )
        .await;

        let runner = spawn_runner(&manager, model);
        tokio::time::sleep(Duration::from_millis(300)).await;

        assert!(
            !runner.is_finished(),
            "an unseeded SOP must not take the runner down"
        );
        assert_eq!(
            StateManager::get_sp_value(&mut con, "a").await,
            Some(false.to_spvalue()),
            "and nothing may have run"
        );

        // Seeding it after the fact is enough to get going.
        StateManager::set_sp_value(&mut con, "sop_alpha_sop_enabled", &true.to_spvalue()).await;
        assert_eq!(
            wait_for(&mut con, "sop_alpha_sop_state", "completed", 5000).await,
            "completed"
        );
        runner.abort();
    }

    /// The escape hatch: with `MICRO_SP_READ_FULL_STATE` set, the runner reads
    /// the whole keyspace every tick instead of its key set - and still drives
    /// two SOPs at once.
    #[tokio::test]
    #[serial]
    async fn read_full_state_env_var_still_drives_several_sops() {
        // SAFETY: serialized with the rest of this crate's Redis tests via
        // `#[serial]`, which uses a single global lock, so no other test
        // observes this env var while it is set.
        unsafe {
            std::env::set_var("MICRO_SP_READ_FULL_STATE", "1");
        }
        let (_container, manager) = redis().await;
        let mut con = manager.get_connection().await;

        let state = domain(&["a", "b"]);
        let model = deploy(
            vec![
                sop("sop_alpha", SOP::Sequence(vec![step("alpha", "a", &state)])),
                sop("sop_beta", SOP::Sequence(vec![step("beta", "b", &state)])),
            ],
            &["a", "b"],
            &manager,
        )
        .await;

        let runner = spawn_runner(&manager, model);
        enable(&mut con, "sop_alpha").await;
        enable(&mut con, "sop_beta").await;

        let alpha = wait_for(&mut con, "sop_alpha_sop_state", "completed", 5000).await;
        let beta = wait_for(&mut con, "sop_beta_sop_state", "completed", 5000).await;
        runner.abort();
        unsafe {
            std::env::remove_var("MICRO_SP_READ_FULL_STATE");
        }

        assert_eq!(alpha, "completed");
        assert_eq!(beta, "completed");
    }

    /// `Parallel` and `Alternative` still behave inside a multi-SOP run - the
    /// tree walk is shared with `sop_runner`, but this pins that sharing.
    #[tokio::test]
    #[serial]
    async fn branch_nodes_still_work_when_several_sops_run() {
        let (_container, manager) = redis().await;
        let mut con = manager.get_connection().await;

        let state = domain(&["a", "b", "c", "d"]);
        let model = deploy(
            vec![
                sop(
                    "sop_parallel",
                    SOP::Parallel(vec![step("alpha", "a", &state), step("beta", "b", &state)]),
                ),
                sop(
                    "sop_alternative",
                    SOP::Alternative(vec![step("gamma", "c", &state), step("delta", "d", &state)]),
                ),
            ],
            &["a", "b", "c", "d"],
            &manager,
        )
        .await;

        let runner = spawn_runner(&manager, model);
        enable(&mut con, "sop_parallel").await;
        enable(&mut con, "sop_alternative").await;

        assert_eq!(
            wait_for(&mut con, "sop_parallel_sop_state", "completed", 5000).await,
            "completed"
        );
        assert_eq!(
            wait_for(&mut con, "sop_alternative_sop_state", "completed", 5000).await,
            "completed"
        );
        runner.abort();

        for flag in ["a", "b"] {
            assert_eq!(
                StateManager::get_sp_value(&mut con, flag).await,
                Some(true.to_spvalue()),
                "both Parallel branches must run"
            );
        }
        assert_eq!(
            StateManager::get_sp_value(&mut con, "c").await,
            Some(true.to_spvalue()),
            "the first viable Alternative branch runs"
        );
        assert_eq!(
            StateManager::get_sp_value(&mut con, "d").await,
            Some(false.to_spvalue()),
            "and the second must not also run"
        );
    }

    /// The key set has to be complete: reading a variable that is not in the
    /// state panics, so a hole here is a dead runner rather than a degraded one.
    #[test]
    fn the_static_key_set_covers_every_per_sop_key_and_every_model_variable() {
        use std::collections::HashSet;

        let state = domain(&["a", "b"]);
        let model = Model::new(
            SP,
            vec![],
            vec![],
            vec![],
            vec![
                sop("sop_alpha", SOP::Sequence(vec![step("alpha", "a", &state)])),
                sop("sop_beta", SOP::Sequence(vec![step("beta", "b", &state)])),
            ],
            vec![],
        );

        let keys = sop_multi_runner_static_keys(SP, &model);
        let mut sorted = keys.clone();
        sorted.sort_unstable();
        sorted.dedup();
        assert_eq!(keys, sorted, "key sets must be sorted and deduplicated");

        let set: HashSet<String> = keys.into_iter().collect();
        for key in [
            "sp_dashboard_command",
            "sp_sop_runner_information",
            "sop_alpha_sop_enabled",
            "sop_alpha_sop_state",
            "sop_alpha_sop_information",
            "sop_beta_sop_enabled",
            "sop_beta_sop_state",
            "sop_beta_sop_information",
            "a",
            "b",
        ] {
            assert!(set.contains(key), "the static key set is missing '{key}'");
        }
        for key in model_variable_keys(&model) {
            assert!(set.contains(&key), "missing model variable '{key}'");
        }
        // The single-SOP runner's channel is deliberately not touched, so the
        // two runners cannot fight over it.
        for key in ["sp_sop_id", "sp_sop_enabled", "sp_sop_state"] {
            assert!(
                !set.contains(key),
                "the multi runner must not read the single-SOP key '{key}'"
            );
        }
    }

    /// The seeding helper covers exactly the two variables the runner needs the
    /// caller to create, and leaves `_sop_information` to
    /// `generate_operation_state_variables`.
    #[test]
    fn the_seeding_helper_creates_the_request_and_status_variables() {
        let state = domain(&["a"]);
        let sops = vec![sop(
            "sop_alpha",
            SOP::Sequence(vec![step("alpha", "a", &state)]),
        )];

        let seeded = generate_multi_sop_state_variables(&sops, TARGET);
        assert_eq!(
            seeded.get_value("sop_alpha_sop_enabled", TARGET),
            Some(false.to_spvalue())
        );
        assert_eq!(
            seeded.get_value("sop_alpha_sop_state", TARGET),
            Some("initial".to_spvalue())
        );
        assert!(
            !seeded.contains("sop_alpha_sop_information"),
            "`_sop_information` is generate_operation_state_variables' job"
        );
        assert_eq!(seeded.state.len(), 2);
    }
}
