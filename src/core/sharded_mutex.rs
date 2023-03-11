use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::sync::{Mutex, Arc};
use crate::State;

pub type StateShard = State;

// // might need a hashmap after all...
// #[derive(Debug)]
// pub struct ShardedMutex {
//     pub mutexes: Vec<Mutex<StateShard>>,
// }

// impl ShardedMutex {
//     pub fn new(num_shards: usize) -> Self {
//         let mut mutexes = Vec::with_capacity(num_shards);
//         for _ in 0..num_shards {
//             mutexes.push(Mutex::new(StateShard::new()));
//         }
//         Self { mutexes }
//     }

//     // pub fn from(state: State) -> Self {
//     //     let keys = state.state.keys();
//     //     let mut mutexes = Vec::with_capacity(state.state.len());
//     //     for key in keys {
//     //         let value = state.state.get(key).unwrap();
//     //         let new_state_shard = StateShard {
//     //             state: HashMap::from([(key.to_owned(), value.to_owned())]),
//     //         };
//     //         mutexes.push(Mutex::new(new_state_shard));
//     //     }
//     //     Self { mutexes }
//     // }

//     // let shard = db[hash(key) % db.len()].lock().unwrap();
//     // shard.insert(key, value);

//     // pub fn from(state: State) -> Self {
//     //     let keys = state.state.keys();
//     //     let new_sharded_mutex = ShardedMutex::new(keys.len());
//     //     for key in keys {
//     //         let mut shard = new_sharded_mutex.lock(key);
//     //         shard.state.insert(key.to_string(), state.state.get(key).unwrap().clone());
//     //     }
//     //     new_sharded_mutex
//     // }

//     pub fn lock(&self, key: &str) -> std::sync::MutexGuard<State> {
//         let shard_index = self.get_shard_index(key);
//         self.mutexes[shard_index].lock().unwrap()
//     }

//     pub fn collect_all(&self) -> Arc<Mutex<State>>  {
//         let mut result = HashMap::new();

//         for mutex in &self.mutexes {
//             let shard = mutex.lock().unwrap();
//             for (key, value) in shard.state.clone() {
//                 result.insert(key.clone(), value.clone());
//             }
//         }

//         let arc = Arc::new(Mutex::new(State {state: result}));
//         arc

//     }

//     pub fn get_shard_index(&self, key: &str) -> usize {
//         let mut hasher = std::collections::hash_map::DefaultHasher::new();
//         key.hash(&mut hasher);
//         hasher.finish() as usize % self.mutexes.len()
//     }
// }

// might need a hashmap after all...
#[derive(Debug)]
pub struct ShardedMutex {
    pub mutexes: Vec<Mutex<HashMap<String, StateShard>>>,
}

impl ShardedMutex {
    pub fn new(num_shards: usize) -> Self {
        let mut mutexes = Vec::with_capacity(num_shards);
        for _ in 0..num_shards {
            mutexes.push(Mutex::new(HashMap::new()));
        }
        Self { mutexes }
    }

    pub fn lock(&self, key: &str) -> std::sync::MutexGuard<HashMap<String, StateShard>> {
        let shard_index = self.get_shard_index(key);
        self.mutexes[shard_index].lock().unwrap()
    }

    // pub fn collect_all(&self) -> Arc<Mutex<State>>  {
    //     let mut result = HashMap::new();

    //     for mutex in &self.mutexes {
    //         let shard = mutex.lock().unwrap();
    //         for (key, value) in shard.state.clone() {
    //             result.insert(key.clone(), value.clone());
    //         }
    //     }

    //     let arc = Arc::new(Mutex::new(State {state: result}));
    //     arc

    // }

    pub fn get_shard_index(&self, key: &str) -> usize {
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        key.hash(&mut hasher);
        hasher.finish() as usize % self.mutexes.len()
    }
}