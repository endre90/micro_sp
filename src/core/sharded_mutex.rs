use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::sync::Mutex;
use crate::State;

// define alias from State Shard
type StateShard = State;

#[derive(Debug)]
pub struct ShardedMutex {
    pub mutexes: Vec<Mutex<StateShard>>,
}

impl ShardedMutex {
    pub fn new(num_shards: usize) -> Self {
        let mut mutexes = Vec::with_capacity(num_shards);
        for _ in 0..num_shards {
            mutexes.push(Mutex::new(StateShard::new()));
        }
        Self { mutexes }
    }

    pub fn from(state: State) -> Self {
        let keys = state.state.keys();
        let mut mutexes = Vec::with_capacity(state.state.len());
        for key in keys {
            let value = state.state.get(key).unwrap();
            let new_state_shard = StateShard {
                state: HashMap::from([(key.to_owned(), value.to_owned())]),
            };
            mutexes.push(Mutex::new(new_state_shard));
        }
        Self { mutexes }
    }

    pub fn lock(&self, key: &str) -> std::sync::MutexGuard<StateShard> {
        let shard_index = self.get_shard_index(key);
        self.mutexes[shard_index].lock().unwrap()
    }

    pub fn get_shard_index(&self, key: &str) -> usize {
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        key.hash(&mut hasher);
        hasher.finish() as usize % self.mutexes.len()
    }
}