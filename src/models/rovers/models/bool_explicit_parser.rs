use super::*;

use std::fs::File;
use std::io;
use std::io::prelude::*;
use std::collections::HashMap;

pub fn parser(name: &str) -> (ParamPlanningProblem, HashMap<String, Vec<String>>) {
    let mut instance = File::open(&format!("src/models/rovers/instances/{}.pddl", name)).unwrap();
    let mut instance_buffer = String::new();

    instance.read_to_string(&mut instance_buffer).unwrap();
    let mut instance_lines = instance_buffer.lines();

    let mut objects_strings = vec![];
    let mut init_strings = vec![];
    let mut goal_strings = vec![];

    let mut next_instance_line = "SOME";

    while next_instance_line != "NONE" {
        
        next_instance_line = match instance_lines.next() {
            Some(x) => x,
            None => "NONE"
        };

        if next_instance_line.contains(":objects") {
            objects_strings.push(next_instance_line)
        } else if next_instance_line.contains(":init") {
            init_strings.push(next_instance_line)
        } else if next_instance_line.contains(":goal") {
            goal_strings.push(next_instance_line)
        }
    }

    let mut landers = vec!();
    let mut rovers = vec!();
    let mut objectives = vec!();
    let mut cameras = vec!();
    let mut modes = vec!();
    let mut stores = vec!();
    let mut waypoints = vec!();

    for elem in objects_strings {
        if elem.contains("Lander") {
            landers = elem.split(|c| c == ' ').filter(|x| *x != "-" && *x != "Lander" && *x != ":objects").map(|x| x.to_string()).collect();
        } else if elem.contains("Rover") {
            rovers = elem.split(|c| c == ' ').filter(|x| *x != "-" && *x != "Rover" && *x != ":objects").map(|x| x.to_string()).collect();
        } else if elem.contains("Objective") {
            objectives = elem.split(|c| c == ' ').filter(|x| *x != "-" && *x != "Objective" && *x != ":objects").map(|x| x.to_string()).collect();
        } else if elem.contains("Camera") {
            cameras = elem.split(|c| c == ' ').filter(|x| *x != "-" && *x != "Camera" && *x != ":objects").map(|x| x.to_string()).collect();
        } else if elem.contains("Mode") {
            modes = elem.split(|c| c == ' ').filter(|x| *x != "-" && *x != "Mode" && *x != ":objects").map(|x| x.to_string()).collect();
        } else if elem.contains("Store") {
            stores = elem.split(|c| c == ' ').filter(|x| *x != "-" && *x != "Store" && *x != ":objects").map(|x| x.to_string()).collect();
        } else if elem.contains("Waypoint") {
            waypoints = elem.split(|c| c == ' ').filter(|x| *x != "-" && *x != "Waypoint" && *x != ":objects").map(|x| x.to_string()).collect();
        }
    }

    let mut objects: HashMap<String, Vec<String>> = HashMap::new();
    objects.insert("lander".to_string(), landers);
    objects.insert("rover".to_string(), rovers);
    objects.insert("objective".to_string(), objectives);
    objects.insert("camera".to_string(), cameras);
    objects.insert("mode".to_string(), modes);
    objects.insert("store".to_string(), stores);
    objects.insert("waypoint".to_string(), waypoints);

    let init_data: Vec<Vec<String>> = init_strings
        .iter()
        .map(|x| x.split(|c| c == ' ')
        .filter(|x| *x != ":init")
        .map(|y| y.to_owned()).collect())
        .collect();

    let mut predicates = File::open(&format!("src/models/rovers/instances/predicates.pddl")).unwrap();
    let mut predicates_buffer = String::new();

    predicates.read_to_string(&mut predicates_buffer).unwrap();
    let mut predicates_lines = predicates_buffer.lines();
    let mut predicates_strings = vec![];
    let mut next_predicates_line = "SOME";

    while next_predicates_line != "NONE" {
        next_predicates_line = match predicates_lines.next() {
            Some(x) => x,
            None => "NONE"
        };

        predicates_strings.push(next_predicates_line);
    }

    let predicate_data: Vec<Vec<String>> = predicates_strings
        .iter()
        .map(|x| x.split(|c| c == ' ')
        .map(|y| y.to_owned()).collect())
        .collect();

    fn from_template(data: &Vec<String>, val: bool) -> Predicate {
        match data.len() {
            2 =>  pass!(&new_bool_assign_c!(&format!("{}_{}", data[0], data[1]), val, "c")),
            3 =>  pass!(&new_bool_assign_c!(&format!("{}_{}_{}", data[0], data[1], data[2]), val, "c")),
            4 =>  pass!(&new_bool_assign_c!(&format!("{}_{}_{}_{}", data[0], data[1], data[2], data[3]), val, "c")),
            _ => panic!("wrong pddl predicate or key 1?")
        }
    }

    let mut initial_assert = vec![];

    // generate positives
    for elem in &init_data {
        initial_assert.push(from_template(elem, true))
    }

    // generate negatives
    for elem in &predicate_data {
        match elem.len() {
            2 => {
                for e1 in objects.get(&elem[1]).unwrap_or(&vec!()).to_vec() {
                    if !initial_assert.contains(&pass!(&new_bool_assign_c!(&format!("{}_{}", elem[0], e1), true, "c"))) {
                        initial_assert.push(pass!(&new_bool_assign_c!(&format!("{}_{}", elem[0], e1), false, "c")))
                    }
                }
            },
            3 => {
                for e1 in objects.get(&elem[1]).unwrap_or(&vec!()).to_vec() {
                    for e2 in objects.get(&elem[2]).unwrap_or(&vec!()).to_vec() {
                        if !initial_assert.contains(&pass!(&new_bool_assign_c!(&format!("{}_{}_{}", elem[0], e1, e2), true, "c"))) {
                            initial_assert.push(pass!(&new_bool_assign_c!(&format!("{}_{}_{}", elem[0], e1, e2), false, "c")))
                        }
                    }
                }
            },
            4 => {
                for e1 in objects.get(&elem[1]).unwrap_or(&vec!()).to_vec() {
                    for e2 in objects.get(&elem[2]).unwrap_or(&vec!()).to_vec() {
                        for e3 in objects.get(&elem[3]).unwrap_or(&vec!()).to_vec() {
                            if !initial_assert.contains(&pass!(&new_bool_assign_c!(&format!("{}_{}_{}_{}", elem[0], e1, e2, e3), true, "c"))) {
                                initial_assert.push(pass!(&new_bool_assign_c!(&format!("{}_{}_{}_{}", elem[0], e1, e2, e3), false, "c")))
                            }
                        }
                    }
                }
            },
            _ => ()
        }
    }

    let goal_data: Vec<Vec<String>> = goal_strings
        .iter()
        .map(|x| x.split(|c| c == ' ')
        .filter(|x| *x != ":goal")
        .map(|y| y.to_owned()).collect())
        .collect();

    let mut goal_assert = vec![];

    // generate goal positives
    for elem in &goal_data {
        goal_assert.push(from_template(elem, true))
    }

    for d in &initial_assert {
        println!("init_data: {:?}", d);
    }
    
    let problem = ParamPlanningProblem::new(
        name, 
        &ParamPredicate::new(&initial_assert), 
        &ParamPredicate::new(&goal_assert), 
        &vec!(), 
        &Predicate::TRUE,
        &vec!()
    );

    (problem, objects)
}