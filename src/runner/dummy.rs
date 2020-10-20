use super::*;
use r2r::*;
use std::io;
use std::sync::{Arc, Mutex};
use tokio::sync::mpsc::channel;
use tokio::time::{delay_for, interval, Duration, Instant};

/// A dummy ROS2 node (micro_sp inverse) is made here. Based on the provided model,
/// an inverse instance of micro_sp is generated that reacts to the main instance of
/// micro_sp for virtual commissioning purposes. Subscribers are generated for command
/// kind variables and publishers for measured kind variables. In Paradigm::Raar, the
/// command values "ref" are mapped to the "act_ref" measured variables and published
/// after a delay. These subs and pubs run asynchronously in their green threads.
pub async fn raar_dummy(ros_ctx: &Context, prob: &PlanningProblem) -> r2r::Node {
    let mut node = Node::create(ros_ctx.clone(), "dummy", "")
        .expect("Error 8fddc8c1-7cce-4cd7-97e0-c16438ac3a28: Creating ros node failed.");
    let problem = prob.clone();
    let vars = get_problem_vars(&problem);
    let msr_var_vals: Vec<EnumValue> = vars
        .iter()
        .filter(|x| x.kind == Kind::Measured)
        .map(|x| EnumValue::new(x, "dummy_value", None))
        .collect();
    let hnd_var_vals: Vec<EnumValue> = vars
        .iter()
        .filter(|x| x.kind == Kind::Handshake)
        .map(|x| EnumValue::new(x, "dummy_value", None))
        .collect();
    let cmd_var_vals: Vec<EnumValue> = vars
        .iter()
        .filter(|x| x.kind == Kind::Command)
        .map(|x| EnumValue::new(x, "dummy_value", None))
        .collect();
    // generate subscribers for Kind::Command kind variables
    let mut ros_receivers: Vec<(String, tokio::sync::mpsc::Receiver<String>)> = vec![];
    for v in &cmd_var_vals {
        let (mut tx, rx) = channel::<String>(10);
        ros_receivers.push((serde_json::to_string(&v).unwrap_or_default(), rx));
        let sub = move |x: r2r::std_msgs::msg::String| {
            tx.try_send(x.data).unwrap_or_default();
        };
        let _subref = node
            .subscribe(&format!("/{}", v.var.name), Box::new(sub))
            .expect("69900836-cc9c-4ea5-9f2f-1f585dae70b1: Creating subscribers failed.");
    }
    // generate publishers for Kind::Measured kind variables
    let mut ros_senders: Vec<(String, tokio::sync::mpsc::Sender<String>)> = vec![];
    for v in msr_var_vals.clone() {
        let publisher = node
            .create_publisher::<std_msgs::msg::String>(&format!("/{}", v.var.name))
            .expect("Error f93c6d99-5725-467a-8a96-e49f72b3485f: Creating publishers failed.");
        let (tx, rx) = channel::<String>(10);
        ros_senders.push((serde_json::to_string(&v).unwrap_or_default(), tx));
        tokio::task::spawn(async {
            let writer = runner::publisher::publisher(publisher, rx);
            let _res = tokio::try_join!(writer);
        });
    }
    // generate publishers for Kind::Handshake kind variables
    let mut ros_handshakers: Vec<(String, tokio::sync::mpsc::Sender<String>)> = vec![];
    for v in hnd_var_vals.clone() {
        let publisher = node
            .create_publisher::<std_msgs::msg::String>(&format!("/{}", v.var.name))
            .expect("Error f93c6d99-5725-467a-8a96-e49f72b3485f: Creating publishers failed.");
        let (tx, rx) = channel::<String>(10);
        ros_handshakers.push((serde_json::to_string(&v).unwrap_or_default(), tx));
        tokio::task::spawn(async {
            let writer = runner::publisher::publisher(publisher, rx);
            let _res = tokio::try_join!(writer);
        });
    }

    tokio::task::spawn(async {
        let recv = mapper(ros_receivers, ros_handshakers, ros_senders);
        let _res = tokio::try_join!(recv);
    });
    node
}

async fn mapper(
    ros_receivers: Vec<(String, tokio::sync::mpsc::Receiver<String>)>,
    ros_handshakers: Vec<(String, tokio::sync::mpsc::Sender<String>)>,
    ros_senders: Vec<(String, tokio::sync::mpsc::Sender<String>)>,
) -> io::Result<()> {
    let mut command_list = vec![];
    for r in ros_receivers {
        let past_time = Instant::now().checked_sub(Duration::new(6, 0));
        let amkvp = Arc::new(Mutex::new((r.0.clone(), past_time.unwrap())));
        let amkvp1 = amkvp.clone();
        tokio::task::spawn(async {
            let receiver = receiver::receiver(amkvp, r.1);
            let _res = tokio::try_join!(receiver);
        });
        command_list.push(amkvp1);
    }

    let mut handshake_list = vec![];
    for r in ros_handshakers {
        let past_time = Instant::now().checked_sub(Duration::new(6, 0));
        let amkvp = Arc::new(Mutex::new((r.0.clone(), past_time.unwrap())));
        let amkvp1 = amkvp.clone();
        tokio::task::spawn(async {
            let sender = sender::sender(amkvp, r.1, Arc::new(Mutex::new(0)));
            let _res = tokio::try_join!(sender);
        });
        handshake_list.push(amkvp1);
    }

    let mut measured_list = vec![];
    for r in ros_senders {
        let past_time = Instant::now().checked_sub(Duration::new(6, 0));
        let amkvp = Arc::new(Mutex::new((r.0.clone(), past_time.unwrap())));
        let amkvp1 = amkvp.clone();
        tokio::task::spawn(async {
            let sender = sender::sender(amkvp, r.1, Arc::new(Mutex::new(4000)));
            let _res = tokio::try_join!(sender);
        });
        measured_list.push(amkvp1);
    }

    // let mut delay_acc = ["dummy_value"; 100];

    loop {
        let looping_now = Instant::now();
        // delay_for(Duration::from_millis(3000)).await;
        let command_vec = &command_list
            .iter()
            .map(|x| {
                let des: EnumValue = serde_json::from_str(&x.lock().unwrap().0).unwrap();
                let duration = match looping_now.checked_duration_since(x.lock().unwrap().1) {
                    Some(x) => x,
                    None => Duration::new(6, 0),
                };
                EnumValue::new(&des.var, &des.val, Some(&duration))
            })
            .collect::<Vec<EnumValue>>();

        let _measured_vec = &measured_list
            .iter()
            .map(|x| {
                let des: EnumValue = serde_json::from_str(&x.lock().unwrap().0).unwrap();
                let dummy = EnumValue::new(&des.var, "dummy_value", None);
                let get_val: &EnumValue = command_vec
                    .iter()
                    .find(|y| y.var.r#type == des.var.r#type)
                    .unwrap_or(&dummy);
                let update = EnumValue::new(&des.var, &get_val.val, None);

                *x.lock().unwrap() = (serde_json::to_string(&update).unwrap(), Instant::now());
                EnumValue::new(
                    &des.var,
                    &update.val,
                    Some(&looping_now.saturating_duration_since(x.lock().unwrap().1)),
                )
            })
            .collect::<Vec<EnumValue>>();

        let _handshake_list = &handshake_list
            .iter()
            .map(|x| {
                let des: EnumValue = serde_json::from_str(&x.lock().unwrap().0).unwrap();
                let dummy = EnumValue::new(&des.var, "dummy_value", None);
                let get_val: &EnumValue = command_vec
                    .iter()
                    .find(|y| y.var.r#type == des.var.r#type)
                    .unwrap_or(&dummy);
                let update = EnumValue::new(&des.var, &get_val.val, None);
                *x.lock().unwrap() = (serde_json::to_string(&update).unwrap(), Instant::now());
                EnumValue::new(
                    &des.var,
                    &update.val,
                    Some(&looping_now.saturating_duration_since(x.lock().unwrap().1)),
                )
            })
            .collect::<Vec<EnumValue>>();

        let mut interval = interval(Duration::from_millis(25));
        interval.tick().await;
        interval.tick().await;
    }
}
