use micro_sp_tools::*;
use std::io;
use tokio::sync::mpsc::channel;
use tokio::time::Duration;
mod model;
mod publisher;
mod receiver;
mod runner;
mod sender;
mod cstate;
mod mstate;
use r2r::*;

#[tokio::main]
async fn main() -> io::Result<()> {
    let ros_ctx = Context::create()
        .expect("Error 3357ef39-2674-46c8-9841-bd126e70e059: Creating ros context failed.");
    let mut node = Node::create(ros_ctx, "micro_sp", "")
        .expect("Error 8fddc8c1-7cce-4cd7-97e0-c16438ac3a28: Creating ros node failed.");

    let problem = model::model();
    let vars = get_problem_vars(&problem);

    let msr_var_vals: Vec<EnumVariableValue> = vars
        .iter()
        .filter(|x| x.kind == ControlKind::Measured)
        .map(|x| EnumVariableValue::timed(x, "dummy_value", Duration::new(6, 0)))
        .collect();

    let cmd_var_vals: Vec<EnumVariableValue> = vars
        .iter()
        .filter(|x| x.kind == ControlKind::Command)
        .map(|x| EnumVariableValue::timed(x, "dummy_value", Duration::new(6, 0)))
        .collect();

    // generate subscribers for ControlKind::Measured kind variables (maybe all? testing needed)
    let mut ros_receivers: Vec<(String, tokio::sync::mpsc::Receiver<String>)> = vec![];
    for v in &msr_var_vals {
        let (mut tx, rx) = channel::<String>(10);
        ros_receivers.push((serde_json::to_string(&v).unwrap(), rx));
        let sub = move |x: r2r::std_msgs::msg::String| {
            tx.try_send(x.data).unwrap_or_default();
        };
        let _subref = node
            .subscribe(&format!("/{}", v.var.name), Box::new(sub))
            .expect("69900836-cc9c-4ea5-9f2f-1f585dae70b1: Creating subscribers failed.");
    }

    // generate publishers for ControlKind::Command kind variables
    let mut ros_senders: Vec<(String, tokio::sync::mpsc::Sender<String>)> = vec![];
    for v in cmd_var_vals.clone() {
        let publisher = node
            .create_publisher::<std_msgs::msg::String>(&format!("/{}", v.var.name))
            .expect("Error f93c6d99-5725-467a-8a96-e49f72b3485f: Creating publishers failed.");
        let (tx, rx) = channel::<String>(10);
        ros_senders.push((serde_json::to_string(&v).unwrap(), tx));
        tokio::task::spawn(async {
            let writer = publisher::publisher(publisher, rx);
            let _res = tokio::try_join!(writer);
        });
    }

    // // make a publisher for the global state
    // let state_publisher = node
    //     .create_publisher::<std_msgs::msg::String>("/state")
    //     .expect("Error f93c6d99-5725-467a-8a96-e49f72b3485f: Creating state publisher failed.");
    // let (tx, rx) = channel::<String>(10);
    // let state_publisher_data = (serde_json::to_string(&State::new()).unwrap(), tx);
    // tokio::task::spawn(async {
    //     let writer = publisher::publisher(state_publisher, rx);
    //     let _res = tokio::try_join!(writer);
    // });

    tokio::task::spawn(async {
        let recv = runner::runner(problem, ros_receivers, ros_senders);
        let _res = tokio::try_join!(recv);
    });

    loop {
        node.spin_once(std::time::Duration::from_millis(10));
    }
}
