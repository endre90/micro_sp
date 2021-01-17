use super::*;

use std::fs::File;
use std::io::prelude::*;
use std::collections::HashMap;

pub fn parser(name: &str) -> (ParamPlanningProblem, HashMap<String, Vec<String>>) {
    let mut instance = File::open(&format!("src/models/gripper/eq_instances/{}.pddl", name)).unwrap();
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

    let mut rooms = vec!();
    let mut grippers = vec!();
    let mut balls = vec!();

    for elem in objects_strings {
        if elem.contains(" - room") {
            rooms = elem.split(|c| c == ' ').filter(|x| *x != "-" && *x != "room" && *x != ":objects").map(|x| x.to_string()).collect();
        } else if elem.contains(" - ball") {
            balls = elem.split(|c| c == ' ').filter(|x| *x != "-" && *x != "ball" && *x != ":objects").map(|x| x.to_string()).collect();
        } else if elem.contains(" - gripper") {
            grippers = elem.split(|c| c == ' ').filter(|x| *x != "-" && *x != "gripper" && *x != ":objects").map(|x| x.to_string()).collect();
        }
    }

    let mut objects: HashMap<String, Vec<String>> = HashMap::new();
    objects.insert("room".to_string(), rooms);
    objects.insert("ball".to_string(), balls);
    objects.insert("gripper".to_string(), grippers);
    
    for o in &objects {
        println!("objects: {:?}", o)
    }

    let init_data: Vec<Vec<String>> = init_strings
        .iter()
        .map(|x| x.split(|c| c == ' ')
        .filter(|x| *x != ":init")
        .map(|y| y.to_owned()).collect::<Vec<String>>())
        // .map(|k| k.iter().skip(1).map(|y| y.to_owned()).collect::<Vec<String>>())
        .collect();

    let init_data_clone = init_data.clone();

    let bool_init_data: Vec<Vec<String>> = init_data
        .iter()
        .filter(|x| x[0] == "bool")
        .map(|k| k.iter().skip(1).map(|y| y.to_owned()).collect::<Vec<String>>())
        .collect();

    let enum_init_data: Vec<Vec<String>> = init_data_clone
        .iter()
        .filter(|x| x[0] == "enum")
        .map(|k| k.iter().skip(1).map(|y| y.to_owned()).collect::<Vec<String>>())
        .collect();

    for o in &init_data {
            println!("init data: {:?}", o)
        }
    for o in &bool_init_data {
            println!("bool data: {:?}", o)
        }
    for o in &enum_init_data {
            println!("enum data: {:?}", o)
        }

    let mut predicates = File::open(&format!("src/models/gripper/eq_instances/predicates.pddl")).unwrap();
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
        if next_predicates_line != "NONE" {
            predicates_strings.push(next_predicates_line);
        }
    }

    let enum_predicate_data: Vec<Vec<String>> = predicates_strings
        .iter()
        .map(|x| x.split(|c| c == ' ')
        .map(|y| y.to_owned()).collect::<Vec<String>>())
        .filter(|z| z[0] == "enum")
        .map(|k| k.iter().skip(1).map(|y| y.to_owned()).collect::<Vec<String>>())
        .collect();

    let bool_predicate_data: Vec<Vec<String>> = predicates_strings
        .iter()
        .map(|x| x.split(|c| c == ' ')
        .map(|y| y.to_owned()).collect::<Vec<String>>())
        .filter(|z| z[0] == "bool")
        .map(|k| k.iter().skip(1).map(|y| y.to_owned()).collect::<Vec<String>>())
        .collect();

    for o in &bool_predicate_data {
        println!("bool predicates: {:?}", o)
    }
    for o in &enum_predicate_data {
        println!("enum predicates: {:?}", o)
    }

    fn bool_from_template(data: &Vec<String>, val: bool) -> Predicate {
        match data.len() {
            2 =>  pass!(&new_bool_assign_c!(&format!("{}_{}", data[0], data[1]), val, "c")),
            3 =>  pass!(&new_bool_assign_c!(&format!("{}_{}_{}", data[0], data[1], data[2]), val, "c")),
            4 =>  pass!(&new_bool_assign_c!(&format!("{}_{}_{}_{}", data[0], data[1], data[2], data[3]), val, "c")),
            _ => panic!("wrong pddl predicate or key 1?")
        }
    }

    fn get_enum_domain(name: &str) -> Vec<&str> { 
        match name {
            "at-robby" => vec!("rooma", "roomb"),
            "at" => vec!("rooma", "roomb", "left", "right"),
            "carry" => vec!("ball1", "ball2", "ball3", "ball4", "ball5"),
            _ => panic!("unknown predicate! {}", name)
        }
    }

    fn enum_from_template(data: &Vec<String>) -> Predicate {
        match data.len() {
            2 => pass!(&new_enum_assign_c!(&format!("{}", data[0]), get_enum_domain(&data[0]), &data[1], "c", "c")),
            3 => pass!(&new_enum_assign_c!(&format!("{}_{}", data[0], data[1]), get_enum_domain(&data[0]), &data[2], "c", "c")),
            _ => panic!("wrong pddl predicate or key 2? {}", data.len())
        }
    }

    let mut bool_initial_assert = vec![];
    let mut enum_initial_assert = vec![];

    // generate initial positives for bool vars
    for elem in &bool_init_data {
        bool_initial_assert.push(bool_from_template(elem, true))
    }

    // generate initials for enum vars
    for elem in &enum_init_data {
        enum_initial_assert.push(enum_from_template(elem))
    }

    // generate negatives for bool vars
    for elem in &bool_predicate_data {
        match elem.len() {
            2 => {
                for e1 in objects.get(&elem[1]).unwrap_or(&vec!()).to_vec() {
                    if !bool_initial_assert.contains(&pass!(&new_bool_assign_c!(&format!("{}_{}", elem[0], e1), true, "c"))) {
                        bool_initial_assert.push(pass!(&new_bool_assign_c!(&format!("{}_{}", elem[0], e1), false, "c")))
                    }
                }
            },
            3 => {
                for e1 in objects.get(&elem[1]).unwrap_or(&vec!()).to_vec() {
                    for e2 in objects.get(&elem[2]).unwrap_or(&vec!()).to_vec() {
                        if !bool_initial_assert.contains(&pass!(&new_bool_assign_c!(&format!("{}_{}_{}", elem[0], e1, e2), true, "c"))) {
                            bool_initial_assert.push(pass!(&new_bool_assign_c!(&format!("{}_{}_{}", elem[0], e1, e2), false, "c")))
                        }
                    }
                }
            },
            4 => {
                for e1 in objects.get(&elem[1]).unwrap_or(&vec!()).to_vec() {
                    for e2 in objects.get(&elem[2]).unwrap_or(&vec!()).to_vec() {
                        for e3 in objects.get(&elem[3]).unwrap_or(&vec!()).to_vec() {
                            if !bool_initial_assert.contains(&pass!(&new_bool_assign_c!(&format!("{}_{}_{}_{}", elem[0], e1, e2, e3), true, "c"))) {
                                bool_initial_assert.push(pass!(&new_bool_assign_c!(&format!("{}_{}_{}_{}", elem[0], e1, e2, e3), false, "c")))
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
        .map(|y| y.to_owned()).collect::<Vec<String>>())
        .collect();

    let goal_data_clone = goal_data.clone();

    let bool_goal_data: Vec<Vec<String>> = goal_data
        .iter()
        .filter(|x| x[0] == "bool")
        .map(|k| k.iter().skip(1).map(|y| y.to_owned()).collect::<Vec<String>>())
        .collect();

    let enum_goal_data: Vec<Vec<String>> = goal_data_clone
        .iter()
        .filter(|x| x[0] == "enum")
        .map(|k| k.iter().skip(1).map(|y| y.to_owned()).collect::<Vec<String>>())
        .collect();

    for o in &goal_data {
            println!("goal data: {:?}", o)
        }
    for o in &bool_goal_data {
            println!("bool goal data: {:?}", o)
        }
    for o in &enum_goal_data {
            println!("enum goal data: {:?}", o)
        }

    let mut bool_goal_assert = vec![];
    let mut enum_goal_assert = vec![];

    for elem in &bool_goal_data {
        bool_goal_assert.push(bool_from_template(elem, true))
    }

    for elem in &enum_goal_data {
        enum_goal_assert.push(enum_from_template(elem))
    }

    for d in &bool_initial_assert {
        println!("bool_init_data: {:?}", d);
    }

    for d in &enum_initial_assert {
        println!("enum_init_data: {:?}", d);
    }

    let mut initial_assert = vec!();
    initial_assert.extend(bool_initial_assert);
    initial_assert.extend(enum_initial_assert);

    for d in &bool_goal_assert {
        println!("bool_goal_data: {:?}", d);
    }

    for d in &enum_goal_assert {
        println!("enum_goal_data: {:?}", d);
    }

    let mut goal_assert = vec!();
    goal_assert.extend(bool_goal_assert);
    goal_assert.extend(enum_goal_assert);

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