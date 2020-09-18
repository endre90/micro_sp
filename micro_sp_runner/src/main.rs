use tokio::prelude::*;
use r2r::*;
use tokio::sync::mpsc::channel;
use std::sync::{Arc, Mutex};
use micro_sp_tools::*;
mod emmiter;
mod model;
// use super::*;

// pub async fn runner(prob: &PlanningProblem) -> () {
//     let ros_ctx = Context::create().expect("panic");
//     let mut node = Node::create(ros_ctx, "testnode", "").expect("panic");
//     let vars = GetProblemVars::new(&prob);
//     let mut pubs = vec!();
//     for v in &vars {
//         pubs.push(node.create_publisher::<std_msgs::msg::String>(&format!("/{}", v.name)).expect("asdf"));
//     }    
// }

#[tokio::main]
async fn main() -> io::Result<()> {
    let ros_ctx = Context::create().expect("panic");
    let mut node = Node::create(ros_ctx, "testnode", "").expect("panic");
    let publisher = node.create_publisher::<std_msgs::msg::String>("/test2").expect("asdf");

    let problem = model::model();
    let vars = GetProblemVars::new(&problem);
    // let mut pubs = vec!();
    let pubs: Vec<r2r::Publisher<r2r::std_msgs::msg::String>> = vars.iter().map(|x| node.create_publisher::<std_msgs::msg::String>(&format!("/{}", x.name)).expect("asdf")).collect();
    // for v in &vars {
    //     pubs.push(node.create_publisher::<std_msgs::msg::String>(&format!("/{}", v.name)).expect("asdf"));
    // }    

    let (mut tx, rx) = channel::<String>(10);
    tokio::task::spawn(async {
        // for p in pubs {
            
            let writer = emmiter::emmiter(publisher, rx);
            let _res = tokio::try_join!(writer);
        // }
        
    });

    loop {
        tx.try_send("somedata".to_string()).unwrap_or_default();
        node.spin_once(std::time::Duration::from_millis(10));
    }
}
