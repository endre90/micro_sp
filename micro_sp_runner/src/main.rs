use r2r::*;
use std::sync::Mutex;
use std::sync::Arc;
use tokio::sync::mpsc::channel;
use micro_sp_tools::*;
use lib::{KeyValuePair, State};
use std::io;
mod runner;
mod receiver;
mod emmiter;
mod model;

#[tokio::main]
async fn main() -> io::Result<()> {
    let ros_ctx = Context::create()
        .expect("Error 3357ef39-2674-46c8-9841-bd126e70e059: Creating ros context failed.");
    let mut node = Node::create(ros_ctx, "micro_sp", "")
        .expect("Error 8fddc8c1-7cce-4cd7-97e0-c16438ac3a28: Creating ros node failed.");

    let problem = model::model();
    let vars = GetProblemVars::new(&problem);
    let msr_vars: Vec<EnumVariable> = vars.iter().filter(|x| x.kind == ControlKind::Measured).map(|x| x.clone()).collect();
    let cmd_vars: Vec<EnumVariable> = vars.iter().filter(|x| x.kind == ControlKind::Command).map(|x| x.clone()).collect();

    // let measured_values = msr_vars.iter().map(|x| KeyValuePair::new(x.name.as_str(), "dummy_value")).collect();
    // let measured_state = State::new(&measured_values);
    // println!("{:?}", measured_state);

    // generate subscribers for ControlKind::Measured kind variables
    let mut ros_receivers: Vec<(String, KeyValuePair, tokio::sync::mpsc::Receiver<String>)> = vec!();
    for v in &msr_vars {
        let (mut tx, rx) = channel::<String>(10);
        ros_receivers.push((v.name.clone(), KeyValuePair::new(&v.name, "dummy_value"), rx));
        let sub = move |x: r2r::std_msgs::msg::String| {
            tx.try_send(x.data).unwrap_or_default();
        };
        let _subref = node.subscribe(&format!("/{}", v.name), Box::new(sub))
            .expect("69900836-cc9c-4ea5-9f2f-1f585dae70b1: Creating subscribers failed.");
    }  

    // generate publishers for ControlKind::Command kind variables
    let mut ros_senders: Vec<(String, tokio::sync::mpsc::Sender<String>)> = vec!();
    for v in cmd_vars.clone() {
        let publisher = node.create_publisher::<std_msgs::msg::String>(&format!("/{}", v.name))
            .expect("Error f93c6d99-5725-467a-8a96-e49f72b3485f: Creating publishers failed.");
        let (tx, rx) = channel::<String>(10);
        ros_senders.push((v.name, tx));
        tokio::task::spawn(async{
            let writer = emmiter::emmiter(publisher, rx);
            let _res = tokio::try_join!(writer);
        });
    }

    tokio::task::spawn(async{
        let recv = runner::runner(problem, ros_receivers, ros_senders);
        let _res = tokio::try_join!(recv);
    });

    loop {

        // for t in &measured_list {
        //     println!("{:?}", *t.lock().unwrap());
        // }

        
        node.spin_once(std::time::Duration::from_millis(10));
    }
}