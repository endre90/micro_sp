use micro_sp_tools::*;
use tokio::prelude::*;
use r2r::*;
use tokio::sync::mpsc::channel;

pub async fn runner(prob: &PlanningProblem) -> () {
    let ros_ctx = Context::create().expect("panic");
    let mut node = Node::create(ros_ctx, "testnode", "").expect("panic");
    let vars = GetProblemVars::new(&prob);
    let mut pubs = vec!();
    for v in &vars {
        pubs.push(node.create_publisher::<std_msgs::msg::String>(&format!("/{}", v.name)).expect("asdf"));
    }    
}