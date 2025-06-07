use crate::*;
use serde::{Deserialize, Serialize};
// use crate::{
//     get_predicate_vars_all, Action,
//     Predicate, SPVariable, State,
// };
use std::{fmt, hash::Hash};

/// A planning transition T contains a guard predicate G : S → {false, true},
/// and a set of action functions A, where ∀a ∈ A, a : S → S models
/// the updates of the state variables. If the guard predicate evaluates to
/// true, the transition can occur, after which the actions of the transition
/// describe how the variables are updated. The notation we use to represent
/// a planning transition is T : G/A.
///
/// A running transition Tr extends the planning transition with an additional
/// running guard Gr and additional running action Ar. We write
/// running transitions as Tr : G / Gr / A / Ar , where g and gr are both guard
/// predicates and G ∧ Gr : S → {false, true}, and A and Ar are both action
/// functions, where ∀a ∈ A ∪ Ar , a : S → S model the updates of the values
/// of the state variables. While planning, only G and A are considered, i.e.
/// the running transition is evaluated and taken as a planning transition.
/// When the execution engine is running the plan, it is considering all
/// components of Tr, i.e. the running transition guard becomes G ∧ Gr and the
/// set of transition actions becomes A ∪ Ar.
#[derive(Debug, Clone, Eq, Hash, Serialize, Deserialize)]
pub struct Transition {
    pub name: String,
    pub guard: Predicate,
    pub runner_guard: Predicate,
    pub delay_ms: u64,
    pub actions: Vec<Action>,
    pub runner_actions: Vec<Action>,
}

// impl Hash for Transition {
//     fn hash<H: Hasher>(&self, s: &mut H) {
//         self.guard.hash(s);
//         self.runner_guard.hash(s);
//         self.actions
//         //     .keys()
//         //     .into_iter()
//         //     .map(|x| x.to_owned())
//         //     .collect::<Vec<String>>()
//         //     .hash(s);
//         // self.state
//         //     .values()
//         //     .into_iter()
//         //     .map(|x| x.var.to_owned())
//         //     .collect::<Vec<SPVariable>>()
//         //     .hash(s);
//         // self.state
//         //     .values()
//         //     .into_iter()
//         //     .map(|x| x.val.to_owned())
//         //     .collect::<Vec<SPValue>>()
//         //     .hash(s);
//     }
// }

impl Transition {
    /// Define a new transition. Use parse() instead.
    pub fn new(
        name: &str,
        guard: Predicate,
        runner_guard: Predicate,
        delay_ms: u64,
        actions: Vec<Action>,
        runner_actions: Vec<Action>,
    ) -> Transition {
        Transition {
            name: name.to_string(),
            guard,
            runner_guard,
            delay_ms,
            actions,
            runner_actions,
        }
    }

    /// Define a new transition using strings.
    pub fn parse(
        name: &str,
        guard: &str,
        runner_guard: &str,
        delay_ms: u64,
        actions: Vec<&str>,
        runner_actions: Vec<&str>,
        state: &State,
    ) -> Transition {
        Transition::new(
            name,
            match pred_parser::pred(guard, state) {
                Ok(guard_predicate) => guard_predicate,
                Err(e) => {
                    log::error!(target: &&format!("transition_parser"), 
                        "Failed to parse guard {guard} with: {e}");
                    log::error!(target: &&format!("transition_parser"), 
                        "Guard set to FALSE, fix the model.");
                    Predicate::FALSE
                }
            },
            match pred_parser::pred(runner_guard, state) {
                Ok(guard_predicate) => guard_predicate,
                Err(e) => {
                    log::error!(target: &&format!("transition_parser"), 
                        "Failed to parse guard {runner_guard} with: {e}");
                    log::error!(target: &&format!("transition_parser"), 
                        "Runner guard set to FALSE, fix the model.");
                    Predicate::FALSE
                }
            },
            delay_ms,
            actions
                .iter()
                .map(|action| match pred_parser::action(action, state) {
                    Ok(action_def) => action_def,
                    Err(e) => {
                        log::error!(target: &&format!("transition_parser"), 
                            "Failed to parse action {action} with: {e}");
                        log::error!(target: &&format!("transition_parser"), 
                            "Action set to EMPTY, fix the model.");
                        Action::empty()
                    }
                })
                .collect::<Vec<Action>>(),
            runner_actions
                .iter()
                .map(|action| match pred_parser::action(action, state) {
                    Ok(action_def) => action_def,
                    Err(e) => {
                        log::error!(target: &&format!("transition_parser"), 
                            "Failed to parse runner_actions {action} with: {e}");
                        log::error!(target: &&format!("transition_parser"), 
                            "Runner action set to EMPTY, fix the model.");
                        Action::empty()
                    }
                })
                .collect::<Vec<Action>>(),
        )
    }

    ///
    pub fn empty() -> Transition {
        Transition::new(
            "empty",
            Predicate::FALSE,
            Predicate::FALSE,
            0,
            vec![],
            vec![],
        )
    }

    pub fn eval_planning(self, state: &State) -> bool {
        self.guard.eval(state)
    }

    pub fn eval_running(self, state: &State) -> bool {
        self.guard.eval(state) && self.runner_guard.eval(state)
    }

    pub fn take_planning(self, state: &State) -> State {
        let mut new_state = state.clone();
        for a in self.actions {
            new_state = a.assign(&new_state)
        }
        new_state
    }

    pub fn take_running(self, state: &State) -> State {
        let mut new_state = state.clone();
        for a in self.actions {
            new_state = a.assign(&new_state)
        }
        for a in self.runner_actions {
            new_state = a.assign(&new_state)
        }
        new_state
    }

    pub fn relax(self, vars: &Vec<String>) -> Transition {
        let r_guard = self.guard.remove(vars);
        let r_runner_guard = self.runner_guard.remove(vars);
        let mut r_actions = vec![];
        let mut r_runner_actions = vec![];
        self.actions
            .iter()
            .for_each(|x| match vars.contains(&x.var.name) {
                false => r_actions.push(x.clone()),
                true => (),
            });
        self.runner_actions
            .iter()
            .for_each(|x| match vars.contains(&x.var.name) {
                false => r_runner_actions.push(x.clone()),
                true => (),
            });
        Transition {
            name: self.name,
            guard: match r_guard {
                Some(x) => x,
                None => Predicate::TRUE,
            },
            runner_guard: match r_runner_guard {
                Some(x) => x,
                None => Predicate::TRUE,
            },
            delay_ms: self.delay_ms,
            actions: r_actions,
            runner_actions: r_runner_actions,
        }
    }

    // TODO: test...
    pub fn contains_planning(self, var: &String) -> bool {
        let guard_vars: Vec<String> = get_predicate_vars_all(&self.guard)
            .iter()
            .map(|p| p.name.to_owned())
            .collect();
        let actions_vars: Vec<String> =
            self.actions.iter().map(|a| a.var.name.to_owned()).collect();
        guard_vars.contains(var) || actions_vars.contains(var)
    }
}

impl PartialEq for Transition {
    fn eq(&self, other: &Transition) -> bool {
        self.guard == other.guard
            && self.runner_guard == other.runner_guard
            && self.actions == other.actions
            && self.runner_actions == other.runner_actions
    }
}

impl Default for Transition {
    fn default() -> Self {
        Transition {
            name: "unknown".to_string(),
            guard: Predicate::TRUE,
            runner_guard: Predicate::TRUE,
            delay_ms: 0,
            actions: vec![],
            runner_actions: vec![],
        }
    }
}

// TODO: test
pub fn get_transition_vars_all(trans: &Transition) -> Vec<SPVariable> {
    let mut s = Vec::new();
    let guard_vars = get_predicate_vars_all(&trans.guard);
    let runner_guard_vars = get_predicate_vars_all(&trans.runner_guard);
    let action_vars: Vec<SPVariable> = trans.actions.iter().map(|x| x.var.to_owned()).collect();
    let runner_action_vars: Vec<SPVariable> = trans
        .runner_actions
        .iter()
        .map(|x| x.var.to_owned())
        .collect();
    s.extend(guard_vars);
    s.extend(runner_guard_vars);
    s.extend(action_vars);
    s.extend(runner_action_vars);
    s.sort();
    s.dedup();
    s
}

// TODO: test
pub fn get_transition_model_vars_all(model: &Vec<Transition>) -> Vec<SPVariable> {
    let mut s = Vec::new();
    model
        .iter()
        .for_each(|x| s.extend(get_transition_vars_all(x)));
    s.sort();
    s.dedup();
    s
}

impl fmt::Display for Transition {
    fn fmt(&self, fmtr: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut action_string = "".to_string();
        let mut actions = self.actions.clone();
        match actions.pop() {
            Some(last_action) => {
                action_string = actions
                    .iter()
                    .map(|x| format!("{}, ", x.to_string()))
                    .collect::<String>();
                let last_action_string = &format!("{}", last_action.to_string());
                action_string.extend(last_action_string.chars());
            }
            None => (),
        }
        write!(fmtr, "{}: {} / [{}]", self.name, self.guard, action_string)
    }
}

#[cfg(test)]
mod tests {

    use crate::*;

    fn john_doe() -> Vec<(SPVariable, SPValue)> {
        let name = v!("name");
        let surname = v!("surname");
        let height = iv!("height");
        let weight = fv!("weight");
        let smart = bv!("smart");
        let alive = bv!("alive");

        vec![
            (name, "John".to_spvalue()),
            (surname, "Doe".to_spvalue()),
            (height, 185.to_spvalue()),
            (weight, 80.0.to_spvalue()),
            (smart, true.to_spvalue()),
            (alive, true.to_spvalue()),
        ]
    }

    #[test]
    fn test_transition_new() {
        let weight = fv!("weight");
        let a1 = a!(weight.clone(), 85.0.wrap());
        let t1 = Transition::new(
            "gains_weight",
            Predicate::TRUE,
            Predicate::TRUE,
            0,
            vec![a1.clone()],
            vec![],
        );
        let t2 = Transition::new(
            "gains_weight",
            Predicate::TRUE,
            Predicate::TRUE,
            0,
            vec![a1],
            vec![],
        );
        assert_eq!(t1, t2);
    }

    #[test]
    fn test_transition_new_macro() {
        let weight = fv!("weight");
        let a1 = a!(weight.clone(), 85.0.wrap());
        let t1 = t_plan!("gains_weight", Predicate::TRUE, vec!(a1.clone()));
        let t2 = t_plan!("gains_weight", Predicate::TRUE, vec!(a1));
        assert_eq!(t1, t2);
    }

    #[test]
    fn test_transition_eval_planning() {
        let s = State::from_vec(&john_doe());
        let weight = fv!("weight");
        let a1 = a!(weight.clone(), 85.0.wrap());
        let t1 = t_plan!("gains_weight", Predicate::TRUE, vec!(a1.clone()));
        let t2 = t_plan!("gains_weight", Predicate::FALSE, vec!(a1));
        assert!(t1.eval_planning(&s));
        assert!(!t2.eval_planning(&s));
    }

    #[test]
    fn test_transition_eval_running() {
        let s = State::from_vec(&john_doe());
        let t1 = t!(
            "gains_weight",
            "true",
            "true",
            0,
            vec!("var:weight <- 85.0", "var:height <- 190"),
            Vec::<&str>::new(),
            &s
        );
        let t2 = t!(
            "gains_weight",
            "true",
            "false",
            0,
            vec!("var:weight <- 85.0"),
            Vec::<&str>::new(),
            &s
        );
        assert!(t1.eval_running(&s));
        assert!(!t2.eval_running(&s));
    }

    #[test]
    #[should_panic]
    fn test_transition_planner_var_in_runner_guard_panic() {
        let s = State::from_vec(&john_doe());
        let t1 = t!(
            "gains_weight",
            "true",
            "var:weight == 85.0",
            0,
            vec!("var:weight <- 85.0", "var:height <- 190"),
            Vec::<&str>::new(),
            &s
        );
        assert!(t1.eval_running(&s));
    }

    // #[test]
    // #[should_panic]
    // fn test_transition_runner_var_in_planner_guard_panic() {
    //     let s = State::from_vec(&john_doe());
    //     let t1 = t!(
    //         "gains_weight",
    //         "var:alive == true",
    //         "true",
    //         vec!("var:weight <- 85.0", "var:height <- 190"),
    //         Vec::<&str>::new(),
    //         &s
    //     );
    //     assert!(t1.eval_running(&s));
    // }

    // #[test]
    // #[should_panic]
    // fn test_transition_planner_var_in_runner_action_panic() {
    //     let s = State::from_vec(&john_doe());
    //     let t1 = t!(
    //         "gains_weight",
    //         "true",
    //         "true",
    //         Vec::<&str>::new(),
    //         vec!("var:weight <- 85.0", "var:height <- 190"),
    //         &s
    //     );
    //     assert!(t1.eval_running(&s));
    // }

    // #[test]
    // #[should_panic]
    // fn test_transition_runner_var_in_planner_action_panic() {
    //     let s = State::from_vec(&john_doe());
    //     let t1 = t!(
    //         "gains_weight",
    //         "true",
    //         "true",
    //         vec!("var:alive <- false", "var:height <- 190"),
    //         Vec::<&str>::new(),
    //         &s
    //     );
    //     assert!(t1.eval_running(&s));
    // }

    #[test]
    fn test_transition_take_planning() {
        let s = State::from_vec(&john_doe());
        let weight = fv!("weight");
        let a1 = a!(weight.clone(), 82.5.wrap());
        let a2 = a!(weight.clone(), 85.0.wrap());
        let t1 = t_plan!("gains_weight", eq!(weight.wrap(), 80.0.wrap()), vec!(a1));
        let t2 = t_plan!(
            "gains_weight_again",
            eq!(weight.wrap(), 82.5.wrap()),
            vec!(a2)
        );
        let s_next_1 = t1.take_planning(&s);
        let s_next_2 = t2.take_planning(&s_next_1);
        let new_state = s.clone().update("weight", 85.0.to_spvalue());
        assert_eq!(s_next_2, new_state);
    }

    // #[test]
    // #[should_panic]
    // fn test_transition_take_planning_panic() {
    //     let s = State::from_vec(&john_doe());
    //     let weight = fv!("weight");
    //     let a1 = a!(weight.clone(), 87.0.wrap());
    //     let t1 = t_plan!("gains_weight", eq!(weight.wrap(), 80.0.wrap()), vec!(a1));
    //     t1.take_planning(&s);
    // }

    // #[test]
    // fn test_transition_take_planning_fail() {
    //     let s = State::from_vec(&john_doe());
    //     let weight = fv!("weight");
    //     let a1 = a!(weight.clone(), 87.0.wrap());
    //     let t1 = t_plan!("gains_weight", eq!(weight.wrap(), 82.5.wrap()), vec!(a1));
    //     let next = t1.take_planning(&s);
    //     assert_eq!(next, s);
    // }

    #[test]
    fn test_transition_action_ordering() {
        let s = State::from_vec(&john_doe());
        let weight = fv!("weight");
        let a1 = a!(weight.clone(), 82.5.wrap());
        let a2 = a!(weight.clone(), 85.0.wrap());
        let t1 = t_plan!(
            "gains_weight",
            eq!(weight.wrap(), 80.0.wrap()),
            vec!(a1, a2)
        );
        let s_next_1 = t1.take_planning(&s);
        assert_eq!(s_next_1.get_value("weight"), 85.0.to_spvalue());
    }

    #[test]
    #[should_panic]
    fn test_transition_action_ordering_panic() {
        let s = State::from_vec(&john_doe());
        let weight = fv!("weight");
        let a1 = a!(weight.clone(), 82.5.wrap());
        let a2 = a!(weight.clone(), 85.0.wrap());
        let a3 = a!(weight.clone(), 87.5.wrap());
        let t1 = t_plan!(
            "gains_weight",
            eq!(weight.wrap(), 80.0.wrap()),
            vec!(a1, a3, a2)
        );
        let s_next_1 = t1.take_planning(&s);
        assert_eq!(s_next_1.get_value("weight"), 87.5.to_spvalue());
    }

    #[test]
    fn test_transition_action_ordering_fail() {
        let s = State::from_vec(&john_doe());
        let weight = fv!("weight");
        let a1 = a!(weight.clone(), 82.5.wrap());
        let a2 = a!(weight.clone(), 85.0.wrap());
        let t1 = t_plan!(
            "gains_weight",
            eq!(weight.wrap(), 80.0.wrap()),
            vec!(a2, a1)
        );
        let s_next_1 = t1.take_planning(&s);
        assert_ne!(s_next_1.get_value("weight"), 85.0.to_spvalue());
    }

    #[test]
    fn test_transition_equality() {
        let weight = fv!("weight");
        let a1 = a!(weight.clone(), 82.5.wrap());
        let a2 = a!(weight.clone(), 85.0.wrap());
        let a3 = a!(weight.clone(), 87.5.wrap());

        // Transitions should be equal even if they have a different name
        let t1 = t_plan!(
            "gains_weight_again",
            eq!(&weight.wrap(), 80.0.wrap()),
            vec!(a1.clone(), a2.clone(), a3.clone())
        );
        let t2 = t_plan!(
            "gains_weight_again",
            eq!(&weight.wrap(), 80.0.wrap()),
            vec!(a1.clone(), a2.clone(), a3.clone())
        );
        let t3 = t_plan!(
            "loses_weight_again",
            eq!(&weight.wrap(), 80.0.wrap()),
            vec!(a1.clone(), a2.clone(), a3.clone())
        );
        let t4 = t_plan!(
            "loses_weight_again",
            eq!(&weight.wrap(), 80.0.wrap()),
            vec!(a3.clone(), a2.clone())
        );
        let t5 = t_plan!(
            "loses_weight_again",
            eq!(&weight.wrap(), 85.0.wrap()),
            vec!(a3.clone(), a2.clone())
        );
        assert_eq!(t1, t2);
        assert_eq!(t1, t3);
        assert_ne!(t3, t4);
        assert_ne!(t4, t5);
    }

    #[test]
    fn test_transition_contained_in_vec() {
        let weight = fv!("weight");
        let a1 = a!(weight.clone(), 82.5.wrap());
        let a2 = a!(weight.clone(), 85.0.wrap());
        let a3 = a!(weight.clone(), 87.5.wrap());

        // Transitions should be equal even if they have a different name
        let t1 = t_plan!(
            "gains_weight_again",
            eq!(&weight.wrap(), 80.0.wrap()),
            vec!(a1.clone(), a2.clone(), a3.clone())
        );
        let t2 = t_plan!(
            "gains_weight_again",
            eq!(&weight.wrap(), 80.0.wrap()),
            vec!(a1.clone(), a2.clone(), a3.clone())
        );
        let t3 = t_plan!(
            "loses_weight_again",
            eq!(&weight.wrap(), 80.0.wrap()),
            vec!(a1.clone(), a2.clone(), a3.clone())
        );
        let t4 = t_plan!(
            "loses_weight_again",
            eq!(&weight.wrap(), 80.0.wrap()),
            vec!(a3.clone(), a2.clone())
        );
        let t5 = t_plan!(
            "loses_weight_again",
            eq!(&weight.wrap(), 85.0.wrap()),
            vec!(a3.clone(), a2.clone())
        );
        let trans2 = vec![t2];
        let trans3 = vec![t3];
        let trans4 = vec![t4.clone()];
        let trans5 = vec![t4, t5];
        assert!(trans2.contains(&t1));
        assert!(trans3.contains(&t1));
        assert!(!trans4.contains(&t1));
        assert!(!trans5.contains(&t1));
    }

    #[test]
    fn test_transition_vec_equality() {
        let weight = fv!("weight");
        let a1 = a!(weight.clone(), 82.5.wrap());
        let a2 = a!(weight.clone(), 85.0.wrap());
        let a3 = a!(weight.clone(), 87.5.wrap());

        // Transitions should be equal even if they have a different name
        let t1 = t_plan!(
            "gains_weight_again",
            eq!(&weight.wrap(), 80.0.wrap()),
            vec!(a1.clone(), a2.clone(), a3.clone())
        );
        let t2 = t_plan!(
            "gains_weight_again",
            eq!(&weight.wrap(), 80.0.wrap()),
            vec!(a1.clone(), a2.clone(), a3.clone())
        );
        let t3 = t_plan!(
            "loses_weight_again",
            eq!(&weight.wrap(), 80.0.wrap()),
            vec!(a1.clone(), a2.clone(), a3.clone())
        );
        let t4 = t_plan!(
            "loses_weight_again",
            eq!(&weight.wrap(), 80.0.wrap()),
            vec!(a3.clone(), a2.clone())
        );
        let trans1 = vec![t1.clone(), t3.clone()];
        let trans2 = vec![t2.clone(), t3.clone()];
        let trans3 = vec![t2.clone(), t4.clone()];
        assert_eq!(trans1, trans2);
        assert_ne!(trans2, trans3);
    }

    // #[test]
    // fn test_asdf() {

    //     let mut operations = vec!();
    //     let competition_state = v!("competition_state");

    //     // Locations of AGVs, can be: kitting, assembly_front, assembly_back, warehouse, UNKNOWN
    //     let agv_1_location = v!("agv_1_location");
    //     let agv_2_location = v!("agv_2_location");
    //     let agv_3_location = v!("agv_3_location");
    //     let agv_4_location = v!("agv_4_location");

    //     let floor_robot_request_trigger = bv!("floor_robot_request_trigger");
    //     let floor_robot_request_state = v!("floor_robot_request_state");
    //     let floor_robot_fail_counter = iv!("floor_robot_fail_counter");
    //     let floor_robot_health = bv!("floor_robot_health");
    //     let floor_robot_command = v!("floor_robot_command");
    //     let floor_robot_current_position_name = v!("floor_robot_current_position_name");

    //     let floor_robot_part_gripper_enabled = bv!("floor_robot_part_gripper_enabled");
    //     let floor_robot_part_gripper_attached = bv!("floor_robot_part_gripper_attached");
    //     let floor_robot_tray_gripper_enabled = bv!("floor_robot_tray_gripper_enabled");
    //     let floor_robot_tray_gripper_attached = bv!("floor_robot_tray_gripper_attached");

    //     let floor_robot_gripper_request_trigger = bv!("floor_robot_gripper_request_trigger");
    //     let floor_robot_gripper_request_state = v!("floor_robot_gripper_request_state");
    //     let floor_robot_gripper_fail_counter = iv!("floor_robot_gripper_fail_counter");
    //     let floor_robot_gripper_command = v!("floor_robot_gripper_command");

    //     // -----------------------------------------------------------------------

    //     let state = State::new();
    //     let state = state.add(assign!(competition_state, SPValue::UNKNOWN));

    //     let state = state.add(assign!(agv_1_location, SPValue::UNKNOWN));
    //     let state = state.add(assign!(agv_2_location, SPValue::UNKNOWN));
    //     let state = state.add(assign!(agv_3_location, SPValue::UNKNOWN));
    //     let state = state.add(assign!(agv_4_location, SPValue::UNKNOWN));

    //     let state = state.add(assign!(floor_robot_request_trigger, false.to_spvalue()));
    //     let state = state.add(assign!(floor_robot_request_state, "initial".to_spvalue()));
    //     let state = state.add(assign!(floor_robot_fail_counter, 0.to_spvalue()));
    //     let state = state.add(assign!(floor_robot_health, false.to_spvalue()));
    //     let state = state.add(assign!(floor_robot_command, SPValue::UNKNOWN));
    //     let state = state.add(assign!(floor_robot_current_position_name, SPValue::UNKNOWN));

    //     let state = state.add(assign!(floor_robot_part_gripper_enabled, false.to_spvalue()));
    //     let state = state.add(assign!(floor_robot_part_gripper_attached, false.to_spvalue()));
    //     let state = state.add(assign!(floor_robot_tray_gripper_enabled, false.to_spvalue()));
    //     let state = state.add(assign!(floor_robot_tray_gripper_attached, false.to_spvalue()));

    //     let state = state.add(assign!(floor_robot_gripper_request_trigger, false.to_spvalue()));
    //     let state = state.add(assign!(floor_robot_gripper_request_state, "initial".to_spvalue()));
    //     let state = state.add(assign!(floor_robot_gripper_fail_counter, 0.to_spvalue()));
    //     let state = state.add(assign!(floor_robot_gripper_command, SPValue::UNKNOWN));

    //     // --------------------------------------------------------------------------

    //     let runner_goal = v!("floor_robot_runner_goal");
    //     let runner_plan = av!("floor_robot_runner_plan");
    //     let runner_plan_info = v!("floor_robot_runner_plan_info");
    //     let runner_plan_state = v!("floor_robot_runner_plan_state");
    //     let runner_plan_current_step = iv!("floor_robot_runner_plan_current_step");
    //     let runner_replan = bv!("floor_robot_runner_replan");
    //     let runner_replanned = bv!("floor_robot_runner_replanned");
    //     let runner_replan_counter = iv!("floor_robot_runner_replan_counter");
    //     let runner_replan_trigger = bv!("floor_robot_runner_replan_trigger");

    //     let state = state.add(assign!(runner_goal, SPValue::UNKNOWN));
    //     let state = state.add(assign!(runner_plan, SPValue::UNKNOWN));
    //     let state = state.add(assign!(runner_plan_info, SPValue::UNKNOWN));
    //     let state = state.add(assign!(runner_plan_state, "empty".to_spvalue()));
    //     let state = state.add(assign!(runner_plan_current_step, SPValue::UNKNOWN));
    //     let state = state.add(assign!(runner_replan, false.to_spvalue()));
    //     let state = state.add(assign!(runner_replanned, false.to_spvalue()));
    //     let state = state.add(assign!(runner_replan_counter, 0.to_spvalue()));
    //     let state = state.add(assign!(runner_replan_trigger, false.to_spvalue()));

    //     for pos in vec![
    //         "kitting_station_1",
    //         "kitting_station_2",
    //         "conveyor_belt"
    //     ] {
    //         operations.push(Operation::new(
    //             &format!("op_floor_robot_direct_joint_move_to_{}", pos),
    //             // precondition
    //             t!(
    //                 // name
    //                 &format!("start_floor_robot_direct_joint_move_to_{}", pos).as_str(),
    //                 // planner guard
    //                 "var:floor_robot_request_state == initial && var:floor_robot_request_trigger == false && var:floor_robot_health == true",
    //                 // runner guard
    //                 "true",
    //                 // planner actions
    //                 vec!(
    //                     &format!("var:floor_robot_command <- direct_joint_move_to_{pos}").as_str(),
    //                     "var:robot_request_trigger <- true"
    //                 ),
    //                 //runner actions
    //                 Vec::<&str>::new(),
    //                 &state
    //             ),
    //             // postcondition
    //             t!(
    //                 // name
    //                 &format!("complete_floor_robot_direct_joint_move_to_{}", pos).as_str(),
    //                 // planner guard
    //                 "true",
    //                 // runner guard
    //                 &format!("var:floor_robot_request_state == succeeded")
    //                     .as_str(),
    //                 // "true",
    //                 // planner actions
    //                 vec!(
    //                     "var:floor_robot_request_trigger <- false",
    //                     "var:floor_robot_request_state <- initial",
    //                     &format!("var:floor_robot_current_position_name <- {pos}")
    //                 ),
    //                 //runner actions
    //                 Vec::<&str>::new(),
    //                 &state
    //             ),
    //             Transition::empty()
    //         ));
    //     }
    // }

    // #[test]
    // fn test_transition_get_vars_all() {
    //     let s = State::from_vec(&john_doe());
    //     let name = v!("name");
    //     let surname = v!("surname");
    //     let height = iv!("height");
    //     let weight = fv!("weight");
    //     let smart = bv!("smart");
    //     let alive = bv!("alive");

    //     let guard = pred_parser::pred("var:smart == TRUE -> (var:alive == FALSE || TRUE)", &s);

    //     // Transitions should be equal even if they have a different name
    //     let t1 = t_plan!("gains_weight_again", eq!(&weight.wrap(), 80.0.wrap()), vec!(a1.clone(), a2.clone(), a3.clone()));
    //     let t2 = t_plan!("gains_weight_again", eq!(&weight.wrap(), 80.0.wrap()), vec!(a1.clone(), a2.clone(), a3.clone()));
    //     let t3 = t_plan!("loses_weight_again", eq!(&weight.wrap(), 80.0.wrap()), vec!(a1.clone(), a2.clone(), a3.clone()));
    //     let t4 = t_plan!("loses_weight_again", eq!(&weight.wrap(), 80.0.wrap()), vec!(a3.clone(), a2.clone()));
    //     let trans1 = vec!(t1.clone(), t3.clone());
    //     let trans2 = vec!(t2.clone(), t3.clone());
    //     let trans3 = vec!(t2.clone(), t4.clone());
    //     assert_eq!(trans1, trans2);
    //     assert_ne!(trans2, trans3);
    // }

    // proptest! {
    //     #![proptest_config(ProptestConfig::with_cases(10))]
    //     #[test]
    //     fn test_transition_mcdc(gripper_ref_val in prop_oneof!("opened", "closed")) {

    //         // let gripper_act = v!("gripper_act", vec!("opened", "closed", "gripping"));
    //         let gripper_ref = v_command!("gripper_ref", vec!("opened", "closed"));

    //         let state = State::new();
    //         // let state = state.add(assign!(gripper_act, "opened".to_spvalue()));
    //         let state = state.add(assign!(gripper_ref, gripper_ref_val.to_spvalue()));

    //         let start_gripper_close = t!(
    //             // name
    //             "start_gripper_close",
    //             // planner guard
    //             "var:gripper_ref != closed",
    //             // runner guard
    //             "true",
    //             // planner actions
    //             vec!("var:gripper_ref <- closed"),
    //             //runner actions
    //             Vec::<&str>::new(),
    //             &state
    //         );

    //         // MC/DC

    //         // 1. Every point of entry and exit in the program has been invoked at least once.
    //         // => This probably doesn't mean much because the transition can be either taken or not taken, there is no alternatives.
    //         if gripper_ref_val == "opened" {
    //             prop_assert!(start_gripper_close.eval_planning(&state));
    //         } else {
    //             prop_assert!(!start_gripper_close.eval_planning(&state));
    //         }

    //         // 2.  Every condition in a decision in the program has taken all possible outcomes at least once.
    //         // => During running, the guard "var:gripper_ref != closed" has to be true at least once.

    //         // 3. Every decision in the program has taken all possible outcomes at least once.
    //         // => Only one decision is present, so need to do extra things here.

    //         // 4. Each condition in a decision has been shown to independently affect
    //         // that decision’s outcome. A condition is shown to independently affect
    //         // a decision’s outcome by varying just that condition while holding fixed
    //         // all other conditions.
    //         // => There is only one variable that can affect the outcome of the program.'

    //     }
    // }

    // proptest! {
    //     #![proptest_config(ProptestConfig::with_cases(10))]
    //     #[test]
    //     fn my_behavior_model_works(gantry_act_val in prop_oneof!("a", "b")) {

    //         let m = rita_model();
    //         // let model = Model::new(&m.0, m.1, m.2, m.3, m.4);
    //         // let gantry_act = v!("gantry_act", vec!("a", "b", "atr"));
    //         let new_state = m.1.update("gantry_act", gantry_act_val.to_spvalue());

    //         let model = Model::new(
    //             "asdf",
    //             new_state.clone(),
    //             m.2,
    //             m.3,
    //             vec!()
    //         );

    //         let plan = bfs_operation_planner(model.state.clone(), extract_goal_from_state(&model.state.clone()), model.operations.clone(), 50);
    //         for p in plan.plan {
    //             println!("{}", p);
    //         }

    //         // let mut runner = TestRunner::default();
    //         // let config = ProptestConfig::with_cases(10); // Set the number of test cases to 10
    //         // runner.set_config(config);

    //         prop_assert!(plan.found);
    //         // prop_assert!(!model.is_empty());
    //         // prop_assert!(model.last_value().is_some());
    //     }
    // }
}
