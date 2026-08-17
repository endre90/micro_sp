use tokio::sync::mpsc;

use crate::{running::goal_runner::goal_runner, transforms::interface::tf_interface, *};
use std::sync::Arc;

// Run everything and provide a model
//
// DONE: PERF: `model.clone()` was done five times here and two of the spawned
// runners then cloned it *again* internally, so the process held seven deep
// copies of every operation, transition, predicate and action for its whole
// lifetime. The model is wrapped in an `Arc` once and each task gets an
// `Arc::clone`, which is a refcount bump. The runners still take `&Model`, so
// no signature changed - the reference is taken from the `Arc` inside each
// task.
//
// `sp_id` is still cloned per task, deliberately: it is a short `String` and
// four of them at startup is not worth an `Arc` and the churn.
//
// PERF (the big architectural one): seven independent tasks each poll Redis on
// their own timer - 50 ms, 100 ms x4, 200 ms x2, 250 ms, 500 ms. Together they
// issue roughly 40-60 PINGs, ~20 blocking `KEYS *` scans and a similar number of
// MGET/MSET pairs per second, whether or not anything is happening. That is the
// idle CPU floor. And because a single logical step (auto operation fires ->
// SOP runner observes -> plan runner reacts) has to travel through Redis
// between each pair of runners, the observed reaction time is the *sum* of the
// intervening tick intervals, not the fastest one. Two directions worth taking:
//   1. Event-driven: have each runner subscribe to changes on its own key set
//      (Redis keyspace notifications or an explicit PUBLISH on write) and use
//      `tokio::select!` between that stream and a slow watchdog tick. Latency
//      then tracks the actual Redis round trip (sub-millisecond locally)
//      instead of the tick period, and idle CPU drops to near zero.
//   2. Co-locate the tightly coupled runners: `sop_runner`,
//      `auto_operation_runner` and `planned_operation_runner` all read the same
//      full state and all write operation variables. Running them as three
//      phases of one loop over one shared in-memory `State` would remove two of
//      the three full-state reads per cycle *and* the Redis hop between them,
//      leaving Redis as the external interface/persistence layer rather than
//      the inter-runner message bus.
// A middle ground that needs no restructuring: keep one authoritative in-memory
// `State` per process behind an `Arc<RwLock<State>>`, have exactly one task
// sync it with Redis, and let the runners read from memory and publish deltas.
pub async fn main_runner(
    sp_id: &String,
    model: Model,
    number_of_timers: u64,
    connection_manager: &Arc<ConnectionManager>,
) {
    // Logs from extern crates to stdout
    // initialize_env_logger();

    // // Enable coverability tracking:
    // let coverability_tracking = false;

    // // Add the variables that keep track of the runner state
    // let runner_vars = generate_runner_state_variables(&sp_id);
    // let state = state.extend(runner_vars, true);

    // let op_vars = generate_operation_state_variables(&model, coverability_tracking);
    // let state = state.extend(op_vars, true);

    // Start the on-disk activity log if the environment asked for one. A no-op
    // otherwise, so a consuming package that has not opted in never has files
    // appear in its working directory. See `utils::activity_log`.
    activity_log::init_from_env();

    // One deep copy of the model for the whole process; every task below holds
    // an `Arc::clone` of it.
    let model = Arc::new(model);

    log::info!(target: &format!("{sp_id}_micro_sp"), "Spawning planner.");
    let model_clone = Arc::clone(&model);
    let con_clone = connection_manager.clone();
    let sp_id_clone = sp_id.clone();
    tokio::task::spawn(async move {
        planner_ticker(&sp_id_clone, &model_clone, &con_clone)
            .await
            .unwrap()
    });

    // let (op_log_tx, op_log_rx) = mpsc::channel::<LogMsg>(100);
    // log::info!(target: &format!("{sp_id}_micro_sp"), "Spawning operation logging receiver.");
    // let con_clone = connection_manager.clone();
    // let sp_id_clone = sp_id.clone();
    // tokio::task::spawn(async move {
    //     operation_log_receiver_task(op_log_rx, &con_clone, &sp_id_clone).await
    // });

    log::info!(target:  &format!("{sp_id}_micro_sp"), "Spawning SOP runner.");
    let model_clone = Arc::clone(&model);
    let con_clone = connection_manager.clone();
    let sp_id_clone = sp_id.clone();
    // let op_log_tx_clone = op_log_tx.clone();
    // let sop_log_tx_clone = sop_op_log_tx.clone();
    tokio::task::spawn(async move {
        sop_runner(&sp_id_clone, &model_clone, &con_clone)
            .await
            .unwrap()
    });

    log::info!(target:  &format!("{sp_id}_micro_sp"), "Spawning operation runner.");
    let model_clone = Arc::clone(&model);
    let con_clone = connection_manager.clone();
    // let op_log_tx_clone = op_log_tx.clone();
    // let sop_log_tx_clone = sop_op_log_tx.clone();
    tokio::task::spawn(async move {
        planned_operation_runner(&model_clone, &con_clone)
            .await
            .unwrap()
    });

    log::info!(target: &format!("{sp_id}_micro_sp"), "Spawning auto transition runner");
    let model_clone = Arc::clone(&model);
    let con_clone = connection_manager.clone();
    // let op_log_tx_clone = op_log_tx.clone();
    tokio::task::spawn(async move {
        auto_transition_runner(&model_clone.name, &model_clone, &con_clone)
            .await
            .unwrap()
    });

    log::info!(target: &format!("{sp_id}_micro_sp"), "Spawning auto operation runner");
    let model_clone = Arc::clone(&model);
    let con_clone = connection_manager.clone();
    // let op_log_tx_clone = op_log_tx.clone();
    // let sop_log_tx_clone = sop_op_log_tx.clone();
    tokio::task::spawn(async move {
        auto_operation_runner(
            &model_clone.name,
            &model_clone,
            // op_log_tx_clone,
            // sop_log_tx_clone,
            &con_clone,
        )
        .await
        .unwrap()
    });

    // log::info!(target: &format!("{sp_id}_micro_sp"), "Spawning mutexed auto operation runner");
    // let model_clone = model.clone();
    // let con_clone = connection_manager.clone();
    // let op_log_tx_clone = op_log_tx.clone();
    // // let sop_log_tx_clone = sop_op_log_tx.clone();
    // tokio::task::spawn(async move {
    //     mutexed_auto_operation_runner(
    //         &model_clone.name,
    //         &model_clone,
    //         op_log_tx_clone,
    //         // sop_log_tx_clone,
    //         &con_clone,
    //     )
    //     .await
    //     .unwrap()
    // });

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
/// entirely through shared keys written by different tasks on different timers,
/// which is exactly the arrangement the module note above flags as the
/// architectural risk. If a key name, a state string or a handshake order drifts
/// on one side only, this is the test that notices.
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
    /// quiet. This is the idle-load property the whole `PERF` write-up is about,
    /// measured the only way that actually settles it: watch the keyspace.
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
