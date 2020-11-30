use super::*;

use std::fs::File;
use std::io;
use std::io::prelude::*;

pub fn parser(name: &str) -> (ParamPlanningProblem, Vec<String>) {
    let nr_balls = name.split("_").map(|x| x.to_owned()).collect::<Vec<String>>()[1].parse().unwrap();
    let balls = (0..nr_balls).collect::<Vec<u64>>().iter().map(|x| format!("ball{}", x)).collect();

    let mut initial_vec = vec!();
    for b in &balls {
        initial_vec.push(
            pass!(&new_bool_assign_c!(&format!("at_{}_rooma", b), true, String::from(b)))
        );
        initial_vec.push(
            pass!(&new_bool_assign_c!(&format!("at_{}_roomb", b), false, String::from(b)))
        );
        for g in vec!("left", "right") {
            initial_vec.push(
                pass!(&new_bool_assign_c!(&format!("{}_carry_{}", g, b), false, "g"))
            );
        }         
    }

    initial_vec.push(
        pass!(&new_bool_assign_c!(&format!("at-robby_rooma"), true, "r"))
    );

    initial_vec.push(
        pass!(&new_bool_assign_c!(&format!("at-robby_roomb"), false, "r"))
    );

    let mut goal_vec = vec!();
    for b in &balls {
        goal_vec.push(
            pass!(&new_bool_assign_c!(&format!("at_{}_roomb", b), true, String::from(b)))
        )
    }

     let problem = ParamPlanningProblem::new(
        name, 
        &ParamPredicate::new(&initial_vec), 
        &ParamPredicate::new(&goal_vec),
        &vec!(), 
        &Predicate::TRUE,
        &vec!()
    );

    (problem, balls)
}