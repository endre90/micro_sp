#![allow(unused_imports)]
#![allow(dead_code)]
use crate::{
    a, assign, av_command, av_estimated, av_measured, av_runner, bv_command, bv_estimated,
    bv_measured, bv_runner, eq, fv_command, fv_estimated, fv_measured, fv_runner, iv_command,
    iv_estimated, iv_measured, iv_runner, t, t_plan, v_command, v_estimated, v_measured, v_runner,
    Operation,
};
use crate::{
    pred_parser, Action, Predicate, SPAssignment, SPValue, SPValueType, SPVariable, SPVariableType,
    State, ToSPValue, ToSPWrapped, ToSPWrappedVar, Transition,
};
use std::collections::{HashMap, HashSet};
use std::f32::consts::E;
use proptest::prelude::*;

fn john_doe() -> Vec<(SPVariable, SPValue)> {
    let name = v_estimated!("name", vec!("John", "Jack"));
    let surname = v_estimated!("surname", vec!("Doe", "Crawford"));
    let height = iv_estimated!("height", vec!(180, 185, 190));
    let weight = fv_estimated!("weight", vec!(80.0, 82.5, 85.0));
    let smart = bv_estimated!("smart");
    let alive = bv_runner!("alive");

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
    let weight = fv_estimated!("weight", vec!(80.0, 82.5, 85.0));
    let a1 = a!(weight.clone(), 85.0.wrap());
    let t1 = Transition::new(
        "gains_weight",
        Predicate::TRUE,
        Predicate::TRUE,
        vec![a1.clone()],
        vec![],
    );
    let t2 = Transition::new(
        "gains_weight",
        Predicate::TRUE,
        Predicate::TRUE,
        vec![a1],
        vec![],
    );
    assert_eq!(t1, t2);
}

#[test]
fn test_transition_new_macro() {
    let weight = fv_estimated!("weight", vec!(80.0, 82.5, 85.0));
    let a1 = a!(weight.clone(), 85.0.wrap());
    let t1 = t_plan!("gains_weight", Predicate::TRUE, vec!(a1.clone()));
    let t2 = t_plan!("gains_weight", Predicate::TRUE, vec!(a1));
    assert_eq!(t1, t2);
}

#[test]
fn test_transition_eval_planning() {
    let s = State::from_vec(&john_doe());
    let weight = fv_estimated!("weight", vec!(80.0, 82.5, 85.0));
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
        vec!("var:weight <- 85.0", "var:height <- 190"),
        Vec::<&str>::new(),
        &s
    );
    let t2 = t!(
        "gains_weight",
        "true",
        "false",
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
        vec!("var:weight <- 85.0", "var:height <- 190"),
        Vec::<&str>::new(),
        &s
    );
    assert!(t1.eval_running(&s));
}

#[test]
#[should_panic]
fn test_transition_runner_var_in_planner_guard_panic() {
    let s = State::from_vec(&john_doe());
    let t1 = t!(
        "gains_weight",
        "var:alive == true",
        "true",
        vec!("var:weight <- 85.0", "var:height <- 190"),
        Vec::<&str>::new(),
        &s
    );
    assert!(t1.eval_running(&s));
}

#[test]
#[should_panic]
fn test_transition_planner_var_in_runner_action_panic() {
    let s = State::from_vec(&john_doe());
    let t1 = t!(
        "gains_weight",
        "true",
        "true",
        Vec::<&str>::new(),
        vec!("var:weight <- 85.0", "var:height <- 190"),
        &s
    );
    assert!(t1.eval_running(&s));
}

#[test]
#[should_panic]
fn test_transition_runner_var_in_planner_action_panic() {
    let s = State::from_vec(&john_doe());
    let t1 = t!(
        "gains_weight",
        "true",
        "true",
        vec!("var:alive <- false", "var:height <- 190"),
        Vec::<&str>::new(),
        &s
    );
    assert!(t1.eval_running(&s));
}

#[test]
fn test_transition_take_planning() {
    let s = State::from_vec(&john_doe());
    let weight = fv_estimated!("weight", vec!(80.0, 82.5, 85.0));
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
//     let weight = fv_estimated!("weight", vec!(80.0, 82.5, 85.0));
//     let a1 = a!(weight.clone(), 87.0.wrap());
//     let t1 = t_plan!("gains_weight", eq!(weight.wrap(), 80.0.wrap()), vec!(a1));
//     t1.take_planning(&s);
// }

// #[test]
// fn test_transition_take_planning_fail() {
//     let s = State::from_vec(&john_doe());
//     let weight = fv_estimated!("weight", vec!(80.0, 82.5, 85.0));
//     let a1 = a!(weight.clone(), 87.0.wrap());
//     let t1 = t_plan!("gains_weight", eq!(weight.wrap(), 82.5.wrap()), vec!(a1));
//     let next = t1.take_planning(&s);
//     assert_eq!(next, s);
// }

#[test]
fn test_transition_action_ordering() {
    let s = State::from_vec(&john_doe());
    let weight = fv_estimated!("weight", vec!(80.0, 82.5, 85.0, 87.5));
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
    let weight = fv_estimated!("weight", vec!(80.0, 82.5, 85.0, 87.5));
    let a1 = a!(weight.clone(), 82.5.wrap());
    let a2 = a!(weight.clone(), 85.0.wrap());
    let a3 = a!(weight.clone(), 87.5.wrap());
    let t1 = t_plan!(
        "gains_weight",
        eq!(weight.wrap(), 80.0.wrap()),
        vec!(a1, a2, a3)
    );
    let s_next_1 = t1.take_planning(&s);
    assert_eq!(s_next_1.get_value("weight"), 87.5.to_spvalue());
}

#[test]
fn test_transition_action_ordering_fail() {
    let s = State::from_vec(&john_doe());
    let weight = fv_estimated!("weight", vec!(80.0, 82.5, 85.0, 87.5));
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
    let weight = fv_estimated!("weight", vec!(80.0, 82.5, 85.0, 87.5));
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
    let weight = fv_estimated!("weight", vec!(80.0, 82.5, 85.0, 87.5));
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
    let weight = fv_estimated!("weight", vec!(80.0, 82.5, 85.0, 87.5));
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
// fn test_transition_get_vars_all() {
//     let s = State::from_vec(&john_doe());
//     let name = v_estimated!("name", vec!("John", "Jack"));
//     let surname = v_estimated!("surname", vec!("Doe", "Crawford"));
//     let height = iv_estimated!("height", vec!(180, 185, 190));
//     let weight = fv_estimated!("weight", vec!(80.0, 82.5, 85.0));
//     let smart = bv_estimated!("smart");
//     let alive = bv_runner!("alive");

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

proptest! {
    #![proptest_config(ProptestConfig::with_cases(10))]
    #[test]
    fn test_transition_mcdc(gripper_ref_val in prop_oneof!("opened", "closed")) {

        // let gripper_act = v_measured!("gripper_act", vec!("opened", "closed", "gripping"));
        let gripper_ref = v_command!("gripper_ref", vec!("opened", "closed"));

        let state = State::new();
        // let state = state.add(assign!(gripper_act, "opened".to_spvalue()));
        let state = state.add(assign!(gripper_ref, gripper_ref_val.to_spvalue()));

        let start_gripper_close = t!(
            // name
            "start_gripper_close",
            // planner guard
            "var:gripper_ref != closed",
            // runner guard
            "true",
            // planner actions
            vec!("var:gripper_ref <- closed"),
            //runner actions
            Vec::<&str>::new(),
            &state
        );

        // MC/DC

        // 1. Every point of entry and exit in the program has been invoked at least once.
        // => This probably doesn't mean much because the transition can be either taken or not taken, there is no alternatives.
        if gripper_ref_val == "opened" {
            prop_assert!(start_gripper_close.eval_planning(&state));
        } else {
            prop_assert!(!start_gripper_close.eval_planning(&state));
        }
        

        // 2.  Every condition in a decision in the program has taken all possible outcomes at least once.
        // => During running, the guard "var:gripper_ref != closed" has to be true at least once.

        // 3. Every decision in the program has taken all possible outcomes at least once.
        // => Only one decision is present, so need to do extra things here.

        // 4. Each condition in a decision has been shown to independently affect
        // that decision’s outcome. A condition is shown to independently affect
        // a decision’s outcome by varying just that condition while holding fixed
        // all other conditions.
        // => There is only one variable that can affect the outcome of the program.'

    }
}

// proptest! {
//     #![proptest_config(ProptestConfig::with_cases(10))]
//     #[test]
//     fn my_behavior_model_works(gantry_act_val in prop_oneof!("a", "b")) {

//         let m = rita_model();
//         // let model = Model::new(&m.0, m.1, m.2, m.3, m.4);
//         // let gantry_act = v_measured!("gantry_act", vec!("a", "b", "atr"));
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
