use tokio::sync::mpsc::channel;
use micro_sp_tools::*;
use serde_json::*;
use std::io;
mod runner;
mod receiver;
mod test_receiver;
mod sender;
mod publisher;
mod model;
use r2r::*;

#[tokio::main]
async fn main() -> io::Result<()> {
    let ros_ctx = Context::create()
        .expect("Error 3357ef39-2674-46c8-9841-bd126e70e059: Creating ros context failed.");
    let mut node = Node::create(ros_ctx, "micro_sp", "")
        .expect("Error 8fddc8c1-7cce-4cd7-97e0-c16438ac3a28: Creating ros node failed.");

    let problem = model::model();
    let vars = get_problem_vars(&problem);
    let msr_vars: Vec<EnumVariable> = vars.iter().filter(|x| x.kind == ControlKind::Measured).map(|x| x.clone()).collect();
    let cmd_vars: Vec<EnumVariable> = vars.iter().filter(|x| x.kind == ControlKind::Command).map(|x| x.clone()).collect();

    let test_var = TestVariable::new("test1", "1", "d1", &vec!("1", "2", "fasdf"), Some(&TestParameter { name: "5".to_owned(), value: false }), &ControlKind::None);
    let test_var2 = TestVariable::new("test2", "3", "d2", &vec!("3", "4", "fasd5f"), Some(&TestParameter { name: "6".to_owned(), value: true }), &ControlKind::Measured);
    let test_vars = vec!(test_var, test_var2);
    // let serialized = serde_json::to_string(&test_var).unwrap();

    // test receivers
    // generate subscribers for ControlKind::Measured kind variables (maybe all? testing needed)
    let mut test_ros_receivers: Vec<(String, tokio::sync::mpsc::Receiver<String>)> = vec!();
    for v in &test_vars {
        let (mut tx, rx) = channel::<String>(10);
        test_ros_receivers.push((serde_json::to_string(&v).unwrap(), rx));
        let sub = move |x: r2r::std_msgs::msg::String| {
            tx.try_send(x.data).unwrap_or_default();
        };
        let _subref = node.subscribe(&format!("/{}", v.name), Box::new(sub))
            .expect("69900836-cc9c-4ea5-9f2f-1f585dae70b1: Creating subscribers failed.");
    }  

    // generate subscribers for ControlKind::Measured kind variables (maybe all? testing needed)
    let mut ros_receivers: Vec<(KeyValuePair, tokio::sync::mpsc::Receiver<String>)> = vec!();
    for v in &msr_vars {
        let (mut tx, rx) = channel::<String>(10);
        ros_receivers.push((KeyValuePair::dummy(&v.name.key), rx));
        let sub = move |x: r2r::std_msgs::msg::String| {
            tx.try_send(x.data).unwrap_or_default();
        };
        let _subref = node.subscribe(&format!("/{}", v.name.key), Box::new(sub))
            .expect("69900836-cc9c-4ea5-9f2f-1f585dae70b1: Creating subscribers failed.");
    }  

    // generate publishers for ControlKind::Command kind variables
    let mut ros_senders: Vec<(KeyValuePair, tokio::sync::mpsc::Sender<String>)> = vec!();
    for v in cmd_vars.clone() {
        let publisher = node.create_publisher::<std_msgs::msg::String>(&format!("/{}", v.name.key))
            .expect("Error f93c6d99-5725-467a-8a96-e49f72b3485f: Creating publishers failed.");
        let (tx, rx) = channel::<String>(10);
        ros_senders.push((KeyValuePair::dummy(&v.name.key), tx));
        tokio::task::spawn(async{
            let writer = publisher::publisher(publisher, rx);
            let _res = tokio::try_join!(writer);
        });
    }

    tokio::task::spawn(async{
        let recv = runner::runner(problem, ros_receivers, ros_senders, test_ros_receivers);
        let _res = tokio::try_join!(recv);
    });

    loop {       
        node.spin_once(std::time::Duration::from_millis(10));
    }
}