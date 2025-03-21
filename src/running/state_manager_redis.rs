use std::collections::HashMap;

use crate::*;
use redis::{AsyncCommands, Client};
use tokio::sync::{mpsc, oneshot};

/// Available commands that the async tasks can ask from the state manager.
pub enum StateManagement {
    GetState(oneshot::Sender<State>),
    Get((String, oneshot::Sender<SPValue>)),
    SetPartialState(State),
    Set((String, SPValue)),
    // Update
}

// fn sp_value_to_redis_value(value: &SPValue) -> Value {
//     match value {
//         SPValue::Bool(val) => Value::Boolean(*val),
//         SPValue::Float64(ord) => Value::Double(ord.clone().into()),
//         SPValue::Int64(int) => Value::Int(*int),
//         SPValue::String(str) => Value::SimpleString(str.clone()),
//         SPValue::Array(_, spvalues) => Value::Array(
//             spvalues
//                 .iter()
//                 .map(|x| sp_value_to_redis_value(x))
//                 .collect(),
//         ),
//         SPValue::Time(_) => todo!(),
//         SPValue::UNKNOWN => todo!(),
//     }
//     SPValue::to_string(&self, serializer)
// }

// fn redis_value_to_sp_value(value: &Value) -> SPValue {
//     match value {
//         Value::Nil => todo!(),
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
            .hset::<_, _, _, ()>("my_state", &var, assignment.val.to_string())
            .await
        {
            eprintln!("Failed to hset boolean {}: {:?}", var, e);
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
                match con.hgetall::<_, HashMap<String, String>>("my_state").await {
                    Ok(map) => {
                        let old_state = state.clone();
                        println!("OLD: {:#?}", old_state);
                        let new_state = State {
                            state: map
                                .iter()
                                .map(|(key, val)| {
                                    (
                                        key.clone(),
                                        SPAssignment::new(
                                            old_state.get_assignment(key).var,
                                            SPValue::from_string(val),
                                        ),
                                    )
                                })
                                .collect(),
                        };
                        println!("NEW: {:#?}", new_state);
                        let _ = response_sender.send(new_state);
                    }
                    Err(e) => {
                        eprintln!("Failed to hgetall: {:?}", e);
                        panic!("Var doesn't exist!")
                        // let _ = response_sender.send(State::new());
                    }
                }
            }

            StateManagement::Get((var, response_sender)) => {
                match con.hget::<_, _, Option<String>>("my_state", &var).await {
                    Ok(val) => {
                        match val {
                            Some(redis_value) => {
                                let _ = response_sender.send(SPValue::from_string(&redis_value));
                            }
                            None => panic!("Var doesn't exist!"),
                        }
                        panic!("Var doesn't exist!")
                    }
                    Err(e) => {
                        eprintln!("Failed to hget {}: {:?}", var, e);
                    }
                }
            }

            StateManagement::SetPartialState(partial_state) => {
                for (var, assignment) in partial_state.state {
                    if let Err(e) = con
                        .hset::<_, _, _, ()>("my_state", &var, assignment.val.to_string())
                        .await
                    {
                        eprintln!("Failed to hset boolean {}: {:?}", var, e);
                    }
                    state = state.update(&var, assignment.val)
                }
            }

            StateManagement::Set((var, new_val)) => {
                if let Err(e) = con
                    .hset::<_, _, _, ()>("my_state", &var, new_val.to_string())
                    .await
                {
                    eprintln!("Failed to hset boolean {}: {:?}", var, e);
                }
                state = state.update(&var, new_val)
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
