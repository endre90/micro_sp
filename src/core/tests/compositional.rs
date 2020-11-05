use super::*;

#[test]
fn test_activate_next() {
    let p1 = Parameter::new("A", &true);
    let p2 = Parameter::new("B", &false);
    let p3 = Parameter::new("C", &false);
    let params = vec![p1, p2, p3];
    assert_eq!(&format!("{:?}", activate_next(&params)), 
        "[Parameter { name: \"A\", value: true }, Parameter { name: \"B\", value: true }, Parameter { name: \"C\", value: false }]");
}

#[test]
fn test_deactivate_all() {
    let p1 = Parameter::new("A", &true);
    let p2 = Parameter::new("B", &true);
    let p3 = Parameter::new("C", &false);
    let params = vec![p1, p2, p3];
    assert_eq!(&format!("{:?}", deactivate_all(&params)), 
        "[Parameter { name: \"A\", value: false }, Parameter { name: \"B\", value: false }, Parameter { name: \"C\", value: false }]");
}

#[test]
fn test_generate_solve_and_concatenate() {
    let (problem, params) = models::dummy_robot::dummy_robot::param_model();
    let result = parameterized(&problem, &params, 1200);
    println!("FIRST CASE");
    let first_case = generate_and_solve(
        &Case::First, 
        &State::empty(), 
        &problem, 
        &result, 
        &params,
        &0, 
        &0, 
        1200
    );
    println!("CENTRAL CASE 1");
    let central_1 = generate_and_solve(
        &Case::Central, 
        &first_case.trace[0].sink, 
        &problem, 
        &result, 
        &params,
        &0, 
        &1,
        1200
    );
    println!("CENTRAL CASE 2");
    let central_2 = generate_and_solve(
        &Case::Central, 
        &central_1.trace[0].sink, 
        &problem, 
        &result, 
        &params,
        &0, 
        &2,
        1200
    );
    println!("LAST CASE");
    let last_case = generate_and_solve(
        &Case::Last, 
        &central_2.trace[0].sink, 
        &problem, 
        &result, 
        &params,
        &0, 
        &2,
        1200
    );

    let conc = concatenate(&vec!(first_case, central_1, central_2, last_case));
    println!("CONCATENATED");
    pprint_result(&conc)
}

#[test]
fn test_compositional() {
    
    let p1 = Parameter::new("p1", &false);
    let p2 = Parameter::new("p2", &false);
    let dummy_params = vec!(p2, p1);

    let g_param = Parameter::new("g", &false);
    let r_param = Parameter::new("r", &false);
    let b_param = Parameter::new("b", &false);
    let b1 = Parameter::new("ball1", &false);
    let b2 = Parameter::new("ball2", &false);
    let b3 = Parameter::new("ball3", &false);
    let b4 = Parameter::new("ball4", &false);

    // let gripper_params = vec![b1, b2, b3, b4, g_param, r_param];
    let gripper_params = vec![g_param, b_param, r_param];

    // let (problem, _params) = models::dummy_robot::dummy_robot::param_model();
    let problem = models::gripper::parser::parser_model_pure_booleans("instance-2");
    // let problem = models::gripper::parser::parser_model_pure_booleans_2("instance-1");
    // let result = compositional(&problem, &dummy_params); //, 1200);
    let result = compositional(&problem, &gripper_params); //, 1200);
    pprint_result_trans_only(&result)
}