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
fn test_generate_and_solve_first_case() {
    let (problem, params) = models::dummy_robot::dummy_robot::param_model();
    let result = parameterized(&problem, &params, 1200);
    let new_result = generate_and_solve(
        &Case::First, 
        &State::empty(), 
        &problem, 
        &result, 
        &params,
        &0, 
        &0
    );
    pprint_result(&new_result)
}

#[test]
fn test_compositional() {
    let (problem, params) = models::dummy_robot::dummy_robot::param_model();
    let result = compositional(&problem, &params, 1200);
    pprint_result(&result)
}