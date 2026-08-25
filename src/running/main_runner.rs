//! The entry point that puts a model on the road.
//!
//! [`main_runner`] spawns every runner task of the stack - planner, plan runner,
//! SOP runner, the two automatic runners, timers, goals and the transform
//! interface - all sharing one Redis connection manager and one `Arc<Model>`.
//! The runners never talk to each other directly; they hand off through keys in
//! Redis.


use crate::{
    running::{goal_runner::goal_runner, sequential::sequential_runner},
    transforms::interface::tf_interface,
    *,
};
use std::sync::Arc;

/// Spawn the whole runner stack for `model` and return immediately.
///
/// Initialises logging and the activity log, then spawns
/// [`sequential_runner`]: one detached task that
/// drives every runner - planner ticker, SOP runner, plan runner, auto
/// transition runner, auto operation runner, timer interface, goal runner and
/// transform interface - in that order, off one snapshot per tick. Every runner
/// reads and writes `{sp_id}_*` keys plus the model's own variables; see the
/// individual runners for their key sets.
///
/// With `MICRO_SP_SEQUENTIAL=0` the eight run as separate detached tasks
/// instead, each polling Redis on its own. That is the arrangement in which two
/// runners can decide from the same stale read and the later write wins, so it
/// is an escape hatch rather than a supported mode - see
/// [`sequential_runner_enabled`].
///
/// * `sp_id` - the namespace every runner key is prefixed with.
/// * `model` - the operations, automatic transitions and SOPs to execute. Moved
///   in and shared between the tasks behind an `Arc`.
/// * `number_of_timers` - how many `{sp_id}_timer_N_*` timers the timer
///   interface should drive. Their variables must already exist in the state.
/// * `connection_manager` - the Redis connection pool the tasks clone.
///
/// The caller has to keep the process alive; the spawned tasks are dropped when
/// the runtime shuts down. The state must be seeded first (see
/// [`generate_runner_state_variables`] and
/// [`generate_operation_state_variables`]), because reading a variable that is
/// not in the state panics.
///
/// ```no_run
/// use micro_sp::*;
/// use std::sync::Arc;
///
/// # async fn example(model: Model) {
/// let connection_manager = Arc::new(ConnectionManager::new().await);
///
/// // Seed Redis with the model's variables, then hand it to the runners.
/// main_runner(&"sp".to_string(), model, 3, &connection_manager).await;
///
/// // The runners are detached; keep the process alive.
/// std::future::pending::<()>().await;
/// # }
/// ```
pub async fn main_runner(
    sp_id: &String,
    model: Model,
    number_of_timers: u64,
    connection_manager: &Arc<ConnectionManager>,
) {
    initialize_env_logger();
    activity_log::init_from_env();

    // One deep copy of the model for the whole process; every task below holds
    // an `Arc::clone` of it.
    let model = Arc::new(model);

    if sequential_runner_enabled() {
        spawn_sequential_runner(sp_id, &model, number_of_timers, connection_manager);
        return;
    }

    log::info!(target: &format!("{sp_id}_micro_sp"), "Spawning planner.");
    let model_clone = Arc::clone(&model);
    let con_clone = connection_manager.clone();
    let sp_id_clone = sp_id.clone();
    tokio::task::spawn(async move {
        planner_ticker(&sp_id_clone, &model_clone, &con_clone)
            .await
            .unwrap()
    });

    log::info!(target:  &format!("{sp_id}_micro_sp"), "Spawning SOP runner.");
    let model_clone = Arc::clone(&model);
    let con_clone = connection_manager.clone();
    let sp_id_clone = sp_id.clone();
    tokio::task::spawn(async move {
        sop_runner(&sp_id_clone, &model_clone, &con_clone)
            .await
            .unwrap()
    });

    log::info!(target:  &format!("{sp_id}_micro_sp"), "Spawning operation runner.");
    let model_clone = Arc::clone(&model);
    let con_clone = connection_manager.clone();
    tokio::task::spawn(async move {
        planned_operation_runner(&model_clone, &con_clone)
            .await
            .unwrap()
    });

    log::info!(target: &format!("{sp_id}_micro_sp"), "Spawning auto transition runner");
    let model_clone = Arc::clone(&model);
    let con_clone = connection_manager.clone();
    tokio::task::spawn(async move {
        auto_transition_runner(&model_clone.name, &model_clone, &con_clone)
            .await
            .unwrap()
    });

    log::info!(target: &format!("{sp_id}_micro_sp"), "Spawning auto operation runner");
    let model_clone = Arc::clone(&model);
    let con_clone = connection_manager.clone();
    tokio::task::spawn(async move {
        auto_operation_runner(
            &model_clone.name,
            &model_clone,
            &con_clone,
        )
        .await
        .unwrap()
    });

    log::info!(target: &format!("{sp_id}_micro_sp"), "Spawning time runner");
    let con_clone = connection_manager.clone();
    let sp_id_clone = sp_id.clone();
    tokio::task::spawn(async move {
        time_interface_runner(&sp_id_clone, &con_clone, number_of_timers)
            .await
            .unwrap()
    });

    log::info!(target: &format!("{sp_id}_micro_sp"), "Spawning time runner");
    let con_clone = connection_manager.clone();
    let sp_id_clone = sp_id.clone();
    tokio::task::spawn(async move { goal_runner(&sp_id_clone, &con_clone).await.unwrap() });

    log::info!(target: &format!("{sp_id}_micro_sp"), "Spawning TF interface");
    let con_clone = connection_manager.clone();
    let sp_id_clone = sp_id.clone();
    tokio::task::spawn(async move { tf_interface(&sp_id_clone, &con_clone).await.unwrap() });
}

/// The whole stack, in one process, against a real Redis.
///
/// Every other test in this crate exercises one runner in isolation with the
/// other runners' outputs faked. This one spawns all eight and checks that they
/// actually compose: a goal is posted to `{sp_id}_incoming_goals`, and the
/// goal runner, the planner ticker, the plan runner and `process_operation`
/// hand off to each other through Redis until the world is in the goal state.
///
/// That handover is the part nobody can test any other way - it is implemented
/// entirely through shared keys written by different tasks on different timers.
/// If a key name, a state string or a handshake order drifts on one side only,
/// this is the test that notices.
/// Whether [`main_runner`] drives everything from one sequential loop.
///
/// On by default. `MICRO_SP_SEQUENTIAL=0`/`false` goes back to the eight
/// separate tasks, which is worth having while the two are being compared - but
/// note that arrangement is what makes read-modify-write across runners
/// non-atomic in the first place, so it is an escape hatch, not a supported
/// mode. See [`crate::running::sequential`].
pub fn sequential_runner_enabled() -> bool {
    match std::env::var("MICRO_SP_SEQUENTIAL") {
        Ok(value) => !matches!(value.trim().to_ascii_lowercase().as_str(), "0" | "false"),
        Err(_) => true,
    }
}

/// Spawn the sequential runner and take the process down if it ever stops.
///
/// The loop is not supposed to return at all, so both a panic and an `Err` mean
/// the control system is gone. Exiting says so. The alternative is what the
/// eight-task arrangement does: `.unwrap()` inside a detached task, so a runner
/// that dies takes no one with it and nothing observes that it is missing - the
/// system keeps running, minus a runner, for as long as nobody notices.
fn spawn_sequential_runner(
    sp_id: &String,
    model: &Arc<Model>,
    number_of_timers: u64,
    connection_manager: &Arc<ConnectionManager>,
) {
    let log_target = format!("{sp_id}_micro_sp");
    log::info!(target: &log_target, "Spawning sequential runner.");

    let model_clone = Arc::clone(model);
    let con_clone = connection_manager.clone();
    let sp_id_clone = sp_id.clone();
    let handle = tokio::task::spawn(async move {
        sequential_runner(&sp_id_clone, &model_clone, &con_clone, number_of_timers)
            .await
            .map_err(|e| e.to_string())
    });

    tokio::task::spawn(async move {
        let reason = match handle.await {
            Ok(Ok(())) => "the sequential runner returned".to_string(),
            Ok(Err(e)) => format!("the sequential runner failed: {e}"),
            Err(e) => format!("the sequential runner panicked: {e}"),
        };
        log::error!(target: &log_target, "{reason}. Exiting.");
        activity_log::flush();
        std::process::exit(1);
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    // `goal_runner` is not re-exported from the crate root (its `pub use` in
    // lib.rs is commented out), so the goal encoding is reached by path.
    use crate::running::goal_runner::{GoalPriority, goal_string_to_sp_value};
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

    fn key(suffix: &str) -> String {
        format!("{SP}_{suffix}")
    }

    /// `pos` walks a -> b -> c, one planned operation per hop, plus one auto
    /// transition so the auto runner has something to do as well.
    fn model_and_domain() -> (Model, State) {
        let mut domain = State::new();
        domain.add_mut(
            SPAssignment::new(SPVariable::new("pos", SPValueType::String), "a".to_spvalue()),
            TARGET,
        );
        domain.add_mut(
            SPAssignment::new(
                SPVariable::new("heartbeat", SPValueType::Bool),
                false.to_spvalue(),
            ),
            TARGET,
        );

        let hop = |name: &str, from: &str, to: &str| {
            Operation::new(
                name,
                Some(10_000),
                Some(10_000),
                None,
                None,
                false,
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
                vec![],
                vec![],
                vec![],
                vec![],
            )
        };

        let model = Model::new(
            SP,
            vec![Transition::parse(
                "beat",
                "var:heartbeat == false",
                "true",
                vec!["var:heartbeat <- true"],
                Vec::<&str>::new(),
                &domain,
            )],
            vec![],
            vec![],
            vec![],
            vec![hop("a_to_b", "a", "b"), hop("b_to_c", "b", "c")],
        );

        (model, domain)
    }

    async fn boot(manager: &Arc<ConnectionManager>) -> Model {
        let (model, domain) = model_and_domain();

        let mut state = generate_runner_state_variables(SP, 1, TARGET);
        state.extend_mut(generate_operation_state_variables(&model, false, TARGET), true);
        state.extend_mut(domain, true);
        state.add_mut(
            SPAssignment::new(
                SPVariable::new(&key("dashboard_command"), SPValueType::String),
                "none".to_spvalue(),
            ),
            TARGET,
        );
        // The timer runner is spawned with one timer, so its keys must exist.
        for (suffix, value) in [
            ("timer_1_request_trigger", false.to_spvalue()),
            (
                "timer_1_request_state",
                ActionRequestState::Initial.to_string().to_spvalue(),
            ),
            ("timer_1_command", "sleep".to_spvalue()),
            ("timer_1_duration_ms", 100.to_spvalue()),
            ("timer_1_elapsed_ms", 0.to_spvalue()),
        ] {
            let variable = match &value {
                SPValue::Bool(_) => SPVariable::new(&key(suffix), SPValueType::Bool),
                SPValue::Int64(_) => SPVariable::new(&key(suffix), SPValueType::Int64),
                _ => SPVariable::new(&key(suffix), SPValueType::String),
            };
            state.add_mut(SPAssignment::new(variable, value), TARGET);
        }
        state = state.update(&key("terminated_operations"), Vec::<SPValue>::new().to_spvalue());
        state = state.update(&key("current_goal_state"), "initial".to_spvalue());
        state = state.update(&key("plan_state"), "initial".to_spvalue());
        state = state.update(&key("planner_state"), "ready".to_spvalue());

        let mut con = manager.get_connection().await;
        StateManager::set_state(&mut con, &state).await;
        model
    }

    async fn wait_for(con: &mut SPConnection, k: &str, expected: SPValue, ms: u64) -> SPValue {
        let deadline = std::time::Instant::now() + Duration::from_millis(ms);
        let mut last = SPValue::Bool(BoolOrUnknown::UNKNOWN);
        while std::time::Instant::now() < deadline {
            if let Some(value) = StateManager::get_sp_value(con, k).await {
                last = value;
                if last == expected {
                    return last;
                }
            }
            tokio::time::sleep(Duration::from_millis(5)).await;
        }
        last
    }

    /// Post a goal and let the stack get there on its own.
    #[tokio::test]
    #[serial]
    async fn a_goal_posted_to_the_inbox_is_planned_and_executed_by_the_whole_stack() {
        let (_container, manager) = redis().await;
        let mut con = manager.get_connection().await;
        let model = boot(&manager).await;

        main_runner(&SP.to_string(), model, 1, &manager).await;

        // The only input: a goal, exactly as an external client would post it.
        StateManager::set_sp_value(
            &mut con,
            &key("incoming_goals"),
            &vec![goal_string_to_sp_value(
                "",
                &"var:pos == c".to_string(),
                GoalPriority::Normal,
            )]
            .to_spvalue(),
        )
        .await;

        let reached = wait_for(&mut con, "pos", "c".to_spvalue(), 15000).await;

        assert_eq!(
            reached,
            "c".to_spvalue(),
            "the stack should have planned a -> b -> c and executed it; \
             planner said: {:?}, plan state: {:?}",
            StateManager::get_sp_value(&mut con, &key("planner_information")).await,
            StateManager::get_sp_value(&mut con, &key("plan_state")).await,
        );

        // And the auto transition runner did its (independent) job too, which
        // shows the runners are not starving each other.
        assert_eq!(
            StateManager::get_sp_value(&mut con, "heartbeat").await,
            Some(true.to_spvalue())
        );
    }

    /// The goal is released again afterwards, so a second goal can follow. This
    /// is the loop a deployment actually runs in, and it only closes if every
    /// handover works in both directions.
    #[tokio::test]
    #[serial]
    async fn two_goals_in_a_row_both_complete() {
        let (_container, manager) = redis().await;
        let mut con = manager.get_connection().await;
        let model = boot(&manager).await;

        main_runner(&SP.to_string(), model, 1, &manager).await;

        StateManager::set_sp_value(
            &mut con,
            &key("incoming_goals"),
            &vec![goal_string_to_sp_value(
                "",
                &"var:pos == b".to_string(),
                GoalPriority::Normal,
            )]
            .to_spvalue(),
        )
        .await;
        assert_eq!(
            wait_for(&mut con, "pos", "b".to_spvalue(), 15000).await,
            "b".to_spvalue(),
            "the first goal should have been reached"
        );

        // Wait for the goal runner to release the goal before posting the next.
        assert_eq!(
            wait_for(&mut con, &key("current_goal_state"), "initial".to_spvalue(), 5000).await,
            "initial".to_spvalue(),
            "the first goal should have been released"
        );

        StateManager::set_sp_value(
            &mut con,
            &key("incoming_goals"),
            &vec![goal_string_to_sp_value(
                "",
                &"var:pos == c".to_string(),
                GoalPriority::Normal,
            )]
            .to_spvalue(),
        )
        .await;
        assert_eq!(
            wait_for(&mut con, "pos", "c".to_spvalue(), 15000).await,
            "c".to_spvalue(),
            "the second goal should have been reached too"
        );
    }

    /// A goal that is already satisfied is not an error - the planner says so
    /// and the goal completes without anything being executed.
    #[tokio::test]
    #[serial]
    async fn a_goal_that_already_holds_completes_without_executing_anything() {
        let (_container, manager) = redis().await;
        let mut con = manager.get_connection().await;
        let model = boot(&manager).await;

        main_runner(&SP.to_string(), model, 1, &manager).await;

        StateManager::set_sp_value(
            &mut con,
            &key("incoming_goals"),
            &vec![goal_string_to_sp_value(
                "",
                &"var:pos == a".to_string(),
                GoalPriority::Normal,
            )]
            .to_spvalue(),
        )
        .await;

        assert_eq!(
            wait_for(&mut con, &key("plan_state"), "completed".to_spvalue(), 10000).await,
            "completed".to_spvalue()
        );
        assert_eq!(
            StateManager::get_sp_value(&mut con, "pos").await,
            Some("a".to_spvalue()),
            "nothing should have moved"
        );
    }

    /// An unreachable goal has to fail rather than hang - the whole point of
    /// reporting `not_found` back up the stack.
    #[tokio::test]
    #[serial]
    async fn an_unreachable_goal_fails_and_releases_the_runner() {
        let (_container, manager) = redis().await;
        let mut con = manager.get_connection().await;
        let model = boot(&manager).await;

        main_runner(&SP.to_string(), model, 1, &manager).await;

        StateManager::set_sp_value(
            &mut con,
            &key("incoming_goals"),
            &vec![goal_string_to_sp_value(
                "",
                &"var:pos == nowhere".to_string(),
                GoalPriority::Normal,
            )]
            .to_spvalue(),
        )
        .await;

        // planner -> not_found -> plan failed -> goal failed -> released.
        assert_eq!(
            wait_for(&mut con, &key("planner_state"), "not_found".to_spvalue(), 15000).await,
            "not_found".to_spvalue()
        );
        assert_eq!(
            wait_for(&mut con, &key("current_goal_state"), "initial".to_spvalue(), 5000).await,
            "initial".to_spvalue(),
            "an unreachable goal must not wedge the runner"
        );
    }

    /// With all eight runners up and nothing asked of them, the process must be
    /// quiet, measured the only way that actually settles it: watch the keyspace.
    #[tokio::test]
    #[serial]
    async fn a_fully_booted_but_idle_stack_writes_nothing() {
        let (_container, manager) = redis().await;
        let mut con = manager.get_connection().await;
        let model = boot(&manager).await;

        main_runner(&SP.to_string(), model, 1, &manager).await;

        // Let the auto transition settle first - that one does have work to do.
        assert_eq!(
            wait_for(&mut con, "heartbeat", true.to_spvalue(), 5000).await,
            true.to_spvalue()
        );
        tokio::time::sleep(Duration::from_millis(200)).await;

        let before = StateManager::get_full_state(&mut con).await.unwrap();
        tokio::time::sleep(Duration::from_millis(500)).await;
        let after = StateManager::get_full_state(&mut con).await.unwrap();

        let diff = before.get_diff_partial_state(&after);
        assert!(
            diff.state.is_empty(),
            "an idle stack of eight runners must not write anything: {diff:?}"
        );
    }
}
