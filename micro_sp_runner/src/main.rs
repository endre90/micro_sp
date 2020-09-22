use tokio::prelude::*;
use r2r::*;
use tokio::sync::mpsc::channel;
use micro_sp_tools::*;
mod emmiter;
mod model;

#[tokio::main]
async fn main() -> io::Result<()> {
    let ros_ctx = Context::create().expect("panic");
    let mut node = Node::create(ros_ctx, "testnode", "").expect("panic");

    let problem = model::model();
    let vars = GetProblemVars::new(&problem);
    let cmd_vars: Vec<EnumVariable> = vars.iter().filter(|x| x.kind == ControlKind::Command).map(|x| x.clone()).collect();

    let mut senders: Vec<(String, tokio::sync::mpsc::Sender<String>)> = vec!();
    for v in cmd_vars.clone() {
        let publisher = node.create_publisher::<std_msgs::msg::String>(&format!("/{}", v.name))
            .expect("Error f93c6d99-5725-467a-8a96-e49f72b3485f: Creating publishers failed.");
        let (tx, rx) = channel::<String>(10);
        senders.push((v.name, tx));
        tokio::task::spawn(async{
            let writer = emmiter::emmiter(publisher, rx);
            let _res = tokio::try_join!(writer);
        });
    }

    loop {
        for v in &cmd_vars {
            for s in &senders {
                if v.name == s.0 {
                    s.1.clone().try_send(s.0.to_string()).unwrap_or_default();
                }
            }
        }
        node.spin_once(std::time::Duration::from_millis(10));
    }
}