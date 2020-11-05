use super::*;

use std::fs::File;
use std::io;
use std::io::prelude::*;

#[test]
fn test_parser_model_pure_booleans() {
    let model = parser_model_pure_booleans("instance-2");

    let g_param = Parameter::new("g", &false);
    let r_param = Parameter::new("r", &false);
    let b_param = Parameter::new("b", &true);
    let none = Parameter::new("NONE", &true); 

    let params = vec![g_param, r_param, b_param, none];

    let result = parameterized(&model, &params, 1200);
    pprint_result_trans_only(&result)
}

#[test]
fn test_parser_model_enumerated_booleans() {
    let model = parser_model_enumerated_booleans("instance-1");

    let g_param = Parameter::new("g", &true);
    let r_param = Parameter::new("r", &true);
    let b_param = Parameter::new("b", &true);
    let none = Parameter::new("NONE", &true); 

    let params = vec![b_param, r_param, g_param, none];

    let result = parameterized(&model.0, &params, 1200);
    pprint_result_trans_only(&result)
}

pub fn parser_model_enumerated_booleans(name: &str) -> (ParamPlanningProblem, Vec<Parameter>) {
    let g_param = Parameter::new("g", &true);
    let r_param = Parameter::new("r", &false);
    let b_param = Parameter::new("b", &true);
    // let none = Parameter::new("NONE", &true); 

    let params = vec![b_param, g_param, r_param];
    let mut domain = File::open(&format!("src/models/gripper/instances/domain.pddl")).unwrap();
    let mut instance = File::open(&format!("src/models/gripper/instances/{}.pddl", name)).unwrap();
    let mut i_buffer = String::new();
    let mut d_buffer = String::new();

    domain.read_to_string(&mut d_buffer).unwrap();
    instance.read_to_string(&mut i_buffer).unwrap();

    let d_parts: Vec<&str> = d_buffer.split("(:").collect();
    let i_parts: Vec<&str> = i_buffer.split("(:").collect();

    let mut pred_untrimmed_str = "";

    let mut obj_untrimmed_str = "";
    let mut init_untrimmed_str = "";
    let mut goal_untrimmed_str = "";

    for p in d_parts {
        if p.starts_with("predicates") {
            pred_untrimmed_str = p;
        }
    }

    for p in i_parts {
        if p.starts_with("obj") {
            obj_untrimmed_str = p;
        } else if p.starts_with("init") {
            init_untrimmed_str = p;
        } else if p.starts_with("goal") {
            goal_untrimmed_str = p;
        }
    }

    let pred_trimmed_str = pred_untrimmed_str
        .replace("))\n\n   ", "")
        .replace("predicates (", "");
    let pred_vec: Vec<&str> = pred_trimmed_str.split(")\n\t\t(").collect();

    let obj_trimmed_str = obj_untrimmed_str
        .replace(")\n   ", "")
        .replace("objects ", "");
    let obj_vec_init: Vec<&str> = obj_trimmed_str.split(")\n          (").collect();
    let obj_vec: Vec<&str> = obj_vec_init[0].split(" ").collect();

    let init_trimmed_str = init_untrimmed_str
        .replace("))\n   ", "")
        .replace("init (", "");
    let init_vec_init: Vec<&str> = init_trimmed_str.split(")\n          (").collect();
    let items = vec!("ball ", "room ", "gripper ");

    let mut init_vec = vec!();
    for i in init_vec_init {
        if !(i.contains("ball ")  || i.contains("room ") || i.contains("gripper ")) {
            init_vec.push(i)
        }
    }

    let goal_trimmed_str = goal_untrimmed_str
        .replace("))))", "")
        .replace("goal (and (", "");
    let goal_vec: Vec<&str> = goal_trimmed_str.split(")\n               (").collect();

    let rooms = obj_vec
        .iter()
        .filter(|x| x.contains("room"))
        .map(|x| x.to_owned())
        .collect();
    let balls = obj_vec
        .iter()
        .filter(|x| x.contains("ball"))
        .map(|x| x.to_owned())
        .collect();
    let grippers = vec!["left", "right"];

    let model = domain::gripper_model_enumerated_booleans(&rooms, &grippers, &balls);
    let vars = get_param_model_vars(&model.0);

    println!("{:?}", rooms);
    println!("{:?}", balls);

    let init_vec_fin_pos: Vec<String> = init_vec.iter().map(|c| c.replace(" ", "_")).collect();
    let goal_vec_fin_pos: Vec<String> = goal_vec.iter().map(|c| c.replace(" ", "_")).collect();

    let init_vec_final_positive: Vec<&str> = init_vec_fin_pos.iter().map(|x| x.as_str()).collect();
    let goal_vec_final_positive: Vec<&str> = goal_vec_fin_pos.iter().map(|x| x.as_str()).collect();

    let instance_init_variables: Vec<Variable> = vars.iter().filter(|x| Some(x.name.clone()) == init_vec_final_positive.iter().find(|y| **y == x.name).map(|x| x.to_owned().to_owned())).map(|x| x.to_owned()).collect();
    let instance_goal_variables: Vec<Variable> = vars.iter().filter(|x| Some(x.name.clone()) == goal_vec_final_positive.iter().find(|y| **y == x.name).map(|x| x.to_owned().to_owned())).map(|x| x.to_owned()).collect();

    // let instance_init_variables: Vec<Variable> = init_vec_final_positive
    //     .iter()
    //     .map(|x| enum_c!(x, "boolean", vec!("true", "false")))
    //     .collect();

        println!("VARS");
        for v in &vars {
            println!("{:?}", v);
        }
        println!("===================================");
    
        let instance_init_neg_variables: Vec<Variable> =
            vars.difference(instance_init_variables.clone());
    
        println!("POSITIVE");
        for v in &instance_init_variables {
            println!("{:?}", v);
        }
        println!("===================================");
    
        println!("NEGATIVE");
        for v in &instance_init_neg_variables {
            println!("{:?}", v);
        }
        println!("===================================");

    let init_pred_list: Vec<Predicate> = instance_init_variables
        .iter()
        .map(|x| pass!(&enum_assign!(&x, "true"))) // Predicate::ASS(Assignment::new(&x, "true", None)))
        .collect();

    let init_neg_pred_list: Vec<Predicate> = instance_init_neg_variables
        .iter()
        .map(|x| pass!(&enum_assign!(&x, "false"))) // Predicate::ASS(Assignment::new(&x, "false", None)))
        .collect();

    let goal_pred_list: Vec<Predicate> = instance_goal_variables
        .iter()
        .map(|x| pass!(&enum_assign!(&x, "true"))) // Predicate::ASS(Assignment::new(&x, "true", None)))
        .collect();

    // let goal_pred_list = goal_vec_final_positive
    //     .iter()
    //     .map(|x| {
    //         pass!(new_enum_assign_c!(x, "boolean" , vec!("true", "false"), "true")) //&enum_c!(x, "boolean", vec!("true", "false")))
    //     })
    //     .collect();

    let mut init_list = vec![];
    for i in vec![init_pred_list, init_neg_pred_list] {
        init_list.extend(i)
    }

    let init = ParamPredicate::new(&init_list);
    let goal = ParamPredicate::new(&goal_pred_list);

    let problem = ParamPlanningProblem::new("gripper", &init, &goal, &model.0, &model.1, &20);

    (problem, params)
}



pub fn parser_model_pure_booleans(name: &str) -> ParamPlanningProblem {
    let mut domain = File::open(&format!("src/models/gripper/instances/domain.pddl")).unwrap();
    let mut instance = File::open(&format!("src/models/gripper/instances/{}.pddl", name)).unwrap();
    let mut i_buffer = String::new();
    let mut d_buffer = String::new();

    domain.read_to_string(&mut d_buffer).unwrap();
    instance.read_to_string(&mut i_buffer).unwrap();

    let d_parts: Vec<&str> = d_buffer.split("(:").collect();
    let i_parts: Vec<&str> = i_buffer.split("(:").collect();

    let mut pred_untrimmed_str = "";

    let mut obj_untrimmed_str = "";
    let mut init_untrimmed_str = "";
    let mut goal_untrimmed_str = "";

    for p in d_parts {
        if p.starts_with("predicates") {
            pred_untrimmed_str = p;
        }
    }

    for p in i_parts {
        if p.starts_with("obj") {
            obj_untrimmed_str = p;
        } else if p.starts_with("init") {
            init_untrimmed_str = p;
        } else if p.starts_with("goal") {
            goal_untrimmed_str = p;
        }
    }

    let pred_trimmed_str = pred_untrimmed_str
        .replace("))\n\n   ", "")
        .replace("predicates (", "");
    let pred_vec: Vec<&str> = pred_trimmed_str.split(")\n\t\t(").collect();

    let obj_trimmed_str = obj_untrimmed_str
        .replace(")\n   ", "")
        .replace("objects ", "");
    let obj_vec_init: Vec<&str> = obj_trimmed_str.split(")\n          (").collect();
    let obj_vec: Vec<&str> = obj_vec_init[0].split(" ").collect();

    let init_trimmed_str = init_untrimmed_str
        .replace("))\n   ", "")
        .replace("init (", "");
    let init_vec_init: Vec<&str> = init_trimmed_str.split(")\n          (").collect();
    let items = vec!("ball ", "room ", "gripper ");

    let mut init_vec = vec!();
    for i in init_vec_init {
        if !(i.contains("ball ")  || i.contains("room ") || i.contains("gripper ")) {
            init_vec.push(i)
        }
    }

    let goal_trimmed_str = goal_untrimmed_str
        .replace("))))", "")
        .replace("goal (and (", "");
    let goal_vec: Vec<&str> = goal_trimmed_str.split(")\n               (").collect();

    let rooms = obj_vec
        .iter()
        .filter(|x| x.contains("room"))
        .map(|x| x.to_owned())
        .collect();
    let balls = obj_vec
        .iter()
        .filter(|x| x.contains("ball"))
        .map(|x| x.to_owned())
        .collect();
    let grippers = vec!["left", "right"];

    let model = domain::gripper_model_pure_booleans(&rooms, &grippers, &balls);
    let vars = get_param_model_vars(&model.0);

    println!("{:?}", rooms);
    println!("{:?}", balls);

    let init_vec_fin_pos: Vec<String> = init_vec.iter().map(|c| c.replace(" ", "_")).collect();
    let goal_vec_fin_pos: Vec<String> = goal_vec.iter().map(|c| c.replace(" ", "_")).collect();

    let init_vec_final_positive: Vec<&str> = init_vec_fin_pos.iter().map(|x| x.as_str()).collect();
    let goal_vec_final_positive: Vec<&str> = goal_vec_fin_pos.iter().map(|x| x.as_str()).collect();

    let instance_init_variables: Vec<Variable> = vars.iter().filter(|x| Some(x.name.clone()) == init_vec_final_positive.iter().find(|y| **y == x.name).map(|x| x.to_owned().to_owned())).map(|x| x.to_owned()).collect();
    let instance_goal_variables: Vec<Variable> = vars.iter().filter(|x| Some(x.name.clone()) == goal_vec_final_positive.iter().find(|y| **y == x.name).map(|x| x.to_owned().to_owned())).map(|x| x.to_owned()).collect();

    // let instance_init_variables: Vec<Variable> = init_vec_final_positive
    //     .iter()
    //     .map(|x| enum_c!(x, "boolean", vec!("true", "false")))
    //     .collect();

        println!("VARS");
        for v in &vars {
            println!("{:?}", v);
        }
        println!("===================================");
    
        let instance_init_neg_variables: Vec<Variable> =
            vars.difference(instance_init_variables.clone());
    
        println!("POSITIVE");
        for v in &instance_init_variables {
            println!("{:?}", v);
        }
        println!("===================================");
    
        println!("NEGATIVE");
        for v in &instance_init_neg_variables {
            println!("{:?}", v);
        }
        println!("===================================");

    let init_pred_list: Vec<Predicate> = instance_init_variables
        .iter()
        .map(|x| pass!(&bool_assign!(&x, &true))) // Predicate::ASS(Assignment::new(&x, "true", None)))
        .collect();

    let init_neg_pred_list: Vec<Predicate> = instance_init_neg_variables
        .iter()
        .map(|x| pass!(&bool_assign!(&x, &false))) // Predicate::ASS(Assignment::new(&x, "false", None)))
        .collect();

    let goal_pred_list: Vec<Predicate> = instance_goal_variables
        .iter()
        .map(|x| pass!(&bool_assign!(&x, &true))) // Predicate::ASS(Assignment::new(&x, "true", None)))
        .collect();

    // let goal_pred_list = goal_vec_final_positive
    //     .iter()
    //     .map(|x| {
    //         pass!(new_enum_assign_c!(x, "boolean" , vec!("true", "false"), "true")) //&enum_c!(x, "boolean", vec!("true", "false")))
    //     })
    //     .collect();

    let mut init_list = vec![];
    for i in vec![init_pred_list, init_neg_pred_list] {
        init_list.extend(i)
    }

    let init = ParamPredicate::new(&init_list);
    let goal = ParamPredicate::new(&goal_pred_list);

    let problem = ParamPlanningProblem::new("gripper", &init, &goal, &model.0, &model.1, &20);

    problem
}


pub fn parser_model_pure_booleans_2(name: &str) -> ParamPlanningProblem {
    let mut domain = File::open(&format!("src/models/gripper/instances/domain.pddl")).unwrap();
    let mut instance = File::open(&format!("src/models/gripper/instances/{}.pddl", name)).unwrap();
    let mut i_buffer = String::new();
    let mut d_buffer = String::new();

    domain.read_to_string(&mut d_buffer).unwrap();
    instance.read_to_string(&mut i_buffer).unwrap();

    let d_parts: Vec<&str> = d_buffer.split("(:").collect();
    let i_parts: Vec<&str> = i_buffer.split("(:").collect();

    let mut pred_untrimmed_str = "";

    let mut obj_untrimmed_str = "";
    let mut init_untrimmed_str = "";
    let mut goal_untrimmed_str = "";

    for p in d_parts {
        if p.starts_with("predicates") {
            pred_untrimmed_str = p;
        }
    }

    for p in i_parts {
        if p.starts_with("obj") {
            obj_untrimmed_str = p;
        } else if p.starts_with("init") {
            init_untrimmed_str = p;
        } else if p.starts_with("goal") {
            goal_untrimmed_str = p;
        }
    }

    let pred_trimmed_str = pred_untrimmed_str
        .replace("))\n\n   ", "")
        .replace("predicates (", "");
    let pred_vec: Vec<&str> = pred_trimmed_str.split(")\n\t\t(").collect();

    let obj_trimmed_str = obj_untrimmed_str
        .replace(")\n   ", "")
        .replace("objects ", "");
    let obj_vec_init: Vec<&str> = obj_trimmed_str.split(")\n          (").collect();
    let obj_vec: Vec<&str> = obj_vec_init[0].split(" ").collect();

    let init_trimmed_str = init_untrimmed_str
        .replace("))\n   ", "")
        .replace("init (", "");
    let init_vec_init: Vec<&str> = init_trimmed_str.split(")\n          (").collect();
    let items = vec!("ball ", "room ", "gripper ");

    let mut init_vec = vec!();
    for i in init_vec_init {
        if !(i.contains("ball ")  || i.contains("room ") || i.contains("gripper ")) {
            init_vec.push(i)
        }
    }

    let goal_trimmed_str = goal_untrimmed_str
        .replace("))))", "")
        .replace("goal (and (", "");
    let goal_vec: Vec<&str> = goal_trimmed_str.split(")\n               (").collect();

    let rooms = obj_vec
        .iter()
        .filter(|x| x.contains("room"))
        .map(|x| x.to_owned())
        .collect();
    let balls = obj_vec
        .iter()
        .filter(|x| x.contains("ball"))
        .map(|x| x.to_owned())
        .collect();
    let grippers = vec!["left", "right"];

    let model = domain::gripper_model_pure_booleans_2(&rooms, &grippers, &balls);
    let vars = get_param_model_vars(&model.0);

    println!("{:?}", rooms);
    println!("{:?}", balls);

    let init_vec_fin_pos: Vec<String> = init_vec.iter().map(|c| c.replace(" ", "_")).collect();
    let goal_vec_fin_pos: Vec<String> = goal_vec.iter().map(|c| c.replace(" ", "_")).collect();

    let init_vec_final_positive: Vec<&str> = init_vec_fin_pos.iter().map(|x| x.as_str()).collect();
    let goal_vec_final_positive: Vec<&str> = goal_vec_fin_pos.iter().map(|x| x.as_str()).collect();

    let instance_init_variables: Vec<Variable> = vars.iter().filter(|x| Some(x.name.clone()) == init_vec_final_positive.iter().find(|y| **y == x.name).map(|x| x.to_owned().to_owned())).map(|x| x.to_owned()).collect();
    let instance_goal_variables: Vec<Variable> = vars.iter().filter(|x| Some(x.name.clone()) == goal_vec_final_positive.iter().find(|y| **y == x.name).map(|x| x.to_owned().to_owned())).map(|x| x.to_owned()).collect();

    // let instance_init_variables: Vec<Variable> = init_vec_final_positive
    //     .iter()
    //     .map(|x| enum_c!(x, "boolean", vec!("true", "false")))
    //     .collect();

        println!("VARS");
        for v in &vars {
            println!("{:?}", v);
        }
        println!("===================================");
    
        let instance_init_neg_variables: Vec<Variable> =
            vars.difference(instance_init_variables.clone());
    
        println!("POSITIVE");
        for v in &instance_init_variables {
            println!("{:?}", v);
        }
        println!("===================================");
    
        println!("NEGATIVE");
        for v in &instance_init_neg_variables {
            println!("{:?}", v);
        }
        println!("===================================");

    let init_pred_list: Vec<Predicate> = instance_init_variables
        .iter()
        .map(|x| pass!(&bool_assign!(&x, &true))) // Predicate::ASS(Assignment::new(&x, "true", None)))
        .collect();

    let init_neg_pred_list: Vec<Predicate> = instance_init_neg_variables
        .iter()
        .map(|x| pass!(&bool_assign!(&x, &false))) // Predicate::ASS(Assignment::new(&x, "false", None)))
        .collect();

    let goal_pred_list: Vec<Predicate> = instance_goal_variables
        .iter()
        .map(|x| pass!(&bool_assign!(&x, &true))) // Predicate::ASS(Assignment::new(&x, "true", None)))
        .collect();

    // let goal_pred_list = goal_vec_final_positive
    //     .iter()
    //     .map(|x| {
    //         pass!(new_enum_assign_c!(x, "boolean" , vec!("true", "false"), "true")) //&enum_c!(x, "boolean", vec!("true", "false")))
    //     })
    //     .collect();

    let mut init_list = vec![];
    for i in vec![init_pred_list, init_neg_pred_list] {
        init_list.extend(i)
    }

    let init = ParamPredicate::new(&init_list);
    let goal = ParamPredicate::new(&goal_pred_list);

    let problem = ParamPlanningProblem::new("gripper", &init, &goal, &model.0, &model.1, &20);

    problem
}
