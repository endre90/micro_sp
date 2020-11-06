use super::*;

#[test]
fn test_new_parameter() {
    assert_eq!(
        Parameter::new("some_name", &false),
        Parameter {
            name: String::from("some_name"),
            value: false
        }
    )
}

#[test]
fn test_none_parameter() {
    assert_eq!(
        Parameter::none(),
        Parameter {
            name: String::from("NONE"),
            value: true
        }
    )
}

#[test]
fn test_new_param_predicate() {
    let pp = ppred!(
        &pass!(&new_enum_assign_m!("var1_m", vec!("a", "b", "c"), "a", "t1", "p1")),
        &pass!(&new_enum_assign_m!("var1_c", vec!("a", "b", "c"), "b", "t1", "p1"))
    );

    println!("{:?}", pp);
}

#[test]
fn test_generate_predicate() {
    let p1 = Parameter::new("p1", &true);
    let p2 = Parameter::new("p2", &false);
    let d = vec!["a", "b", "c"];

    let pp = ppred!(
        &pass!(&new_enum_assign_m!("var1_m", &d, "a", "t1")),
        &pass!(&new_enum_assign_m!("var1_c", &d, "c", "t1")),
        &pass!(&new_enum_assign_m!("var2_m", &d, "b", "t2")),
        &pass!(&new_enum_assign_m!("var2_c", &d, "a", "t2"))
    );

    let params = vec![p1, p2];
    println!("generated {:?}", generate_predicate(&pp, &params));
}

#[test]
fn test_parameterized() {
    let problem = models::dummy_robot::model::model("instance_1");

    let d = deactivate_all(&problem.params);
    println!("prms: {:?}", d);
    let result1 = parameterized(&problem, &d, 1200, 30);
    pprint_result(&result1);
    

    let p1 = &activate_next(&d);
    println!("prms: {:?}", p1);
    let result2 = parameterized(&problem, &p1, 1200, 30);
    pprint_result(&result2);
    

    let p2 = &activate_next(&p1);
    println!("prms: {:?}", p2);
    let result3 = parameterized(&problem, &p2, 1200, 30);
    pprint_result(&result3);
}
