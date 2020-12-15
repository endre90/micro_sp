use super::*;

use std::fs::File;
use std::io;
use std::io::prelude::*;

pub fn parser(name: &str) -> (ParamPlanningProblem, Vec<String>, Vec<String>, Vec<String>, Vec<String>, Vec<String>, Vec<String>, Vec<String>) {
    let mut f = File::open(&format!("src/models/rovers/instances/{}.pddl", name)).unwrap();
    let mut buffer = String::new();

    f.read_to_string(&mut buffer).unwrap();
    // println!("{:?}", buffer);
    let mut lines = buffer.lines();

    let mut objects_strings = vec![];
    let mut init_strings = vec![];
    let mut goal_strings = vec![];

    let mut next_line = "SOME";

    while next_line != "NONE" {
        
        next_line = match lines.next() {
            Some(x) => x,
            None => "NONE"
        };

        if next_line.contains(":objects") {
            objects_strings.push(next_line)
        } else if next_line.contains(":init") {
            init_strings.push(next_line)
        } else if next_line.contains(":goal") {
            goal_strings.push(next_line)
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
            landers = elem.split(|c| c == ' ').filter(|x| *x != "-" && *x != "Lander" && *x != ":objects").collect();
        } else if elem.contains("Rover") {
            rovers = elem.split(|c| c == ' ').filter(|x| *x != "-" && *x != "Rover" && *x != ":objects").collect();
        } else if elem.contains("Objective") {
            objectives = elem.split(|c| c == ' ').filter(|x| *x != "-" && *x != "Objective" && *x != ":objects").collect();
        } else if elem.contains("Camera") {
            cameras = elem.split(|c| c == ' ').filter(|x| *x != "-" && *x != "Camera" && *x != ":objects").collect();
        } else if elem.contains("Mode") {
            modes = elem.split(|c| c == ' ').filter(|x| *x != "-" && *x != "Mode" && *x != ":objects").collect();
        } else if elem.contains("Store") {
            stores = elem.split(|c| c == ' ').filter(|x| *x != "-" && *x != "Store" && *x != ":objects").collect();
        } else if elem.contains("Waypoint") {
            waypoints = elem.split(|c| c == ' ').filter(|x| *x != "-" && *x != "Waypoint" && *x != ":objects").collect();
        }
    }

    println!("rovers: {:?}", rovers);
    println!("landers: {:?}", landers);
    println!("cameras: {:?}", cameras);
    println!("waypoints: {:?}", waypoints);
    println!("stores: {:?}", stores);

    // match li

    // lines.next();
    // lines.next();

    // let objects_replaced = match lines.next() {
    //     Some(x) => x.replace("(:objects ", "").replace(" )", ""),
    //     None => panic!("parsing object line failed"),
    // };

    // let objects: Vec<&str> = objects_replaced.split(|c| c == ' ').collect();

    // let init_replaced = match lines.next() {
    //     Some(x) => x.replace("(:INIT (", "").replace("))", ""),
    //     None => panic!("parsing INIT line failed"),
    // };

    // let inits: Vec<&str> = init_replaced.split(") (").collect();

    // let goal_replaced = match lines.next() {
    //     Some(x) => x.replace("(:goal (AND (", "").replace(")))", ""),
    //     None => panic!("parsing goal line failed"),
    // };

    // let goal: Vec<&str> = goal_replaced.split(") (").collect();

    // let blocks = objects;
    // let clear_vec = inits
    //     .iter()
    //     .map(|x| x.split(" ").collect::<Vec<&str>>())
    //     .filter(|y| y[0] == "CLEAR")
    //     .map(|z| z[1])
    //     .collect::<Vec<&str>>();
    // let ontable_vec = inits
    //     .iter()
    //     .map(|x| x.split(" ").collect::<Vec<&str>>())
    //     .filter(|y| y[0] == "ONTABLE")
    //     .map(|z| z[1])
    //     .collect::<Vec<&str>>();
    // let hand_empty_init = match inits.iter().find(|y| *y == &"HANDEMPTY") {
    //     Some(_) => true,
    //     None => false,
    // };
    // let on_init = inits
    //     .iter()
    //     .map(|x| x.split(" ").collect::<Vec<&str>>())
    //     .filter(|y| y[0] == "ON")
    //     .map(|z| (z[1], z[2]))
    //     .collect::<Vec<(&str, &str)>>();
    // let hand_empty_goal = match goal.iter().find(|y| *y == &"HANDEMPTY") {
    //     Some(_) => true,
    //     None => false,
    // };
    // let on_goal = goal
    //     .iter()
    //     .map(|x| x.split(" ").collect::<Vec<&str>>())
    //     .filter(|y| y[0] == "ON")
    //     .map(|z| (z[1], z[2]))
    //     .collect::<Vec<(&str, &str)>>();

    // // let model = domain::blocksworld_model_boolerated_booleans_invariants(&blocks);

    // println!("blocks: {:?}", blocks);
    // println!("clear_init: {:?}", clear_vec);
    // println!("ontable_init: {:?}", ontable_vec);
    // println!("on_init: {:?}", on_init);
    // println!("on_goal: {:?}", on_goal);

    // // explicitly have to say that others are not clear?
    // let mut clear_predicates = vec![];

    // let unclear_vec = IterOps::difference(blocks.clone(), clear_vec.clone());
    // let no_ontable_vec = IterOps::difference(blocks.clone(), ontable_vec.clone());

    // for x in clear_vec {
    //     clear_predicates.push(
    //         pand!(
    //             &pass!(&new_bool_assign_c!(&format!("clear_{}", x), true, "block"))
    //         )
    //     )
    // }

    // for x in unclear_vec {
    //     clear_predicates.push(
    //         pand!(
    //             &pass!(&new_bool_assign_c!(&format!("clear_{}", x), false, "block"))
    //         )
    //     )
    // }

    // let mut ontable_predicates = vec![];
    // for x in ontable_vec {
    //     ontable_predicates.push(
    //         pand!(
    //             &pass!(&new_bool_assign_c!(&format!("ontable_{}", x), true, "block"))
    //         )
    //     )
    // }

    // for x in no_ontable_vec {
    //     ontable_predicates.push(
    //         pand!(
    //             &pass!(&new_bool_assign_c!(&format!("ontable_{}", x), false, "block"))
    //         )
    //     )
    // }

    // let mut on_predicates = vec![];
    // for (b1, b2) in &on_init {
    //     on_predicates.push(
    //         pand!(
    //             &pass!(&new_bool_assign_c!(&format!("{}_on_{}", b1, b2), true, "block"))
    //         )
    //     )
    // }

    // for b1 in &blocks {
    //     for b2 in &blocks {
    //         if b1 != b2 {
    //             if !on_init.contains(&(*b1, *b2)) {
    //                 on_predicates.push(
    //                     pand!(
    //                         &pass!(&new_bool_assign_c!(&format!("{}_on_{}", b1, b2), false, "block"))
    //                     )
    //                 )
    //             }
    //         }
    //     }
    // }

    // let mut holding_predicates = vec![];
    // for x in &blocks {
    //     holding_predicates.push(
    //         pand!(
    //             &pass!(&new_bool_assign_c!(&format!("holding_{}", x), false, "hand"))
    //         )
    //     )
    // }

    // let initial = ParamPredicate::new(&vec![
    //     Predicate::AND(clear_predicates),
    //     Predicate::AND(ontable_predicates),
    //     Predicate::AND(on_predicates),
    //     Predicate::AND(holding_predicates),
    //     pass!(&new_bool_assign_c!(&format!("hand_empty"), true, "hand"))
    // ]);



    // let mut goal_on_predicates = vec![];
    // for (b1, b2) in on_goal {
    //     goal_on_predicates.push(
    //         pass!(&new_bool_assign_c!(&format!("{}_on_{}", b1, b2), true, "block"))
    //     )
    // }

    // // adjusted for goal decomposition
    // let reversed_goal_for_heuristics = goal_on_predicates.iter().rev().cloned().collect();
    // let goal = ParamPredicate::new(&reversed_goal_for_heuristics);

    // // let goal = ParamPredicate::new(&goal_on_predicates);


    // // let goal = ParamPredicate::new(&goal_on_predicates);
    let problem = ParamPlanningProblem::new(
        name, 
        &ParamPredicate::new(&vec!(Predicate::TRUE)), 
        &ParamPredicate::new(&vec!(Predicate::TRUE)), 
        &vec!(), 
        &Predicate::TRUE,
        &vec!()
    );

    (problem, 
        rovers.iter().map(|x| x.to_string()).collect(), 
        landers.iter().map(|x| x.to_string()).collect(), 
        waypoints.iter().map(|x| x.to_string()).collect(), 
        objectives.iter().map(|x| x.to_string()).collect(), 
        cameras.iter().map(|x| x.to_string()).collect(), 
        modes.iter().map(|x| x.to_string()).collect(), 
        stores.iter().map(|x| x.to_string()).collect()
    ) //.iter().map(|x| x.to_string()).collect())
}