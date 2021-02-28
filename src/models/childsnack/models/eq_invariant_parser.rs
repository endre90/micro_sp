use super::*;

use std::fs::File;
use std::io::prelude::*;
use std::collections::HashMap;

#[allow(dead_code)]
pub fn parser(name: &str) -> (ParamPlanningProblem, HashMap<String, Vec<String>>) {
    let mut instance = File::open(&format!("src/models/childsnack/instances/{}.pddl", name)).unwrap();
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

    let mut children = vec!();
    let mut bread_portions = vec!();
    let mut content_portions = vec!();
    let mut trays = vec!();
    let mut places = vec!();
    let mut sandwiches = vec!();

    for elem in objects_strings {
        if elem.contains(" - child") {
            children = elem.split(|c| c == ' ').filter(|x| *x != "-" && *x != "child" && *x != ":objects").map(|x| x.to_string()).collect();
        } else if elem.contains(" - bread-portion") {
            bread_portions = elem.split(|c| c == ' ').filter(|x| *x != "-" && *x != "bread-portion" && *x != ":objects").map(|x| x.to_string()).collect();
        } else if elem.contains(" - content-portion") {
            content_portions = elem.split(|c| c == ' ').filter(|x| *x != "-" && *x != "content-portion" && *x != ":objects").map(|x| x.to_string()).collect();
        } else if elem.contains(" - tray") {
            trays = elem.split(|c| c == ' ').filter(|x| *x != "-" && *x != "tray" && *x != ":objects").map(|x| x.to_string()).collect();
        } else if elem.contains(" - place") {
            places = elem.split(|c| c == ' ').filter(|x| *x != "-" && *x != "place" && *x != ":objects").map(|x| x.to_string()).collect();
        } else if elem.contains(" - sandwich") {
            sandwiches = elem.split(|c| c == ' ').filter(|x| *x != "-" && *x != "sandwich" && *x != ":objects").map(|x| x.to_string()).collect();
        }
    }

    let mut objects: HashMap<String, Vec<String>> = HashMap::new();
    objects.insert("child".to_string(), children);
    objects.insert("bread_portion".to_string(), bread_portions);
    objects.insert("content_portion".to_string(), content_portions);
    objects.insert("tray".to_string(), trays.clone());
    objects.insert("place".to_string(), places.clone());
    objects.insert("sandwich".to_string(), sandwiches);

    let s_domain1 = vec!("notexist", "served", "kitchen");
    let t_domain1 = vec!("kitchen");
    let c_domain1 = vec!("served");
    let tf_domain = vec!("true", "false");

    let mut sandwich_domain: Vec<&str> = vec!();
    sandwich_domain.extend(s_domain1);
    sandwich_domain.extend(trays.iter().map(|x| x.as_str()).collect::<Vec<&str>>());

    let mut tray_domain = vec!();
    tray_domain.extend(t_domain1);
    tray_domain.extend(places.iter().map(|x| x.as_str()).collect::<Vec<&str>>());

    let mut child_domain = vec!();
    child_domain.extend(c_domain1);
    child_domain.extend(places.iter().map(|x| x.as_str()).collect::<Vec<&str>>());
    let index = child_domain.iter().position(|x| *x == "kitchen").unwrap();
    child_domain.remove(index);

    objects.insert("sandwich_domain".to_string(), sandwich_domain.iter().map(|x| String::from(*x)).collect());
    objects.insert("tray_domain".to_string(), tray_domain.iter().map(|x| String::from(*x)).collect());
    objects.insert("child_domain".to_string(), child_domain.iter().map(|x| String::from(*x)).collect());
    objects.insert("tf_domain".to_string(), tf_domain.iter().map(|x| String::from(*x)).collect());

    for o in &objects {
        println!("{:?}", o)
    }

    let (initial, goal) = match name {
        "instance_10" => (
            vec!(
                pass!(&new_enum_assign_c!("tray1", &places, "kitchen", "places", "c")),
                pass!(&new_enum_assign_c!("at_kitchen_bread_bread1", &tf_domain, "true", "tf", "c")),
                pass!(&new_enum_assign_c!("at_kitchen_content_content1", &tf_domain, "true", "tf", "c")),
                pass!(&new_enum_assign_c!("at_kitchen_bread_bread2", &tf_domain, "true", "tf", "c")),
                pass!(&new_enum_assign_c!("at_kitchen_content_content2", &tf_domain, "true", "tf", "c")),
                pass!(&new_enum_assign_c!("at_kitchen_bread_bread3", &tf_domain, "true", "tf", "c")),
                pass!(&new_enum_assign_c!("at_kitchen_content_content3", &tf_domain, "true", "tf", "c")),
                pass!(&new_enum_assign_c!("no_gluten_bread_bread1", &tf_domain, "true", "tf", "c")),
                pass!(&new_enum_assign_c!("no_gluten_content_content1", &tf_domain, "true", "tf", "c")),
                pass!(&new_enum_assign_c!("no_gluten_bread_bread3", &tf_domain, "true", "tf", "c")),
                pass!(&new_enum_assign_c!("no_gluten_content_content3", &tf_domain, "true", "tf", "c")),
                pass!(&new_enum_assign_c!("allergic_gluten_child1", &tf_domain, "true", "tf", "c")),
                pass!(&new_enum_assign_c!("allergic_gluten_child2", &tf_domain, "true", "tf", "c")),
                pass!(&new_enum_assign_c!("allergic_gluten_child3", &tf_domain, "false", "tf", "c")),
                pass!(&new_enum_assign_c!("child1", &child_domain, "table1", "child", "c")),
                pass!(&new_enum_assign_c!("child2", &child_domain, "table2", "child", "c")),
                pass!(&new_enum_assign_c!("child3", &child_domain, "table3", "child", "c")),
                pass!(&new_enum_assign_c!("sandwich1", &sandwich_domain, "notexist", "sandwich", "c")),
                pass!(&new_enum_assign_c!("sandwich2", &sandwich_domain, "notexist", "sandwich", "c")),
                pass!(&new_enum_assign_c!("sandwich3", &sandwich_domain, "notexist", "sandwich", "c"))
            ),
            vec!(
                pass!(&new_enum_assign_c!("child1", &child_domain, "served", "child", "c")),
                pass!(&new_enum_assign_c!("child2", &child_domain, "served", "child", "c")),
                pass!(&new_enum_assign_c!("child3", &child_domain, "served", "child", "c"))
            )
        ),
        "instance_1" => (
            vec!(
                pass!(&new_enum_assign_c!("tray1", &tray_domain, "kitchen", "tray", "c")),
                pass!(&new_enum_assign_c!("at_kitchen_bread_bread1", &tf_domain, "true", "tf", "c")),
                pass!(&new_enum_assign_c!("at_kitchen_content_content1", &tf_domain, "true", "tf", "c")),
                pass!(&new_enum_assign_c!("at_kitchen_content_content2", &tf_domain, "true", "tf", "c")),
                pass!(&new_enum_assign_c!("no_gluten_bread_bread1", &tf_domain, "true", "tf", "c")),
                pass!(&new_enum_assign_c!("no_gluten_content_content1", &tf_domain, "true", "tf", "c")),
                pass!(&new_enum_assign_c!("allergic_gluten_child1", &tf_domain, "true", "tf", "c")),
                pass!(&new_enum_assign_c!("child1", &child_domain, "table1", "child", "c")),
                pass!(&new_enum_assign_c!("sandwich1", &sandwich_domain, "notexist", "sandwich", "c"))
            ),
            vec!(
                pass!(&new_enum_assign_c!("child1", &child_domain, "served", "child", "c"))
            )
        ),
        "instance_2" => (
            vec!(
                pass!(&new_enum_assign_c!("tray1", &tray_domain, "kitchen", "tray", "c")),
                // pass!(&new_enum_assign_c!("tray2", &tray_domain, "kitchen", "tray", "c")),
                pass!(&new_enum_assign_c!("at_kitchen_bread_bread1", &tf_domain, "true", "tf", "c")),
                pass!(&new_enum_assign_c!("at_kitchen_bread_bread2", &tf_domain, "true", "tf", "c")),
                pass!(&new_enum_assign_c!("at_kitchen_content_content1", &tf_domain, "true", "tf", "c")),
                pass!(&new_enum_assign_c!("at_kitchen_content_content2", &tf_domain, "true", "tf", "c")),
                pass!(&new_enum_assign_c!("no_gluten_bread_bread1", &tf_domain, "true", "tf", "c")),
                // pass!(&new_enum_assign_c!("no_gluten_bread_bread2", &tf_domain, "true", "tf", "c")),
                pass!(&new_enum_assign_c!("no_gluten_content_content1", &tf_domain, "true", "tf", "c")),
                // pass!(&new_enum_assign_c!("no_gluten_content_content2", &tf_domain, "true", "tf", "c")),
                pass!(&new_enum_assign_c!("allergic_gluten_child1", &tf_domain, "true", "tf", "c")),
                // pass!(&new_enum_assign_c!("allergic_gluten_child2", &tf_domain, "true", "tf", "c")),
                pass!(&new_enum_assign_c!("child1", &child_domain, "table1", "child", "c")),
                // pass!(&new_enum_assign_c!("child2", &child_domain, "table2", "child", "c")),
                pass!(&new_enum_assign_c!("sandwich1", &sandwich_domain, "notexist", "sandwich", "c"))
                // pass!(&new_enum_assign_c!("sandwich2", &sandwich_domain, "notexist", "sandwich", "c"))
            ),
            vec!(
                pass!(&new_enum_assign_c!("child1", &child_domain, "served", "child", "c"))
                // pass!(&new_enum_assign_c!("child2", &child_domain, "served", "child", "c"))
            )
        ),
        "instance_3" => (
            vec!(
                pass!(&new_enum_assign_c!("tray1", &tray_domain, "kitchen", "tray", "c")),
                pass!(&new_enum_assign_c!("tray2", &tray_domain, "kitchen", "tray", "c")),
                pass!(&new_enum_assign_c!("tray3", &tray_domain, "kitchen", "tray", "c")),
                pass!(&new_enum_assign_c!("at_kitchen_bread_bread1", &tf_domain, "true", "tf", "c")),
                pass!(&new_enum_assign_c!("at_kitchen_bread_bread2", &tf_domain, "true", "tf", "c")),
                pass!(&new_enum_assign_c!("at_kitchen_bread_bread3", &tf_domain, "true", "tf", "c")),
                pass!(&new_enum_assign_c!("at_kitchen_content_content1", &tf_domain, "true", "tf", "c")),
                pass!(&new_enum_assign_c!("at_kitchen_content_content2", &tf_domain, "true", "tf", "c")),
                pass!(&new_enum_assign_c!("at_kitchen_content_content3", &tf_domain, "true", "tf", "c")),
                pass!(&new_enum_assign_c!("no_gluten_bread_bread2", &tf_domain, "true", "tf", "c")),
                pass!(&new_enum_assign_c!("no_gluten_content_content2", &tf_domain, "true", "tf", "c")),
                pass!(&new_enum_assign_c!("no_gluten_bread_bread1", &tf_domain, "false", "tf", "c")),
                pass!(&new_enum_assign_c!("no_gluten_content_content1", &tf_domain, "false", "tf", "c")),
                pass!(&new_enum_assign_c!("no_gluten_bread_bread3", &tf_domain, "false", "tf", "c")),
                pass!(&new_enum_assign_c!("no_gluten_content_content3", &tf_domain, "false", "tf", "c")),
                pass!(&new_enum_assign_c!("allergic_gluten_child1", &tf_domain, "false", "tf", "c")),
                pass!(&new_enum_assign_c!("allergic_gluten_child2", &tf_domain, "true", "tf", "c")),
                pass!(&new_enum_assign_c!("allergic_gluten_child3", &tf_domain, "false", "tf", "c")),
                pass!(&new_enum_assign_c!("child1", &child_domain, "table1", "child", "c")),
                pass!(&new_enum_assign_c!("child2", &child_domain, "table2", "child", "c")),
                pass!(&new_enum_assign_c!("child3", &child_domain, "table1", "child", "c")),
                pass!(&new_enum_assign_c!("sandwich1", &sandwich_domain, "notexist", "sandwich", "c")),
                pass!(&new_enum_assign_c!("sandwich2", &sandwich_domain, "notexist", "sandwich", "c")),
                pass!(&new_enum_assign_c!("sandwich3", &sandwich_domain, "notexist", "sandwich", "c"))
            ),
            vec!(
                pass!(&new_enum_assign_c!("child1", &child_domain, "served", "child", "c")),
                pass!(&new_enum_assign_c!("child2", &child_domain, "served", "child", "c")),
                pass!(&new_enum_assign_c!("child3", &child_domain, "served", "child", "c"))
            )
        ),
        "instance_4" => (
            vec!(
                pass!(&new_enum_assign_c!("tray1", &tray_domain, "kitchen", "tray", "c")),
                pass!(&new_enum_assign_c!("tray2", &tray_domain, "kitchen", "tray", "c")),
                pass!(&new_enum_assign_c!("tray3", &tray_domain, "kitchen", "tray", "c")),
                pass!(&new_enum_assign_c!("at_kitchen_bread_bread1", &tf_domain, "true", "tf", "c")),
                pass!(&new_enum_assign_c!("at_kitchen_bread_bread2", &tf_domain, "true", "tf", "c")),
                pass!(&new_enum_assign_c!("at_kitchen_bread_bread3", &tf_domain, "true", "tf", "c")),
                pass!(&new_enum_assign_c!("at_kitchen_bread_bread4", &tf_domain, "true", "tf", "c")),
                pass!(&new_enum_assign_c!("at_kitchen_content_content1", &tf_domain, "true", "tf", "c")),
                pass!(&new_enum_assign_c!("at_kitchen_content_content2", &tf_domain, "true", "tf", "c")),
                pass!(&new_enum_assign_c!("at_kitchen_content_content3", &tf_domain, "true", "tf", "c")),
                pass!(&new_enum_assign_c!("at_kitchen_content_content4", &tf_domain, "true", "tf", "c")),
                pass!(&new_enum_assign_c!("no_gluten_bread_bread2", &tf_domain, "true", "tf", "c")),
                pass!(&new_enum_assign_c!("no_gluten_content_content2", &tf_domain, "true", "tf", "c")),
                pass!(&new_enum_assign_c!("no_gluten_bread_bread4", &tf_domain, "true", "tf", "c")),
                pass!(&new_enum_assign_c!("no_gluten_content_content4", &tf_domain, "true", "tf", "c")),
                pass!(&new_enum_assign_c!("no_gluten_bread_bread1", &tf_domain, "false", "tf", "c")),
                pass!(&new_enum_assign_c!("no_gluten_content_content1", &tf_domain, "false", "tf", "c")),
                pass!(&new_enum_assign_c!("no_gluten_bread_bread3", &tf_domain, "false", "tf", "c")),
                pass!(&new_enum_assign_c!("no_gluten_content_content3", &tf_domain, "false", "tf", "c")),
                pass!(&new_enum_assign_c!("allergic_gluten_child1", &tf_domain, "false", "tf", "c")),
                pass!(&new_enum_assign_c!("allergic_gluten_child2", &tf_domain, "true", "tf", "c")),
                pass!(&new_enum_assign_c!("allergic_gluten_child3", &tf_domain, "false", "tf", "c")),
                pass!(&new_enum_assign_c!("allergic_gluten_child4", &tf_domain, "true", "tf", "c")),
                pass!(&new_enum_assign_c!("child1", &child_domain, "table1", "child", "c")),
                pass!(&new_enum_assign_c!("child2", &child_domain, "table2", "child", "c")),
                pass!(&new_enum_assign_c!("child3", &child_domain, "table1", "child", "c")),
                pass!(&new_enum_assign_c!("child4", &child_domain, "table2", "child", "c")),
                pass!(&new_enum_assign_c!("sandwich1", &sandwich_domain, "notexist", "sandwich", "c")),
                pass!(&new_enum_assign_c!("sandwich2", &sandwich_domain, "notexist", "sandwich", "c")),
                pass!(&new_enum_assign_c!("sandwich3", &sandwich_domain, "notexist", "sandwich", "c")),
                pass!(&new_enum_assign_c!("sandwich4", &sandwich_domain, "notexist", "sandwich", "c"))
            ),
            vec!(
                pass!(&new_enum_assign_c!("child1", &child_domain, "served", "child", "c")),
                pass!(&new_enum_assign_c!("child2", &child_domain, "served", "child", "c")),
                pass!(&new_enum_assign_c!("child3", &child_domain, "served", "child", "c")),
                pass!(&new_enum_assign_c!("child4", &child_domain, "served", "child", "c"))
            )
        ),
        _ => panic!("no such instance")
    };

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