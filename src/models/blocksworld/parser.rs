use super::*;

use std::io;
use std::io::prelude::*;
use std::fs::File;

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
        None => panic!("parsing object line failed")
    };

    let objects: Vec<&str> = objects_replaced.split(|c| c == ' ').collect();

    let init_replaced = match lines.next() {
        Some(x) => x.replace("(:INIT (", "").replace("))", ""),
        None => panic!("parsing INIT line failed")
    };

    let inits: Vec<&str> = init_replaced.split(") (").collect();

    let goal_replaced = match lines.next() {
        Some(x) => x.replace("(:goal (AND (", "").replace(")))", ""),
        None => panic!("parsing goal line failed")
    };

    let goal: Vec<&str> = goal_replaced.split(") (").collect();

    let blocks = objects;
    let clear_init = inits.iter().map(|x| x.split(" ").collect::<Vec<&str>>()).filter(|y| y[0] == "CLEAR").map(|z| z[1]).collect::<Vec<&str>>();
    let ontable_init = inits.iter().map(|x| x.split(" ").collect::<Vec<&str>>()).filter(|y| y[0] == "ONTABLE").map(|z| z[1]).collect::<Vec<&str>>();
    let hand_empty_init = match inits.iter().find(|y| *y == &"HANDEMPTY") {
        Some(_) => true,
        None => false
    };
    let on_init = inits.iter().map(|x| x.split(" ").collect::<Vec<&str>>()).filter(|y| y[0] == "ON").map(|z| (z[1], z[2])).collect::<Vec<(&str, &str)>>();
    let hand_empty_goal = match goal.iter().find(|y| *y == &"HANDEMPTY") {
        Some(_) => true,
        None => false
    };
    let on_goal = goal.iter().map(|x| x.split(" ").collect::<Vec<&str>>()).filter(|y| y[0] == "ON").map(|z| (z[1], z[2])).collect::<Vec<(&str, &str)>>();

    let model = domain::blocksworld_model(&blocks);

    println!("blocks: {:?}", blocks);
    println!("clear_init: {:?}", clear_init);
    println!("ontable_init: {:?}", ontable_init);
    println!("on_init: {:?}", on_init);
    println!("on_goal: {:?}", on_goal);

    setup::setup(
        model,
        &blocks,
        &clear_init,
        &ontable_init,
        hand_empty_init,
        &on_init,
        &on_goal,
    )
}