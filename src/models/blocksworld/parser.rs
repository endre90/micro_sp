use super::*;

use std::fs::File;
use std::io;
use std::io::prelude::*;

pub fn parser(name: &str) -> PlanningProblem {
    let mut f = File::open(&format!("src/models/blocksworld/instances/{}.pddl", name)).unwrap();
    let mut buffer = String::new();

    f.read_to_string(&mut buffer).unwrap();
    println!("{:?}", buffer);
    let mut lines = buffer.lines();

    lines.next();
    lines.next();

    let objects_replaced = match lines.next() {
        Some(x) => x.replace("(:objects ", "").replace(" )", ""),
        None => panic!("parsing object line failed"),
    };

    let objects: Vec<&str> = objects_replaced.split(|c| c == ' ').collect();

    let init_replaced = match lines.next() {
        Some(x) => x.replace("(:INIT (", "").replace("))", ""),
        None => panic!("parsing INIT line failed"),
    };

    let inits: Vec<&str> = init_replaced.split(") (").collect();

    let goal_replaced = match lines.next() {
        Some(x) => x.replace("(:goal (AND (", "").replace(")))", ""),
        None => panic!("parsing goal line failed"),
    };

    let goal: Vec<&str> = goal_replaced.split(") (").collect();

    let blocks = objects;
    let clear_vec = inits
        .iter()
        .map(|x| x.split(" ").collect::<Vec<&str>>())
        .filter(|y| y[0] == "CLEAR")
        .map(|z| z[1])
        .collect::<Vec<&str>>();
    let ontable_vec = inits
        .iter()
        .map(|x| x.split(" ").collect::<Vec<&str>>())
        .filter(|y| y[0] == "ONTABLE")
        .map(|z| z[1])
        .collect::<Vec<&str>>();
    let hand_empty_init = match inits.iter().find(|y| *y == &"HANDEMPTY") {
        Some(_) => true,
        None => false,
    };
    let on_init = inits
        .iter()
        .map(|x| x.split(" ").collect::<Vec<&str>>())
        .filter(|y| y[0] == "ON")
        .map(|z| (z[1], z[2]))
        .collect::<Vec<(&str, &str)>>();
    let hand_empty_goal = match goal.iter().find(|y| *y == &"HANDEMPTY") {
        Some(_) => true,
        None => false,
    };
    let on_goal = goal
        .iter()
        .map(|x| x.split(" ").collect::<Vec<&str>>())
        .filter(|y| y[0] == "ON")
        .map(|z| (z[1], z[2]))
        .collect::<Vec<(&str, &str)>>();

    let model = domain::blocksworld_model_enumerated_booleans_invariants(&blocks);

    println!("blocks: {:?}", blocks);
    println!("clear_init: {:?}", clear_vec);
    println!("ontable_init: {:?}", ontable_vec);
    println!("on_init: {:?}", on_init);
    println!("on_goal: {:?}", on_goal);

    // explicitly have to say that others are not clear?
    let mut clear_predicates = vec![];
    let domain = vec!["true", "false"];

    let unclear_vec = IterOps::difference(blocks.clone(), clear_vec.clone());

    for x in clear_vec {
        clear_predicates.push(
            pand!(
                &pass!(&new_enum_assign_c!(&format!("clear_{}", x), &domain, "true"))
            )
        )
    }

    for x in unclear_vec {
        clear_predicates.push(
            pand!(
                &pass!(&new_enum_assign_c!(&format!("clear_{}", x), &domain, "false"))
            )
        )
    }

    let mut ontable_predicates = vec![];
    for x in ontable_vec {
        ontable_predicates.push(
            pand!(
                &pass!(&new_enum_assign_c!(&format!("ontable_{}", x), &domain, "true"))
            )
        )
    }

    let mut on_predicates = vec![];
    for (b1, b2) in on_init {
        on_predicates.push(
            pand!(
                &pass!(&new_enum_assign_c!(&format!("{}_on_{}", b1, b2), &domain, "true"))
            )
        )
    }

    let initial = Predicate::AND(vec![
        Predicate::AND(clear_predicates),
        Predicate::AND(ontable_predicates),
        Predicate::AND(on_predicates),
        pass!(&new_enum_assign_c!(&format!("hand_empty"), &domain, "true"))
    ]);

    let mut goal_on_predicates = vec![];
    for (b1, b2) in on_goal {
        goal_on_predicates.push(
            pand!(
                &pass!(&new_enum_assign_c!(&format!("{}_on_{}", b1, b2), &domain, "true"))
            )
        )
    }

    let goal = Predicate::AND(goal_on_predicates);
    let problem = PlanningProblem::new(
        "blocks_world",
        &initial,
        &goal,
        &model.0,
        &model.1,
        &50,
    );

    problem
    
}

#[test]
fn test_parser() {
    let result = incremental(&parser("probBLOCKS-7-0"), 1200);
    pprint_result_trans_only(&result)
}