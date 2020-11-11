use super::*;

use std::fs::File;
use std::io;
use std::io::prelude::*;

pub fn parser(name: &str) -> (ParamPlanningProblem, Vec<String>) {
    let mut f = File::open(&format!("src/models/blocksworld/instances/{}.pddl", name)).unwrap();
    let mut buffer = String::new();

    f.read_to_string(&mut buffer).unwrap();
    // println!("{:?}", buffer);
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

    let on_init = inits
        .iter()
        .map(|x| x.split(" ").collect::<Vec<&str>>())
        .filter(|y| y[0] == "ON")
        .map(|z| (z[1], z[2]))
        .collect::<Vec<(&str, &str)>>();

    let ontable_vec = inits
        .iter()
        .map(|x| x.split(" ").collect::<Vec<&str>>())
        .filter(|y| y[0] == "ONTABLE")
        .map(|z| z[1])
        .collect::<Vec<&str>>();
    let on_goal = goal
        .iter()
        .map(|x| x.split(" ").collect::<Vec<&str>>())
        .filter(|y| y[0] == "ON")
        .map(|z| (z[1], z[2]))
        .collect::<Vec<(&str, &str)>>();

    let on_domain = blocks
        .iter()
        .chain(vec!["GRIP", "TABLE"].iter())
        .cloned()
        .collect::<Vec<_>>();

    println!("blocks: {:?}", blocks);
    println!("clear_init: {:?}", clear_vec);
    println!("ontable_init: {:?}", ontable_vec);
    println!("on_init: {:?}", on_init);
    println!("on_goal: {:?}", on_goal);

    let mut clear_predicates = vec![];
    for x in clear_vec {
        clear_predicates.push(pand!(&pass!(&new_bool_assign_c!(
            &format!("clear_{}", x),
            true,
            "on"
        ))))
    }

    let mut ontable_predicates = vec![];
    for x in ontable_vec {
        ontable_predicates.push(pand!(pass!(&new_enum_assign_c!(
            &format!("{}_on", x),
            &on_domain,
            "TABLE",
            "on",
            "on"
        ))))
    }

    let mut on_predicates = vec![];
    for (b1, b2) in on_init {
        on_predicates.push(pass!(&new_enum_assign_c!(
            &format!("{}_on", b1),
            &on_domain,
            &format!("{}", b2),
            "on",
            "on"
        )))
    }

    let initial = ParamPredicate::new(&vec![
        Predicate::AND(clear_predicates),
        Predicate::AND(on_predicates),
        Predicate::AND(ontable_predicates),
    ]);

    let mut goal_on_predicates = vec![];
    for (b1, b2) in &on_goal {
        goal_on_predicates.push(pass!(&new_enum_assign_c!(
            &format!("{}_on", b1),
            &on_domain,
            &format!("{}", b2),
            "on",
            "on"
        )));
    }

    // goal refinement for faster "goal decopmosition" planning
    let on_vector = on_goal.iter().map(|x| x.0).collect();
    let goal_ontable_blocks = blocks.clone().difference(on_vector);
    for otb in &goal_ontable_blocks{
        goal_on_predicates.push(pass!(&new_enum_assign_c!(
            &format!("{}_on", otb),
            &on_domain,
            "TABLE",
            "on",
            "on"
        )));
    }

    // let init = ParamPredicate::new(
    //     &vec!(
    //         Predicate::AND(
    //             vec!(
    //                 pass!(&new_bool_assign_c!(&format!("clear_{}", "A"), true, "clear")),
    //                 pass!(&new_bool_assign_c!(&format!("clear_{}", "B"), true, "clear")),
    //                 pass!(&new_bool_assign_c!(&format!("clear_{}", "C"), true, "clear")),
    //                 pass!(&new_bool_assign_c!(&format!("clear_{}", "D"), true, "clear")),
    //                 pass!(&new_enum_assign_c!(&format!("{}_on", "A"), &on_domain, "TABLE", "on", "on")),
    //                 pass!(&new_enum_assign_c!(&format!("{}_on", "B"), &on_domain, "TABLE", "on", "on")),
    //                 pass!(&new_enum_assign_c!(&format!("{}_on", "C"), &on_domain, "TABLE", "on", "on")),
    //                 pass!(&new_enum_assign_c!(&format!("{}_on", "D"), &on_domain, "TABLE", "on", "on"))
    //             )
    //         )
    //     )
    // );

    // let goal = ParamPredicate::new(
    //     &vec!(
    //         Predicate::AND(
    //             vec!(
    //                 pass!(&new_enum_assign_c!(&format!("{}_on", "D"), &on_domain, "C", "on", "on")),
    //                 pass!(&new_enum_assign_c!(&format!("{}_on", "C"), &on_domain, "B", "on", "on")),
    //                 pass!(&new_enum_assign_c!(&format!("{}_on", "B"), &on_domain, "A", "on", "on"))
    //             )
    //         )
    //     )
    // );

    let reversed_goal_for_heuristics = goal_on_predicates.iter().rev().cloned().collect();
    let goal = ParamPredicate::new(&reversed_goal_for_heuristics);

    // let goal = ParamPredicate::new(&goal_on_predicates);
    let problem =
        ParamPlanningProblem::new(name, &initial, &goal, &vec![], &Predicate::TRUE, &vec![]);

    (problem, blocks.iter().map(|x| x.to_string()).collect())

    // let problem = ParamPlanningProblem::new(
    //     name,
    //     &init,
    //     &goal,
    //     &vec!(),
    //     &Predicate::TRUE,
    //     &vec!()
    // );

    // (problem, blocks.iter().map(|x| x.to_string()).collect())
}
