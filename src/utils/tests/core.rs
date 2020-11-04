use super::*;

#[test]
fn test_get_predicate_vars() {
    let d = vec!["a", "b", "c", "d"];
    let vars = get_predicate_vars(
        &por!(
            &pass!(new_enum_assign_e!("x", &d, "a", "type")),
            &pass!(new_enum_assign_e!("y", &d, "b", "type")),
            &pass!(new_enum_assign_e!("z", &d, "c", "type"))
        )
    );
    assert_eq!(3, vars.len());
}

#[test]
fn test_get_param_predicate_vars() {
    let d = vec!["a", "b", "c", "d"];
    let vars = get_param_predicate_vars(
        &ppred!(
            &por!(
                &pass!(new_enum_assign_e!("x", &d, "a", "type")),
                &pass!(new_enum_assign_e!("y", &d, "b", "type")),
                &pass!(new_enum_assign_e!("z", &d, "c", "type"))
            )
        )
    );
    assert_eq!(3, vars.len());
}

#[test]
fn test_assignment_vector_to_predicate_vector() {
    let d = vec!["a", "b", "c", "d"];
    let assignments = vec!(
        new_enum_assign_e!("x", &d, "a", "type"),
        new_enum_assign_e!("y", &d, "b", "type"),
        new_enum_assign_e!("z", &d, "c", "type")
    );
    let pred_vec = assignment_vector_to_predicate_vector(&assignments);
    println!("{:?}", pred_vec)
}

#[test]
fn test_state_to_predicate() {
    let d = vec!["a", "b", "c", "d"];
    let assignments = vec!(
        new_enum_assign_e!("x", &d, "a", "type"),
        new_enum_assign_m!("y", &d, "b", "type"),
        new_enum_assign_c!("z", &d, "c", "type")
    );
    let state = State::from_vec(&assignments);
    let pred_vec = state_to_predicate(&state);
    println!("{:?}", pred_vec)
}

#[test]
fn test_state_to_param_predicate() {
    let d = vec!["a", "b", "c", "d"];
    let assignments = vec!(
        new_enum_assign_e!("x", &d, "a", "type"),
        new_enum_assign_m!("y", &d, "b", "type"),
        new_enum_assign_c!("z", &d, "c", "type")
    );
    let state = State::from_vec(&assignments);
    let pred_vec = state_to_param_predicate(&state);
    println!("{:?}", pred_vec)
}

