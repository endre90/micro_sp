use crate::*;
use serde::{Deserialize, Serialize};
use std::hash::{Hash, Hasher};
use std::time::SystemTime;
use std::{collections::HashMap, fmt};

/// Represents the current state of the system.
///
/// PERF (biggest single win in the crate): `State` is treated as an immutable
/// persistent value (every mutation returns a brand new `State`), but it is
/// backed by a plain `std::collections::HashMap`, which has *no* structural
/// sharing. Every `update()` / `add()` / `remove()` therefore deep-copies the
/// entire map plus every `SPAssignment` (which itself owns a `String` name, an
/// `SPVariable` with another `String`, and an `SPValue` that may be a whole
/// `Vec` or `SPTransformStamped`). A single transition with 4 actions clones
/// the full system state 4 times; a SOP tick with 20 operations clones it
/// hundreds of times. Suggested fixes, in increasing order of effort:
///   1. Add `&mut self` variants (`update_mut`, `add_mut`, `remove_mut`) and use
///      them inside the runners / `Transition::take` / `Action::assign`, keeping
///      the current by-value API only where a fresh copy is genuinely wanted.
///   2. Swap `HashMap<String, SPAssignment>` for a persistent map with
///      structural sharing (`im::HashMap` / `rpds::HashTrieMap`). Then `update`
///      becomes O(log n) with no bulk copying and the current "return a new
///      State" style becomes cheap and correct.
///   3. Store values as `Arc<SPValue>` (and names as `Arc<str>`) so cloning an
///      assignment is a refcount bump instead of a heap copy. This alone kills
///      most of the cost for `Transform`/`Array`/`Map` valued variables.
/// PERF: the key type is `String` and *every* accessor in the runners is called
/// as `state.get_x(&format!("{}_information", op.name), ..)`. That is a fresh
/// heap allocation per variable per tick, and there are dozens per operation
/// per tick. Suggested: precompute the key strings once when an operation
/// becomes active (a small `OperationKeys { state, information, elapsed_exec,
/// .. }` struct held next to the `Operation` in `active_auto_ops`) and pass
/// `&str` slices in, or intern names as `Arc<str>` and key the map on those.
/// PERF: consider `HashMap<..., ..., ahash::RandomState>` (or `FxHashMap`).
/// The default SipHash hasher is DoS-resistant but ~2-3x slower than needed for
/// an internal, non-adversarial keyspace that is hashed on every lookup.
#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
pub struct State {
    pub state: HashMap<String, SPAssignment>,
}

/// PERF: this hash allocates a `Vec` of every key and sorts it on *every* hash
/// call. `bfs_operation_planner` keeps a `HashSet<State>` and hashes a full
/// state per expanded node, so planning is O(nodes * n log n) in hashing alone
/// - a large part of the CPU spike when a plan is requested. Suggested: use an
/// order-independent commutative combiner instead of sorting, e.g. XOR/wrapping
/// -add the per-entry hashes: `self.state.iter().fold(0u64, |acc, (k, v)| acc ^
/// hash_one((k, &v.val)))`, which is O(n) with no allocation. Better still, let
/// the planner hash only the *planning-relevant* variables (the union of
/// `get_all_var_keys()` over the operation model) rather than the whole state -
/// runner bookkeeping like `*_elapsed_executing_ms` changes constantly and
/// makes otherwise-identical planning states hash differently, which both slows
/// the search and defeats the `visited` set.
impl Hash for State {
    fn hash<H: Hasher>(&self, s: &mut H) {
        let mut keys: Vec<&String> = self.state.keys().collect();
        keys.sort(); 

        for key in keys {
            key.hash(s);
            
            if let Some(assignment) = self.state.get(key) {
                assignment.var.hash(s);
                assignment.val.hash(s);
            }
        }
    }
}

impl State {
    pub fn new() -> State {
        let state = HashMap::new();
        State { state }
    }

    pub fn from_vec(vec: &Vec<(SPVariable, SPValue)>) -> State {
        let mut state = HashMap::new();
        vec.iter().for_each(|(var, val)| {
            state.insert(
                var.name.clone(),
                SPAssignment {
                    var: var.clone(),
                    val: val.clone(),
                },
            );
        });
        State { state }
    }

    /// Get the updated values between two states.
    /// pub fn get_changed_values(&self, other_state: &State) -> HashMap<SPVariable, (SPValue, SPValue)> {
    pub fn get_diff_values(&self, other_state: &State) -> HashMap<SPVariable, (SPValue, SPValue)> {
        let mut changed_values = HashMap::new();

        for (key, self_assignment) in &self.state {
            if let Some(other_assignment) = other_state.state.get(key) {
                if self_assignment.val != other_assignment.val {
                    changed_values.insert(
                        self_assignment.var.clone(),
                        (self_assignment.val.clone(), other_assignment.val.clone()),
                    );
                }
            }
        }

        changed_values
    }

    pub fn get_diff_variables(&self, other_state: &State) -> Vec<SPVariable> {
        let mut uncommon_vars = Vec::new();

        for (key, assignment) in &self.state {
            if !other_state.state.contains_key(key) {
                uncommon_vars.push(assignment.var.clone());
            }
        }

        for (key, assignment) in &other_state.state {
            if !self.state.contains_key(key) {
                uncommon_vars.push(assignment.var.clone());
            }
        }

        uncommon_vars
    }

    // Make a new partial state that only consists of updates.
    //
    // PERF: this is the right idea (only ship deltas to Redis) but it iterates
    // `new_state` in full and hashes every key against `self` on every tick of
    // every runner. With `get_full_state` feeding it, that is O(all variables)
    // per runner per tick even when nothing changed. Suggested: have the runners
    // record which keys they actually touched (a `Vec<String>`/`SmallVec` of
    // dirty keys appended by `update_mut`) and build the delta from that list
    // instead of diffing the whole state. That turns a full-state scan into a
    // handful of lookups and is the change that most directly reduces the idle
    // CPU floor of the runner tasks.
    // PERF: `new_assignment.clone()` copies the `SPVariable` as well as the
    // value; only the value is ever written to Redis (see `set_state`), so a
    // `Vec<(&str, &SPValue)>` return type would avoid the copy entirely.
    pub fn get_diff_partial_state(&self, new_state: &State) -> State {
        let mut updated_assignments = HashMap::new();
        for (key, new_assignment) in &new_state.state {
            if let Some(old_assignment) = self.state.get(key) {
                if old_assignment.val != new_assignment.val {
                    updated_assignments.insert(key.clone(), new_assignment.clone());
                }
            }
        }

        State {
            state: updated_assignments,
        }
    }

    // Make a new partial state that only consists of updates.
    //
    // PERF: same full-scan-per-tick cost as `get_diff_partial_state`; see the
    // dirty-key suggestion there. Note the two functions differ only in the
    // `else` branch, so a single `fn diff(&self, new: &State, include_new: bool)`
    // would halve the code without changing behaviour.
    pub fn get_diff_partial_state_and_add_missing(&self, new_state: &State) -> State {
        // let mut updated_assignments = HashMap::new();
        let mut updated_state = State::new();
        for (key, new_assignment) in &new_state.state {
            if let Some(old_assignment) = self.state.get(key) {
                if old_assignment.val != new_assignment.val {
                    updated_state
                        .state
                        .insert(key.clone(), new_assignment.clone());
                }
            } else {
                updated_state
                    .state
                    .insert(new_assignment.var.name.to_string(), new_assignment.clone());
            }
        }

        updated_state
        // State {
        //     state: updated_assignments,
        // }
    }

    // ------------------------------------------------------------------
    // In-place mutation API.
    //
    // `State` is written as a persistent value - `add`/`remove`/`update`/
    // `extend` each return a brand new `State` - but it is backed by a plain
    // `HashMap`, which has no structural sharing. Every one of those calls
    // therefore deep-copies the whole map. Chaining them (which the runners do
    // constantly: `process_operation` ends with three chained `update`s, the
    // goal runner with nine, `add_operation_meta_tracking_variables` calls
    // `add` five times per operation in a loop) multiplies that cost by the
    // length of the chain.
    //
    // The `*_mut` methods below do the same work against `&mut self` with zero
    // copying. The owned methods are kept - the API and all existing call sites
    // are unchanged - but are now thin wrappers that clone exactly once and
    // then delegate, so even code that has not been migrated got cheaper.
    //
    // Prefer the `*_mut` form anywhere a `State` is being built up or is
    // already owned locally; use the owned form only when a genuinely separate
    // copy is wanted (e.g. keeping the pre-tick snapshot to diff against).
    // ------------------------------------------------------------------

    /// Insert a new assignment. Logs and does nothing if the variable already
    /// exists, matching [`State::add`].
    pub fn add_mut(&mut self, assignment: SPAssignment, log_target: &str) {
        if self.state.contains_key(&assignment.var.name) {
            log::error!(target: &log_target,
                "Variable {} already in state! Skipped add.", assignment.var.name);
            return;
        }
        self.state
            .insert(assignment.var.name.clone(), assignment);
    }

    /// Remove a variable. Logs and does nothing if it is not present, matching
    /// [`State::remove`].
    pub fn remove_mut(&mut self, var: &str, log_target: &str) {
        if self.state.remove(var).is_none() {
            log::error!(target: &log_target, "Variable '{}' not in state, can't be removed.", var);
        }
    }

    /// Overwrite the value of an existing variable, keeping its `SPVariable`.
    ///
    /// Panics if the variable is not in the state, matching [`State::update`].
    pub fn update_mut(&mut self, name: &str, val: SPValue) {
        match self.state.get_mut(name) {
            Some(assignment) => assignment.val = val,
            None => panic!("Variable {} not in state.", name),
        }
    }

    /// Merge `other` into `self`, matching [`State::extend`]: when
    /// `overwrite_existing` is true the values from `other` win, otherwise the
    /// values already in `self` are kept.
    pub fn extend_mut(&mut self, other: State, overwrite_existing: bool) {
        if overwrite_existing {
            for (key, assignment) in other.state {
                self.state.insert(key, assignment);
            }
        } else {
            for (key, assignment) in other.state {
                self.state.entry(key).or_insert(assignment);
            }
        }
    }

    pub fn add(&self, assignment: SPAssignment, log_target: &str) -> State {
        let mut new_state = self.clone();
        new_state.add_mut(assignment, log_target);
        new_state
    }

    pub fn remove(&self, var: &str, log_target: &str) -> State {
        let mut new_state = self.clone();
        new_state.remove_mut(var, log_target);
        new_state
    }

    // Panics if the variable is not in the state. Should remain panicking.
    //
    // DONE: PERF (highest-impact one-line fix in the crate): `self.state.clone()`
    // deep-copies the *entire* state map just to read one variable, and then
    // throws the copy away. This is the hot path - `SPWrapped::evaluate` calls
    // it for every variable in every predicate, so evaluating one operation
    // guard with 5 variables copies the whole system state 5 times, and the
    // auto/SOP runners do that for every operation on every 50-200 ms tick.
    // With a few hundred state variables this alone can account for most of the
    // CPU you see while a SOP runs. Fix: `self.state.get(name)` - the clone
    // serves no purpose, the borrow is immediate and the value is cloned out
    // anyway. Same bug in `get_assignment`, `contains` and `add` below.

    // PERF: returning `Option<SPValue>` forces a clone of the value on every
    // read. Consider an additional `get_value_ref(&self, name) -> Option<&SPValue>`
    // and have predicate evaluation compare references, so scalar comparisons
    // (`==`, `<`) never allocate. `SPWrapped::evaluate` returning `Cow<SPValue>`
    // would let the common "variable vs literal" comparison stay allocation-free.

    // PERF/correctness: the `log::error!` + `panic!` pair formats the message
    // twice; also note this function can never return `None` (it panics first),
    // so every `unwrap_or_else(|| panic!(..))` at the call sites is dead code.
    pub fn get_value(&self, name: &str, log_target: &str) -> Option<SPValue> {
        match self.state.get(name) {
            None => {
                log::error!(target: &log_target, "Variable {} not in state!", name);
                panic!("Variable {} not in state!", name)
            }
            Some(x) => Some(x.val.clone()),
        }
    }

    // PERF (applies to this and every `get_*_or_*` accessor below): each of
    // these funnels through `get_value`, which currently clones the whole state
    // map (see the note there) and then clones the value out of it, only for
    // the caller to immediately match one variant out of it. Once `get_value`
    // stops cloning the map, the remaining cost is the value clone - for
    // `get_string_or_*`, `get_array_or_*`, `get_map_or_*` and
    // `get_transform_or_*` that is a full heap copy of a `String`/`Vec`/transform
    // on every read, every tick. Suggested: add borrowing variants that match on
    // `&SPValue` and only clone when the caller actually keeps the value; the
    // runners mostly compare the result against a constant (e.g.
    // `request_state == ActionRequestState::Executing.to_string()`) and could
    // compare `&str` against `&str` with no allocation at all.
    pub fn get_bool_or_unknown(&self, name: &str, log_target: &str) -> BoolOrUnknown {
        match self.get_value(name, &log_target) {
            Some(value) => match value {
                SPValue::Bool(b) => b,
                _ => {
                    log::error!(target: &log_target, "Couldn't get boolean '{}' from the state, resulting to UNKNOWN.", name);
                    BoolOrUnknown::UNKNOWN
                }
            },
            None => BoolOrUnknown::UNKNOWN,
        }
    }

    pub fn get_bool_or_default_to_false(&self, name: &str, log_target: &str) -> bool {
        match self.get_bool_or_unknown(name, &log_target) {
            BoolOrUnknown::Bool(b) => b,
            _ => false,
        }
    }

    pub fn get_bool_or_value(&self, name: &str, value: bool, log_target: &str) -> bool {
        match self.get_bool_or_unknown(name, &log_target) {
            BoolOrUnknown::Bool(b) => b,
            _ => value,
        }
    }

    pub fn get_int_or_unknown(&self, name: &str, log_target: &str) -> IntOrUnknown {
        match self.get_value(name, &log_target) {
            Some(value) => match value {
                SPValue::Int64(i) => i,
                _ => {
                    log::error!(target: &log_target, "Couldn't get int '{}' from the state, resulting to UNKNOWN.", name);
                    IntOrUnknown::UNKNOWN
                }
            },
            None => IntOrUnknown::UNKNOWN,
        }
    }

    pub fn get_int_or_default_to_zero(&self, name: &str, log_target: &str) -> i64 {
        match self.get_int_or_unknown(name, &log_target) {
            IntOrUnknown::Int64(i) => i,
            _ => 0,
        }
    }

    pub fn get_int_or_value(&self, name: &str, value: i64, log_target: &str) -> i64 {
        match self.get_int_or_unknown(name, &log_target) {
            IntOrUnknown::Int64(i) => i,
            _ => value,
        }
    }

    pub fn get_float_or_unknown(&self, name: &str, log_target: &str) -> FloatOrUnknown {
        match self.get_value(name, &log_target) {
            Some(value) => match value {
                SPValue::Float64(f) => f,
                _ => {
                    log::error!(target: &log_target, "Couldn't get float '{}' from the state, resulting to UNKNOWN.", name);
                    FloatOrUnknown::UNKNOWN
                }
            },
            None => FloatOrUnknown::UNKNOWN,
        }
    }

    pub fn get_transform_or_unknown(&self, name: &str, log_target: &str) -> TransformOrUnknown {
        match self.get_value(name, &log_target) {
            Some(value) => match value {
                SPValue::Transform(f) => f,
                _ => {
                    log::error!(target: &log_target, "Couldn't get transform '{}' from the state, resulting to UNKNOWN.", name);
                    TransformOrUnknown::UNKNOWN
                }
            },
            None => TransformOrUnknown::UNKNOWN,
        }
    }

    pub fn get_transform_or_default_to_default(
        &self,
        name: &str,
        log_target: &str,
    ) -> SPTransformStamped {
        match self.get_transform_or_unknown(name, &log_target) {
            TransformOrUnknown::Transform(t) => t,
            _ => SPTransformStamped {
                active_transform: false,
                enable_transform: false,
                time_stamp: SystemTime::now(),
                parent_frame_id: "world".to_string(),
                child_frame_id: "failed_lookup".to_string(),
                transform: SPTransform::default(),
                metadata: MapOrUnknown::UNKNOWN,
            },
        }
    }

    pub fn get_float_or_default_to_zero(&self, name: &str, log_target: &str) -> f64 {
        match self.get_float_or_unknown(name, &log_target) {
            FloatOrUnknown::Float64(f) => f.into_inner(),
            _ => 0.0,
        }
    }

    pub fn get_float_or_value(&self, name: &str, value: f64, log_target: &str) -> f64 {
        match self.get_float_or_unknown(name, &log_target) {
            FloatOrUnknown::Float64(f) => f.into_inner(),
            _ => value,
        }
    }

    pub fn get_string_or_unknown(&self, name: &str, log_target: &str) -> StringOrUnknown {
        match self.get_value(name, &log_target) {
            Some(value) => match value {
                SPValue::String(s) => s,
                _ => {
                    log::error!(target: &log_target, "Couldn't get string '{}' from the state, resulting to UNKNOWN.", name);
                    StringOrUnknown::UNKNOWN
                }
            },
            None => StringOrUnknown::UNKNOWN,
        }
    }

    pub fn get_string_or_default_to_unknown(&self, name: &str, log_target: &str) -> String {
        match self.get_string_or_unknown(name, &log_target) {
            StringOrUnknown::String(s) => s,
            _ => SPValue::String(StringOrUnknown::UNKNOWN).to_string(),
        }
    }

    pub fn get_string_or_value(&self, name: &str, value: String, log_target: &str) -> String {
        match self.get_string_or_unknown(name, &log_target) {
            StringOrUnknown::String(s) => s,
            _ => value,
        }
    }

    pub fn get_array_or_unknown(&self, name: &str, log_target: &str) -> ArrayOrUnknown {
        match self.get_value(name, &log_target) {
            Some(value) => match value {
                SPValue::Array(a) => a,
                _ => {
                    log::error!(target: &log_target, "Couldn't get array '{}' from the state, resulting to UNKNOWN.", name);
                    ArrayOrUnknown::UNKNOWN
                }
            },
            None => ArrayOrUnknown::UNKNOWN,
        }
    }

    pub fn get_array_or_default_to_empty(&self, name: &str, log_target: &str) -> Vec<SPValue> {
        match self.get_array_or_unknown(name, &log_target) {
            ArrayOrUnknown::Array(a) => a,
            _ => {
                vec![]
            }
        }
    }

    pub fn get_array_or_value(
        &self,
        name: &str,
        value: Vec<SPValue>,
        log_target: &str,
    ) -> Vec<SPValue> {
        match self.get_array_or_unknown(name, &log_target) {
            ArrayOrUnknown::Array(a) => a,
            _ => value,
        }
    }

    pub fn get_map_or_unknown(&self, name: &str, log_target: &str) -> MapOrUnknown {
        match self.get_value(name, &log_target) {
            Some(value) => match value {
                SPValue::Map(m) => m,
                _ => {
                    log::error!(target: &log_target, "Couldn't get map '{}' from the state, resulting to UNKNOWN.", name);
                    MapOrUnknown::UNKNOWN
                }
            },
            None => MapOrUnknown::UNKNOWN,
        }
    }

    pub fn get_map_or_default_to_empty(
        &self,
        name: &str,
        log_target: &str,
    ) -> Vec<(SPValue, SPValue)> {
        match self.get_map_or_unknown(name, &log_target) {
            MapOrUnknown::Map(m) => m,
            _ => {
                vec![]
            }
        }
    }

    pub fn get_map_or_value(
        &self,
        name: &str,
        value: Vec<(SPValue, SPValue)>,
        log_target: &str,
    ) -> Vec<(SPValue, SPValue)> {
        match self.get_map_or_unknown(name, &log_target) {
            MapOrUnknown::Map(m) => m,
            _ => value,
        }
    }

    pub fn get_time_or_unknown(&self, name: &str, log_target: &str) -> TimeOrUnknown {
        match self.get_value(name, &log_target) {
            Some(value) => match value {
                SPValue::Time(t) => t,
                _ => {
                    log::error!(target: &log_target, "Couldn't get time '{}' from the state, resulting to UNKNOWN.", name);
                    TimeOrUnknown::UNKNOWN
                }
            },
            None => TimeOrUnknown::UNKNOWN,
        }
    }

    // DONE: PERF: same `self.state.clone()` problem as `get_value` - clones the whole
    // map to fetch one entry. Use `self.state.get(name)`. `Operation::start` /
    // `complete` / `fail` / `timeout` / `bypass` / `retry` / `terminate` each
    // call this once per tick per operation, so it is on the hot path too.

    // PERF: returning an owned `SPAssignment` clones the `SPVariable` (two
    // `String`s) even though every caller only needs it to build an `Action`
    // that targets `assignment.var`. Returning `Option<&SPAssignment>` and
    // letting `Action` borrow (or hold an `Arc<SPVariable>`) removes that.
    pub fn get_assignment(&self, name: &str, log_target: &str) -> SPAssignment {
        match self.state.get(name) {
            None => {
                log::error!(target: &log_target, "Variable {} not in state!", name);
                panic!("Variable {} not in state!", name)
            }
            Some(x) => x.clone(),
        }
    }

    pub fn get_all_vars(&self) -> Vec<SPVariable> {
        self.state
            .iter()
            .map(|(_, assignment)| assignment.var.clone())
            .collect()
    }

    // DONE: PERF: clones the entire map to answer a boolean. Use
    // `self.state.contains_key(name)`.
    pub fn contains(&self, name: &str) -> bool {
        self.state.contains_key(name)
    }

    // DONE: one full-map clone per single-variable write, and it used to clone
    // the target `SPVariable` and the incoming value on top of that. It now
    // clones the map once and mutates the value in place via `update_mut`.
    // Chained `.update(..).update(..)` still costs one clone per link, so
    // prefer `update_mut` on a `&mut State` in loops and runner ticks.
    pub fn update(&self, name: &str, val: SPValue) -> State {
        let mut new_state = self.clone();
        new_state.update_mut(name, val);
        new_state
    }

    // DONE: used to build a third map and clone every key and value of *both*
    // inputs - two full copies of `self` plus a full copy of `other`, even
    // though `other` is taken by value and can simply be consumed. It now
    // clones `self` once and moves `other`'s entries in. Behaviour is
    // unchanged: `overwrite_existing` still decides which side wins.
    pub fn extend(&self, other: State, overwrite_existing: bool) -> State {
        let mut new_state = self.clone();
        new_state.extend_mut(other, overwrite_existing);
        new_state
    }

    // PERF: re-parses the goal predicate from its string form on every call.
    // `planner_ticker` calls it on each replan request, and the parse walks a
    // PEG grammar and allocates the whole `Predicate` tree. Suggested: cache the
    // parsed predicate keyed by the goal string (the goal only changes when the
    // goal runner swaps it, and `*_current_goal_id` already tells you when that
    // happened), so a replan for the same goal reuses the parsed tree.
    pub fn extract_goal(&self, name: &str) -> Predicate {
        match self.state.get(&format!("{}_current_goal_predicate", name)) {
            Some(g_spvalue) => match &g_spvalue.val {
                SPValue::String(StringOrUnknown::String(g_value)) => {
                    match pred_parser::pred(&g_value, &self) {
                        Ok(goal_predicate) => goal_predicate,
                        Err(_) => Predicate::TRUE,
                    }
                }
                _ => Predicate::TRUE,
            },
            None => Predicate::TRUE,
        }
    }
}

impl fmt::Display for State {
    fn fmt(&self, fmtr: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s: String = {
            let mut children: Vec<_> = self
                .state
                .iter()
                .map(|(k, v)| match &v.val {
                    SPValue::Array(arr) => match arr {
                        ArrayOrUnknown::UNKNOWN => format!("    {}: {}", k, v.val),
                        ArrayOrUnknown::Array(some_array) => {
                            let mut sub_children: Vec<String> = vec![format!("    {}:", k)];
                            sub_children.extend(
                                some_array
                                    .iter()
                                    .map(|value| format!("        {}", value))
                                    .collect::<Vec<String>>(),
                            );
                            format!("{}", sub_children.join("\n"))
                        }
                    },
                    _ => format!("    {}: {}", k, v.val),
                })
                .collect();
            children.sort();
            format!("{}", children.join("\n"))
        };

        write!(fmtr, "State: {{\n{}\n}}\n", &s)
    }
}

#[cfg(test)]
mod tests {
    use crate::*;
    use std::time::SystemTime;

    fn create_dummy_transform() -> SPTransformStamped {
        SPTransformStamped {
            active_transform: true,
            enable_transform: true,
            time_stamp: SystemTime::now(),
            parent_frame_id: "world".to_string(),
            child_frame_id: "robot".to_string(),
            transform: SPTransform::default(),
            metadata: MapOrUnknown::Map(vec![("quality".to_spvalue(), "good".to_spvalue())]),
        }
    }

    fn get_initial_state() -> State {
        let name = SPVariable::new("name", SPValueType::String);
        let height = SPVariable::new("height", SPValueType::Int64);
        let smart = SPVariable::new("smart", SPValueType::Bool);
        let weight = SPVariable::new("weight", SPValueType::Float64);
        let items = SPVariable::new("items", SPValueType::Array);
        let data = SPVariable::new("data", SPValueType::Map);
        let pose = SPVariable::new("pose", SPValueType::Transform);
        let time = SPVariable::new("time", SPValueType::Time);

        State::from_vec(&vec![
            (name, "John".to_spvalue()),
            (height, 185.to_spvalue()),
            (smart, true.to_spvalue()),
            (weight, 80.0.to_spvalue()),
            (items, vec![1.to_spvalue()].to_spvalue()),
            (
                data,
                vec![("a".to_spvalue(), "b".to_spvalue())].to_spvalue(),
            ),
            (pose, create_dummy_transform().to_spvalue()),
            (time, SystemTime::now().to_spvalue()),
        ])
    }

    #[test]
    fn test_state_new_and_from_vec() {
        let new_state = State::new();
        assert!(new_state.state.is_empty());

        let initial_state = get_initial_state();
        assert_eq!(initial_state.state.len(), 8);
    }

    #[test]
    fn test_get_value_and_assignment() {
        let state = get_initial_state();
        assert_eq!(state.get_value("height", "t"), Some(185.to_spvalue()));
        let assignment = state.get_assignment("height", "t");
        assert_eq!(assignment.var.name, "height");
        assert_eq!(assignment.val, 185.to_spvalue());
    }

    #[test]
    #[should_panic]
    fn test_get_value_panic() {
        let state = State::new();
        state.get_value("nonexistent", "t").unwrap();
    }

    #[test]
    #[should_panic]
    fn test_get_assignment_panic() {
        let state = State::new();
        state.get_assignment("nonexistent", "t");
    }

    #[test]
    fn test_add() {
        let state = State::new();
        let var = SPVariable::new("v", SPValueType::Bool);
        let assignment = SPAssignment::new(var, true.to_spvalue());
        let new_state = state.add(assignment.clone(), "test");
        assert_eq!(new_state.state.len(), 1);
        let same_state = new_state.add(assignment, "test");
        assert_eq!(same_state.state.len(), 1);
    }

    #[test]
    fn test_update() {
        let state = get_initial_state();
        let updated_state = state.update("height", 190.to_spvalue());
        assert_eq!(
            updated_state.get_value("height", "t"),
            Some(190.to_spvalue())
        );
    }

    #[test]
    #[should_panic]
    fn test_update_panic() {
        let state = State::new();
        state.update("nonexistent", 1.to_spvalue());
    }

    #[test]
    fn test_contains_and_get_all_vars() {
        let state = get_initial_state();
        assert!(state.contains("name"));
        assert!(!state.contains("age"));
        let vars = state.get_all_vars();
        assert_eq!(vars.len(), 8);
        assert!(vars.contains(&SPVariable::new("name", SPValueType::String)));
    }

    #[test]
    fn test_get_diff_values() {
        let var_a = SPVariable::new("a", SPValueType::Int64);
        let var_b = SPVariable::new("b", SPValueType::Bool);
        let var_c = SPVariable::new("c", SPValueType::String);

        let state1 = State::from_vec(&vec![
            (var_a.clone(), 1.to_spvalue()),
            (var_b.clone(), true.to_spvalue()),
            (var_c.clone(), "hello".to_spvalue()),
        ]);

        let var_d = SPVariable::new("d", SPValueType::Float64);
        let state2 = State::from_vec(&vec![
            (var_a.clone(), 2.to_spvalue()),    // Changed value
            (var_b.clone(), true.to_spvalue()), // Same value
            (var_d.clone(), 3.14.to_spvalue()), // Not in state1
        ]);

        let changed = state1.get_diff_values(&state2);

        assert_eq!(changed.len(), 1);
        assert!(changed.contains_key(&var_a));

        let (old_val, new_val) = changed.get(&var_a).unwrap();
        assert_eq!(*old_val, 1.to_spvalue());
        assert_eq!(*new_val, 2.to_spvalue());

        let no_changes = state1.get_diff_values(&state1);
        assert!(no_changes.is_empty());
    }

    #[test]
    fn test_get_diff_variables() {
        let var_a = SPVariable::new("a", SPValueType::Int64);
        let var_b = SPVariable::new("b", SPValueType::Bool);
        let var_c = SPVariable::new("c", SPValueType::String);

        let state1 = State::from_vec(&vec![
            (var_a.clone(), 1.to_spvalue()),
            (var_b.clone(), true.to_spvalue()),
        ]);

        let state2 = State::from_vec(&vec![
            (var_b.clone(), false.to_spvalue()),
            (var_c.clone(), "hello".to_spvalue()),
        ]);

        let mut uncommon = state1.get_diff_variables(&state2);
        uncommon.sort(); // Sort for consistent test results

        let mut expected = vec![var_a.clone(), var_c];
        expected.sort();

        assert_eq!(uncommon, expected);

        // Test with no uncommon variables
        let no_uncommon = state1.get_diff_variables(&state1);
        assert!(no_uncommon.is_empty());

        // Test with an empty state
        let empty_state = State::new();
        let mut uncommon_with_empty = state1.get_diff_variables(&empty_state);
        uncommon_with_empty.sort();

        let mut expected_with_empty = vec![var_a.clone(), var_b.clone()];
        expected_with_empty.sort();

        assert_eq!(uncommon_with_empty, expected_with_empty);
    }

    #[test]
    fn test_get_diff_partial_state() {
        let var_a = SPVariable::new("a", SPValueType::Int64);
        let var_b = SPVariable::new("b", SPValueType::Bool);

        let state1 = State::from_vec(&vec![
            (var_a.clone(), 1.to_spvalue()),
            (var_b.clone(), true.to_spvalue()),
        ]);

        let var_c = SPVariable::new("c", SPValueType::String);
        let state2 = State::from_vec(&vec![
            (var_a.clone(), 2.to_spvalue()),     // Updated
            (var_b.clone(), true.to_spvalue()),  // Unchanged
            (var_c.clone(), "new".to_spvalue()), // New, should be ignored
        ]);

        let updated_state = state1.get_diff_partial_state(&state2);

        assert_eq!(
            updated_state.state.len(),
            1,
            "Only the updated variable should be in the new state"
        );
        assert!(
            updated_state.contains("a"),
            "The updated variable 'a' should be present"
        );
        assert_eq!(
            updated_state.get_value("a", "t"),
            Some(2.to_spvalue()),
            "The value of 'a' should be the new value"
        );
    }

    #[test]
    fn test_extend() {
        let state1 = State::from_vec(&vec![(
            SPVariable::new("a", SPValueType::Int64),
            1.to_spvalue(),
        )]);
        let state2 = State::from_vec(&vec![
            (SPVariable::new("a", SPValueType::Int64), 2.to_spvalue()),
            (SPVariable::new("b", SPValueType::Int64), 3.to_spvalue()),
        ]);

        let extended_overwrite = state1.extend(state2.clone(), true);
        assert_eq!(extended_overwrite.state.len(), 2);
        assert_eq!(extended_overwrite.get_value("a", "t"), Some(2.to_spvalue()));

        let extended_no_overwrite = state1.extend(state2.clone(), false);
        assert_eq!(extended_no_overwrite.state.len(), 2);
        assert_eq!(
            extended_no_overwrite.get_value("a", "t"),
            Some(1.to_spvalue())
        );
    }

    #[test]
    fn test_getters() {
        let state = get_initial_state();
        let wrong_type_state = State::from_vec(&vec![(
            SPVariable::new("smart", SPValueType::Int64),
            0.to_spvalue(),
        )]);

        assert_eq!(
            state.get_bool_or_unknown("smart", "t"),
            BoolOrUnknown::Bool(true)
        );
        assert_eq!(
            wrong_type_state.get_bool_or_unknown("smart", "t"),
            BoolOrUnknown::UNKNOWN
        );
        assert!(state.get_bool_or_default_to_false("smart", "t"));
        assert!(!wrong_type_state.get_bool_or_default_to_false("smart", "t"));
        assert!(state.get_bool_or_value("smart", false, "t"));
        assert!(!wrong_type_state.get_bool_or_value("smart", false, "t"));

        assert_eq!(
            state.get_int_or_unknown("height", "t"),
            IntOrUnknown::Int64(185)
        );
        assert_eq!(state.get_int_or_default_to_zero("height", "t"), 185);
        assert_eq!(state.get_int_or_value("height", 0, "t"), 185);

        assert_eq!(
            state.get_float_or_unknown("weight", "t"),
            FloatOrUnknown::Float64(80.0.into())
        );
        assert_eq!(state.get_float_or_default_to_zero("weight", "t"), 80.0);
        assert_eq!(state.get_float_or_value("weight", 0.0, "t"), 80.0);

        assert_eq!(
            state.get_string_or_unknown("name", "t"),
            StringOrUnknown::String("John".to_string())
        );
        assert_eq!(
            state.get_string_or_default_to_unknown("name", "t"),
            "John".to_string()
        );
        assert_eq!(
            state.get_string_or_value("name", "".to_string(), "t"),
            "John".to_string()
        );

        assert_eq!(
            state.get_array_or_unknown("items", "t"),
            ArrayOrUnknown::Array(vec![1.to_spvalue()])
        );
        assert_eq!(
            state.get_array_or_default_to_empty("items", "t"),
            vec![1.to_spvalue()]
        );
        assert_eq!(
            state.get_array_or_value("items", vec![], "t"),
            vec![1.to_spvalue()]
        );

        assert_eq!(
            state.get_map_or_unknown("data", "t"),
            MapOrUnknown::Map(vec![("a".to_spvalue(), "b".to_spvalue())])
        );
        assert_eq!(
            state.get_map_or_default_to_empty("data", "t"),
            vec![("a".to_spvalue(), "b".to_spvalue())]
        );
        assert_eq!(
            state.get_map_or_value("data", vec![], "t"),
            vec![("a".to_spvalue(), "b".to_spvalue())]
        );

        assert!(matches!(
            state.get_time_or_unknown("time", "t"),
            TimeOrUnknown::Time(_)
        ));

        assert!(matches!(
            state.get_transform_or_unknown("pose", "t"),
            TransformOrUnknown::Transform(_)
        ));
        let default_tf = state.get_transform_or_default_to_default("pose", "t");
        assert_eq!(default_tf.parent_frame_id, "world");
    }

    #[test]
    fn test_getters_defaults() {
        let state = State::new();
        let result = std::panic::catch_unwind(|| {
            state.get_string_or_default_to_unknown("x", "t");
        });
        assert!(result.is_err(), "Was expected to panic, but it did not.");
        let result = std::panic::catch_unwind(|| {
            state.get_array_or_default_to_empty("x", "t");
        });
        assert!(result.is_err(), "Was expected to panic, but it did not.");
        let result = std::panic::catch_unwind(|| {
            state.get_map_or_default_to_empty("x", "t");
        });
        assert!(result.is_err(), "Was expected to panic, but it did not.");
        let result = std::panic::catch_unwind(|| {
            state.get_transform_or_default_to_default("x", "t");
        });
        assert!(result.is_err(), "Was expected to panic, but it did not.");
    }

    #[test]
    fn test_display() {
        let state = get_initial_state();
        let display_str = format!("{}", state);
        assert!(display_str.starts_with("State: {\n"));
        assert!(display_str.contains("    name: John\n"));
        assert!(display_str.contains("    height: 185\n"));
        assert!(display_str.ends_with("}\n"));

        let arr_state = State::from_vec(&vec![(
            SPVariable::new("arr", SPValueType::Array),
            vec![1.to_spvalue(), 2.to_spvalue()].to_spvalue(),
        )]);
        let arr_display = format!("{}", arr_state);
        assert!(arr_display.contains("    arr:\n        1\n        2"));

        let unk_arr_state = State::from_vec(&vec![(
            SPVariable::new("unk", SPValueType::Array),
            SPValue::Array(ArrayOrUnknown::UNKNOWN),
        )]);
        assert!(format!("{}", unk_arr_state).contains("    unk: UNKNOWN"));
    }

    #[test]
    fn test_extract_goal() {
        let state_no_goal = get_initial_state();
        assert_eq!(state_no_goal.extract_goal("g"), Predicate::TRUE);

        let state_with_bad_goal = state_no_goal.add(SPAssignment::new(
            SPVariable::new("g_current_goal_predicate", SPValueType::Int64),
            1.to_spvalue(),
        ), "test");
        assert_eq!(state_with_bad_goal.extract_goal("g"), Predicate::TRUE);
    }
}
/// The typed accessors, exhaustively.
///
/// Every runner reads its state through these, and each one has three
/// behaviours that only differ in the failure cases: the value it returns for a
/// well-typed variable, the sentinel it returns for a variable of the *wrong*
/// type, and - the sharp one - a panic for a variable that is not in the state
/// at all. The existing tests cover a couple of them; this covers the table.
///
/// The wrong-type behaviour is what a consuming package hits when it writes a
/// key by hand from another process, so "returns the default instead of
/// exploding" is a contract, not an accident.
#[cfg(test)]
mod accessor_tests {
    use crate::*;
    use std::time::SystemTime;

    const TARGET: &str = "test";

    fn transform() -> SPTransformStamped {
        SPTransformStamped {
            active_transform: true,
            enable_transform: true,
            time_stamp: SystemTime::now(),
            parent_frame_id: "world".to_string(),
            child_frame_id: "robot".to_string(),
            transform: SPTransform::default(),
            metadata: MapOrUnknown::Map(vec![]),
        }
    }

    /// One variable of every type, plus one of each type holding the UNKNOWN
    /// variant, plus a `wrong` variable that is a string whatever you ask for.
    fn state() -> State {
        State::from_vec(&vec![
            (SPVariable::new("b", SPValueType::Bool), true.to_spvalue()),
            (SPVariable::new("i", SPValueType::Int64), 7.to_spvalue()),
            (SPVariable::new("f", SPValueType::Float64), 2.5.to_spvalue()),
            (SPVariable::new("s", SPValueType::String), "text".to_spvalue()),
            (
                SPVariable::new("arr", SPValueType::Array),
                vec![1.to_spvalue()].to_spvalue(),
            ),
            (
                SPVariable::new("map", SPValueType::Map),
                vec![("k".to_spvalue(), "v".to_spvalue())].to_spvalue(),
            ),
            (
                SPVariable::new("tf", SPValueType::Transform),
                transform().to_spvalue(),
            ),
            (
                SPVariable::new("t", SPValueType::Time),
                SystemTime::now().to_spvalue(),
            ),
            (
                SPVariable::new("wrong", SPValueType::String),
                "not what you asked for".to_spvalue(),
            ),
            (
                SPVariable::new("b_unknown", SPValueType::Bool),
                SPValue::Bool(BoolOrUnknown::UNKNOWN),
            ),
            (
                SPVariable::new("i_unknown", SPValueType::Int64),
                SPValue::Int64(IntOrUnknown::UNKNOWN),
            ),
            (
                SPVariable::new("f_unknown", SPValueType::Float64),
                SPValue::Float64(FloatOrUnknown::UNKNOWN),
            ),
            (
                SPVariable::new("s_unknown", SPValueType::String),
                SPValue::String(StringOrUnknown::UNKNOWN),
            ),
            (
                SPVariable::new("arr_unknown", SPValueType::Array),
                SPValue::Array(ArrayOrUnknown::UNKNOWN),
            ),
            (
                SPVariable::new("map_unknown", SPValueType::Map),
                SPValue::Map(MapOrUnknown::UNKNOWN),
            ),
            (
                SPVariable::new("tf_unknown", SPValueType::Transform),
                SPValue::Transform(TransformOrUnknown::UNKNOWN),
            ),
            (
                SPVariable::new("t_unknown", SPValueType::Time),
                SPValue::Time(TimeOrUnknown::UNKNOWN),
            ),
        ])
    }

    #[test]
    fn a_well_typed_variable_reads_back_as_itself() {
        let state = state();
        assert!(state.get_bool_or_default_to_false("b", TARGET));
        assert_eq!(state.get_int_or_default_to_zero("i", TARGET), 7);
        assert_eq!(state.get_float_or_default_to_zero("f", TARGET), 2.5);
        assert_eq!(state.get_string_or_default_to_unknown("s", TARGET), "text");
        assert_eq!(state.get_array_or_default_to_empty("arr", TARGET).len(), 1);
        assert_eq!(state.get_map_or_default_to_empty("map", TARGET).len(), 1);
        assert_eq!(
            state
                .get_transform_or_default_to_default("tf", TARGET)
                .child_frame_id,
            "robot"
        );
        assert!(matches!(
            state.get_time_or_unknown("t", TARGET),
            TimeOrUnknown::Time(_)
        ));
    }

    /// Asking for the wrong type gives the type's sentinel, not a panic and not
    /// a coerced value.
    #[test]
    fn the_wrong_type_falls_back_to_the_sentinel() {
        let state = state();
        assert!(!state.get_bool_or_default_to_false("wrong", TARGET));
        assert_eq!(state.get_int_or_default_to_zero("wrong", TARGET), 0);
        assert_eq!(state.get_float_or_default_to_zero("wrong", TARGET), 0.0);
        assert!(state.get_array_or_default_to_empty("wrong", TARGET).is_empty());
        assert!(state.get_map_or_default_to_empty("wrong", TARGET).is_empty());
        assert!(matches!(
            state.get_time_or_unknown("wrong", TARGET),
            TimeOrUnknown::UNKNOWN
        ));
        assert!(matches!(
            state.get_bool_or_unknown("wrong", TARGET),
            BoolOrUnknown::UNKNOWN
        ));
        assert!(matches!(
            state.get_int_or_unknown("wrong", TARGET),
            IntOrUnknown::UNKNOWN
        ));
        assert!(matches!(
            state.get_float_or_unknown("wrong", TARGET),
            FloatOrUnknown::UNKNOWN
        ));
        assert!(matches!(
            state.get_array_or_unknown("wrong", TARGET),
            ArrayOrUnknown::UNKNOWN
        ));
        assert!(matches!(
            state.get_map_or_unknown("wrong", TARGET),
            MapOrUnknown::UNKNOWN
        ));
        assert!(matches!(
            state.get_transform_or_unknown("wrong", TARGET),
            TransformOrUnknown::UNKNOWN
        ));

        // The transform sentinel is a real, recognisable value rather than a
        // zeroed struct - `failed_lookup` is what shows up in a log when a
        // lookup did not resolve.
        let fallback = state.get_transform_or_default_to_default("wrong", TARGET);
        assert_eq!(fallback.child_frame_id, "failed_lookup");
        assert!(!fallback.active_transform);
    }

    /// A variable explicitly holding UNKNOWN behaves the same as one of the
    /// wrong type - which is what lets a model declare "not known yet".
    #[test]
    fn an_explicit_unknown_reads_as_the_default() {
        let state = state();
        assert!(!state.get_bool_or_default_to_false("b_unknown", TARGET));
        assert_eq!(state.get_int_or_default_to_zero("i_unknown", TARGET), 0);
        assert_eq!(state.get_float_or_default_to_zero("f_unknown", TARGET), 0.0);
        assert_eq!(
            state.get_string_or_default_to_unknown("s_unknown", TARGET),
            "UNKNOWN"
        );
        assert!(state.get_array_or_default_to_empty("arr_unknown", TARGET).is_empty());
        assert!(state.get_map_or_default_to_empty("map_unknown", TARGET).is_empty());
        assert_eq!(
            state
                .get_transform_or_default_to_default("tf_unknown", TARGET)
                .child_frame_id,
            "failed_lookup"
        );
    }

    /// The `_or_value` variants let the caller supply the fallback instead of
    /// taking the type's own.
    #[test]
    fn the_or_value_accessors_use_the_callers_fallback() {
        let state = state();
        assert!(state.get_bool_or_value("wrong", true, TARGET));
        assert_eq!(state.get_int_or_value("wrong", 42, TARGET), 42);
        assert_eq!(state.get_float_or_value("wrong", 1.5, TARGET), 1.5);
        assert_eq!(
            state.get_string_or_value("wrong", "fallback".to_string(), TARGET),
            "not what you asked for",
            "a well-typed value still wins over the fallback"
        );
        assert_eq!(
            state.get_string_or_value("s_unknown", "fallback".to_string(), TARGET),
            "fallback"
        );
        assert_eq!(
            state.get_array_or_value("wrong", vec![9.to_spvalue()], TARGET),
            vec![9.to_spvalue()]
        );
        assert_eq!(
            state
                .get_map_or_value("wrong", vec![("x".to_spvalue(), "y".to_spvalue())], TARGET)
                .len(),
            1
        );

        // And the real values are returned unchanged when they are the right type.
        assert!(state.get_bool_or_value("b", false, TARGET));
        assert_eq!(state.get_int_or_value("i", 0, TARGET), 7);
        assert_eq!(state.get_float_or_value("f", 0.0, TARGET), 2.5);
    }

    /// The hazard every runner inherits: a key that is absent is not a default,
    /// it is a panic. See the note on `time_runner`'s
    /// `a_timer_that_was_never_initialised_kills_the_runner` for what that
    /// costs in practice.
    #[test]
    fn every_accessor_panics_on_a_missing_key() {
        let state = state();

        macro_rules! assert_panics {
            ($body:expr) => {
                let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| $body));
                assert!(result.is_err(), "expected a panic on a missing key");
            };
        }

        // Keep the panic output quiet - these are all expected.
        let previous = std::panic::take_hook();
        std::panic::set_hook(Box::new(|_| {}));

        assert_panics!(state.get_bool_or_default_to_false("nope", TARGET));
        assert_panics!(state.get_int_or_default_to_zero("nope", TARGET));
        assert_panics!(state.get_float_or_default_to_zero("nope", TARGET));
        assert_panics!(state.get_string_or_default_to_unknown("nope", TARGET));
        assert_panics!(state.get_array_or_default_to_empty("nope", TARGET));
        assert_panics!(state.get_map_or_default_to_empty("nope", TARGET));
        assert_panics!(state.get_transform_or_default_to_default("nope", TARGET));
        assert_panics!(state.get_time_or_unknown("nope", TARGET));

        std::panic::set_hook(previous);
    }

    /// `remove` / `remove_mut` drop a variable; removing one that is not there
    /// is logged and ignored rather than being an error.
    #[test]
    fn removing_a_variable_drops_it_and_removing_a_missing_one_is_harmless() {
        let mut state = state();
        assert!(state.contains("b"));

        state.remove_mut("b", TARGET);
        assert!(!state.contains("b"));

        // Idempotent.
        state.remove_mut("b", TARGET);
        assert!(!state.contains("b"));
        state.remove_mut("never_existed", TARGET);

        // The owned form leaves the original alone.
        let smaller = state.remove("i", TARGET);
        assert!(state.contains("i"));
        assert!(!smaller.contains("i"));
    }

    /// `extend_mut`'s `overwrite_existing` flag decides which side wins, and it
    /// is the difference between "load defaults" and "apply an update".
    #[test]
    fn extend_mut_respects_the_overwrite_flag() {
        let mut keep = State::from_vec(&vec![(
            SPVariable::new("x", SPValueType::Int64),
            1.to_spvalue(),
        )]);
        let other = State::from_vec(&vec![
            (SPVariable::new("x", SPValueType::Int64), 2.to_spvalue()),
            (SPVariable::new("y", SPValueType::Int64), 3.to_spvalue()),
        ]);

        let mut overwrite = keep.clone();
        overwrite.extend_mut(other.clone(), true);
        assert_eq!(overwrite.get_value("x", TARGET), Some(2.to_spvalue()));
        assert_eq!(overwrite.get_value("y", TARGET), Some(3.to_spvalue()));

        keep.extend_mut(other, false);
        assert_eq!(
            keep.get_value("x", TARGET),
            Some(1.to_spvalue()),
            "without overwrite the existing value must be kept"
        );
        assert_eq!(keep.get_value("y", TARGET), Some(3.to_spvalue()));
    }

    /// `update_mut` is the in-place form of `update` and shares its contract,
    /// including the panic - it updates an existing variable, it does not
    /// create one.
    #[test]
    fn update_mut_replaces_a_value_in_place() {
        let mut state = state();
        state.update_mut("i", 99.to_spvalue());
        assert_eq!(state.get_value("i", TARGET), Some(99.to_spvalue()));
    }

    #[test]
    #[should_panic(expected = "not in state")]
    fn update_mut_panics_on_a_variable_that_does_not_exist() {
        let mut state = state();
        state.update_mut("never_declared", 1.to_spvalue());
    }

    /// `extract_goal` turns the goal string a dashboard wrote into a predicate.
    /// Everything it cannot parse becomes `TRUE` - i.e. "already satisfied" -
    /// which is worth knowing: a typo in a goal makes the planner report that
    /// there is nothing to do rather than that the goal was invalid.
    #[test]
    fn an_unparseable_goal_becomes_true_rather_than_an_error() {
        let mut state = state();
        state.add_mut(
            SPAssignment::new(
                SPVariable::new("sp_current_goal_predicate", SPValueType::String),
                "this is not a predicate".to_spvalue(),
            ),
            TARGET,
        );
        assert_eq!(state.extract_goal("sp"), Predicate::TRUE);

        // A goal of the wrong type, and a missing goal, do the same.
        let mut wrong_type = state.clone();
        wrong_type.update_mut("sp_current_goal_predicate", 7.to_spvalue());
        assert_eq!(wrong_type.extract_goal("sp"), Predicate::TRUE);
        assert_eq!(state.extract_goal("no_such_runner"), Predicate::TRUE);
    }

    #[test]
    fn a_parseable_goal_becomes_the_predicate_it_describes() {
        let mut state = state();
        state.add_mut(
            SPAssignment::new(
                SPVariable::new("sp_current_goal_predicate", SPValueType::String),
                "var:i == 7".to_spvalue(),
            ),
            TARGET,
        );

        let goal = state.extract_goal("sp");
        assert_ne!(goal, Predicate::TRUE);
        assert!(goal.eval(&state, TARGET), "the goal already holds in this state");

        state.update_mut("i", 8.to_spvalue());
        assert!(!goal.eval(&state, TARGET));
    }
}
