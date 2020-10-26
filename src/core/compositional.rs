use super::*;
use std::time::Instant;
use z3_sys::*;
use z3_v2::*;

/// Given a parameter list, return it with the next value activated
pub fn activate_next(params: &Vec<Parameter>) -> Vec<Parameter> {
    let mut one_activated = false;
    params
        .iter()
        .map(|x| {
            if !x.value && !one_activated {
                one_activated = true;
                Parameter::new(x.name.as_str(), &true)
            } else {
                x.to_owned().to_owned()
            }
        })
        .collect()
}



#[test]
fn test_activate_next() {
    let p1 = Parameter::new("A", &true);
    let p2 = Parameter::new("B", &false);
    let p3 = Parameter::new("C", &false);
    let params = vec![p1, p2, p3];
    assert_eq!(&format!("{:?}", activate_next(&params)), 
        "[Parameter { name: \"A\", value: true }, Parameter { name: \"B\", value: true }, Parameter { name: \"C\", value: false }]");
}
