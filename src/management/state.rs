use crate::*;
use redis::{aio::MultiplexedConnection};

mod build_state;
mod get_full_state;
mod get_sp_value;
mod get_state_for_keys;
mod set_sp_value;
mod set_state;
mod remove_sp_value;
mod remove_sp_values;
mod flush_state;

pub struct StateManager {}

impl StateManager {
    pub async fn get_full_state(con: &mut MultiplexedConnection) -> Option<State> {
        get_full_state::get_full_state(con).await
    }

    pub async fn get_state_for_keys(
        con: &mut MultiplexedConnection,
        keys: &Vec<String>,
        log_target: &str
    ) -> Option<State> {
        get_state_for_keys::get_state_for_keys(con, keys, &log_target).await
    }

    pub async fn get_sp_value(con: &mut MultiplexedConnection, var: &str) -> Option<SPValue> {
        get_sp_value::get_sp_value(con, var).await
    }

    pub async fn set_state(con: &mut MultiplexedConnection, state: &State) {
        set_state::set_state(con, state).await
    }

    pub async fn set_sp_value(con: &mut MultiplexedConnection, key: &str, value: &SPValue) {
        set_sp_value::set_sp_value(con, key, value).await
    }

    pub async fn remove_sp_value(con: &mut MultiplexedConnection, key: &str) {
        remove_sp_value::remove_sp_value(con, key).await
    }

    pub async fn remove_sp_values(con: &mut MultiplexedConnection, keys: &[String]) {
        remove_sp_values::remove_sp_values(con, keys).await
    }

    pub fn build_state(keys: Vec<String>, values: Vec<Option<String>>) -> State {
        build_state::build_state(keys, values)
    }

    pub async fn flush_state(con: &mut MultiplexedConnection) {
        flush_state::flush_state(con).await
    }
}