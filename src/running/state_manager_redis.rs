use std::collections::HashMap;

use crate::*;
use ordered_float::OrderedFloat;
use redis::{AsyncCommands, Client, Value};
use tokio::sync::{mpsc, oneshot};

/// Available commands that the async tasks can ask from the state manager.
pub enum StateManagement {
    GetState(oneshot::Sender<State>),
    Get((String, oneshot::Sender<SPValue>)),
    SetPartialState(State),
    Set((String, SPValue)),
    // Update
}

// // Serialize to JSON (which embeds type info)
// let serialized = serde_json::to_string(&value).unwrap();
// println!("Serialized: {}", serialized);
// // This might print: {"type":"Int64","value":42}

// // Deserialize back to SPValue
// let deserialized: SPValue = serde_json::from_str(&serialized).unwrap();
// println!("Deserialized: {:?}", deserialized);

// fn sp_value_to_serialied_string(value: &SPValue) -> String {
//     match value {
//         SPValue::Bool(val) => serde_json::to_string(&val).unwrap(),
//         SPValue::Float64(ord) => serde_json::to_string(&ord.into_inner()).unwrap(),
//         SPValue::Int64(int) => serde_json::to_string(&int).unwrap(),
//         SPValue::String(str) => serde_json::to_string(&str).unwrap(),
//         SPValue::Array(_, spvalues) => serde_json::to_string(
//             &spvalues
//                 .iter()
//                 .map(|x| sp_value_to_serialied_string(x))
//                 .collect::<Vec<String>>(),
//         )
//         .unwrap(),
//         SPValue::Time(time) => serde_json::to_string(&time).unwrap(),
//         // Could be that I would need specialized unknown for all the types because I want INT:UNKNOWN
//         SPValue::Unknown(SPValueType::Array) => serde_json::to_string("UNKNOWN").unwrap(),
//     }
// }

// fn redis_value_to_sp_value(value: &Value) -> SPValue {
//     match value {
//         Value::Nil => SPValue::UNKNOWN,
//         Value::Int(int) => SPValue::Int64(*int),
//         Value::BulkString(_) => todo!(),
//         Value::Array(values) => SPValue::Array(
//             SPValueType::UNKNOWN,
//             values.iter().map(|x| redis_value_to_sp_value(x)).collect(),
//         ),
//         Value::SimpleString(str) => SPValue::String(str.clone()),
//         Value::Okay => todo!(),
//         Value::Map(_) => todo!(),
//         Value::Attribute {
//             data: _,
//             attributes: _,
//         } => todo!(),
//         Value::Set(_) => todo!(),
//         Value::Double(float) => SPValue::Float64(OrderedFloat::from(float.clone())),
//         Value::Boolean(bool) => SPValue::Bool(*bool),
//         Value::VerbatimString { format: _, text: _ } => todo!(),
//         Value::BigNumber(_) => todo!(),
//         Value::Push { kind: _, data: _ } => todo!(),
//         Value::ServerError(_) => todo!(),
//     }
// }

// async fn redis_subscriber(command_sender: mpsc::Sender<StateManagement>, redis_client_arc: Arc<Mutex<redis::Client>>) {
//     let redis_client = redis_client_arc.lock().unwrap().clone();
//     let mut pubsub_con = redis_client
//         .get_async_pubsub()
//         .await
//         .expect("Failed to establish a pubsub connection.");
//     pubsub_con
//         .psubscribe("__keyspace@0__:*")
//         .await
//         .expect("Failed to subscribe to Redis KEA changes.");
// }

pub async fn redis_state_manager(
    mut receiver: mpsc::Receiver<StateManagement>,
    // redis_client: redis::Client,
    mut state: State,
) {
    let redis_client =
        Client::open("redis://127.0.0.1/").expect("Failed to instantiate redis client.");
    let mut con = redis_client
        .get_multiplexed_async_connection()
        .await
        .expect("Failed to establish Redis connection.");
    let _: () = redis::cmd("CONFIG")
        .arg("SET")
        .arg("notify-keyspace-events")
        .arg("KEA")
        .query_async(&mut con)
        .await
        .expect("Failed to set notify-keyspace-events for Redis.");

    // First populate the redis DB with the state.
    for (var, assignment) in state.state.clone() {
        if let Err(e) = con
            .set::<_, String, String>(&var, serde_json::to_string(&assignment.val).unwrap())
            .await
        {
            eprintln!("Failed to set {}: {:?}", var, e);
        }
    }

    // redis::cmd("CONFIG")
    //     .arg("SET")
    //     .arg("notify-keyspace-events")
    //     .arg("KEA")
    //     .query_async::<_, ()>(&mut con)
    //     .await
    //     .expect("Failed to set notify-keyspace-events for Redis.");
    // let mut pubsub_con = redis_client
    //     .get_async_pubsub()
    //     .await
    //     .expect("Failed to establish a pubsub connection.");

    // // Subscribe to keyspace notification channel for DB 0, all keys
    // pubsub_con
    //     .psubscribe("__keyspace@0__:*")
    //     .await
    //     .expect("Failed to subscribe to Redi KEA changes.");
    // Listen for commands on the channel
    while let Some(command) = receiver.recv().await {
        match command {
            StateManagement::GetState(response_sender) => {
                let keys: Vec<String> = con.keys("*").await.expect("Failed to get all keys.");

                let values: Vec<Option<String>> = con
                    .mget(&keys)
                    .await
                    .expect("Failed to get values for all keys.");

                let mut map: HashMap<String, SPAssignment> = HashMap::new();
                for (key, maybe_value) in keys.into_iter().zip(values.into_iter()) {
                    if let Some(value) = maybe_value {
                        let var = state.get_assignment(&key).var;
                        let new_assignment =
                            SPAssignment::new(var, serde_json::from_str(&value).unwrap());
                        map.insert(key, new_assignment);
                    }
                }

                let _ = response_sender.send(State { state: map });
            }

            StateManagement::Get((var, response_sender)) => {
                match con.get::<_, Option<String>>(&var).await {
                    Ok(val) => {
                        match val {
                            Some(redis_value) => {
                                let _ = response_sender.send(serde_json::from_str(&redis_value).unwrap());
                            }
                            None => panic!("Var doesn't exist!"),
                        }
                        panic!("Var doesn't exist!")
                    }
                    Err(e) => {
                        eprintln!("Failed to get {}: {:?}", var, e);
                        panic!("Var doesn't exist!")
                    }
                }
            }

            StateManagement::SetPartialState(partial_state) => {
                for (var, assignment) in partial_state.state {
                    state = state.update(&var, assignment.val.clone());
                    if let Err(e) = con
                        .set::<_, String, Value>(
                            &var,
                            serde_json::to_string(&assignment.val).unwrap(),
                        )
                        .await
                    {
                        eprintln!("Failed to set {}: {:?}", var, e);
                        panic!("!")
                    }
                }
            }

            StateManagement::Set((var, val)) => {
                state = state.update(&var, val.clone());
                if let Err(e) = con
                    .set::<_, String, Value>(&var, serde_json::to_string(&val).unwrap())
                    .await
                {
                    eprintln!("Failed to set {}: {:?}", var, e);
                    panic!("!")
                }
            }
        }
    }
}

// use redis::{AsyncCommands, Client, RedisResult};
// use tokio_stream::StreamExt; // to easily iterate over incoming messages

// #[tokio::main]
// async fn main() -> RedisResult<()> {
//     // 1) Create a client
//     let client = Client::open("redis://127.0.0.1/")?;

//     // 2) (Optional) Enable keyspace notifications at runtime
//     {
//         let mut con = client.get_async_connection().await?;
//         redis::cmd("CONFIG")
//             .arg("SET")
//             .arg("notify-keyspace-events")
//             .arg("KEA")
//             .query_async(&mut con)
//             .await?;
//     }

//     // 3) Get a dedicated pubsub connection
//     let mut pubsub_con = client
//         .get_async_connection()
//         .await?
//         .into_pubsub();

//     // 4) Subscribe to keyspace notification channel for DB 0, all keys
//     //    e.g., __keyspace@0__:*   or  __keyevent@0__:*  (depending on whether you want key names or event types)
//     pubsub_con.psubscribe("__keyspace@0__:*").await?;

//     // 5) Turn pubsub into a stream
//     let mut stream = pubsub_con.on_message();

//     println!("Subscribed. Listening for key changes in DB=0 ...");

//     // 6) Read messages in a loop
//     while let Some(msg) = stream.next().await {
//         // The payload is the event type, e.g. "set", "expired", "hset", etc.
//         let payload: String = msg.get_payload()?;
//         // The channel name includes the key, e.g. "__keyspace@0__:my_key"
//         let channel = msg.get_channel_name();

//         println!("Received event: channel='{}' payload='{}'", channel, payload);

//         // If you want to parse out the key name:
//         // channel format is "__keyspace@<db>__:<key>", so let's remove the prefix
//         // to get the actual key that changed.
//         if let Some(stripped) = channel.strip_prefix("__keyspace@0__:") {
//             let key = stripped;
//             println!("  -> Key that changed: {}", key);
//             println!("  -> Event type: {}", payload);
//         }
//     }

//     Ok(())
// }
