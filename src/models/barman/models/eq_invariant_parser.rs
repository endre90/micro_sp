use super::*;

use std::fs::File;
use std::io::prelude::*;
use std::collections::HashMap;

#[allow(dead_code)]
pub fn parser(name: &str) -> (ParamPlanningProblem, HashMap<String, Vec<String>>) {

    let mut instance = File::open(&format!("src/models/barman/instances/{}.pddl", name)).unwrap();
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

    let mut hands = vec!();
    let mut levels = vec!();
    let mut dispensers = vec!();
    let mut ingredients = vec!();
    let mut cocktails = vec!();
    let mut shots = vec!();
    let mut shakers = vec!();

    for elem in objects_strings {
        if elem.contains(" - hand") {
            hands = elem.split(|c| c == ' ').filter(|x| *x != "-" && *x != "hand" && *x != ":objects").map(|x| x.to_string()).collect();
        } else if elem.contains(" - level") {
            levels = elem.split(|c| c == ' ').filter(|x| *x != "-" && *x != "level" && *x != ":objects").map(|x| x.to_string()).collect();
        } else if elem.contains(" - dispenser") {
            dispensers = elem.split(|c| c == ' ').filter(|x| *x != "-" && *x != "dispenser" && *x != ":objects").map(|x| x.to_string()).collect();
        } else if elem.contains(" - ingredient") {
            ingredients = elem.split(|c| c == ' ').filter(|x| *x != "-" && *x != "ingredient" && *x != ":objects").map(|x| x.to_string()).collect();
        } else if elem.contains(" - cocktail") {
            cocktails = elem.split(|c| c == ' ').filter(|x| *x != "-" && *x != "cocktail" && *x != ":objects").map(|x| x.to_string()).collect();
        } else if elem.contains(" - shot") {
            shots = elem.split(|c| c == ' ').filter(|x| *x != "-" && *x != "shot" && *x != ":objects").map(|x| x.to_string()).collect();
        } else if elem.contains(" - shaker") {
            shakers = elem.split(|c| c == ' ').filter(|x| *x != "-" && *x != "shaker" && *x != ":objects").map(|x| x.to_string()).collect();
        }
    }

    let mut beverages = vec!();
    beverages.extend(ingredients.clone());
    beverages.extend(cocktails.clone());

    let mut containers = vec!();
    containers.extend(shots.clone());
    containers.extend(shakers.clone());

    let mut objects: HashMap<String, Vec<String>> = HashMap::new();
    objects.insert("hand".to_string(), hands);
    objects.insert("level".to_string(), levels);
    objects.insert("dispenser".to_string(), dispensers);
    objects.insert("ingredient".to_string(), ingredients.clone());
    objects.insert("cocktail".to_string(), cocktails);
    objects.insert("shot".to_string(), shots);
    objects.insert("shaker".to_string(), shakers);
    objects.insert("beverage".to_string(), beverages.clone());
    objects.insert("container".to_string(), containers);

    for o in &objects {
        println!("{:?}", o)
    }

    let pos_domain = vec!("left", "right", "table");
    
    let mut state_domain: Vec<&str> = vec!();
    let clean = vec!("clean");
    let empty = beverages.iter().map(|x| format!("empty_{}", x)).collect::<Vec<String>>();
    let contains = beverages.iter().map(|x| format!("contains_{}", x)).collect::<Vec<String>>();
    let mut contains_mix = vec!();
    for ingredient1 in &ingredients {
        for ingredient2 in &ingredients {
            contains_mix.push(
                format!("contains_{}_{}", ingredient1, ingredient2)
            )
        }
    }
    state_domain.extend(clean);
    state_domain.extend(empty.iter().map(|x| x.as_str()).collect::<Vec<&str>>());
    state_domain.extend(contains.iter().map(|x| x.as_str()).collect::<Vec<&str>>());
    state_domain.extend(contains_mix.iter().map(|x| x.as_str()).collect::<Vec<&str>>());

    println!("{:?}", state_domain);

    fn instance_1(name: &str, objects: HashMap<String, Vec<String>>, pos_domain: &Vec<&str>, state_domain: &Vec<&str>) -> (ParamPlanningProblem, HashMap<String, Vec<String>>) {

        let levels = objects.get("level").unwrap_or(&vec!()).to_vec();
        let ingredients = objects.get("ingredient").unwrap_or(&vec!()).to_vec();

        let initial = vec!(
            pass!(&new_enum_assign_c!("pos_shaker1", &pos_domain, "table", "pos", "c")),
            pass!(&new_enum_assign_c!("pos_shot1", &pos_domain, "table", "pos", "c")),
            pass!(&new_enum_assign_c!("dispenses_dispenser1", &ingredients, "ingredient1", "disp", "c")),
            pass!(&new_enum_assign_c!("state_shaker1", &state_domain, "clean", "state", "c")),
            pass!(&new_enum_assign_c!("state_shot1", &state_domain, "clean", "state", "c")),
            pass!(&new_enum_assign_c!("shaker_empty_level_shaker1", &levels ,"l0", "level", "c")),
            pass!(&new_enum_assign_c!("shaker_level_shaker1", &levels, "l0", "level", "c")),
            pass!(&new_enum_assign_c!("next_l0", &levels, "l1", "level", "c")),
            pass!(&new_enum_assign_c!("next_l1", &levels, "l2", "level", "c")),
            pass!(&new_enum_assign_c!("cocktail_part1", &ingredients, "ingredient1", "cocktail", "c")),
            pass!(&new_enum_assign_c!("cocktail_part2", &ingredients, "ingredient2", "cocktail", "c"))
        );
    
        let goal = vec!(
            pass!(&new_enum_assign_c!(&format!("state_shaker1"), &state_domain, &format!("contains_cocktail1"), "state", "c"))
        );

        let c = Parameter::new("c", &true);

        let problem = ParamPlanningProblem::new(
            name, 
            &ParamPredicate::new(&initial), 
            &ParamPredicate::new(&goal), 
            &vec!(), 
            &Predicate::TRUE,
            &vec!(c)
        );
    
        (problem, objects)
    }

    match name {
        "instance_1" => instance_1(name, objects, &pos_domain, &state_domain),
        // "instance_2" => instance_2(),
        _ => panic!("unknown instance")
    }
}