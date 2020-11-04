use super::*;
use std::time::Duration;

fn is_enum_fruit(
    fruit: &str,
    r#type: &str,
    domain: &Vec<&str>,
    param: &str,
    kind: &Kind,
) -> Variable {
    Variable {
        name: fruit.to_owned(),
        value_type: SPValueType::String,
        domain: domain
            .iter()
            .map(|x| String::from(x.to_owned()).to_spvalue())
            .collect(),
        param: Parameter::new(&param, &true),
        r#type: r#type.to_owned(),
        kind: kind.to_owned(),
    }
}

fn is_enum_fruit_something(
    fruit: &str,
    r#type: &str,
    domain: &Vec<&str>,
    param: &str,
    kind: &Kind,
    value: &str,
    life: u64,
) -> Assignment {
    Assignment {
        var: Variable {
            name: fruit.to_owned(),
            value_type: SPValueType::String,
            domain: domain
                .iter()
                .map(|x| String::from(x.to_owned()).to_spvalue())
                .collect(),
            param: Parameter::new(&param, &true),
            r#type: r#type.to_owned(),
            kind: kind.to_owned(),
        },
        val: SPValue::String(String::from(value)),
        lifetime: Duration::from_millis(life),
    }
}


#[test]
fn test_ptrue_macro() {
    let t = ptrue!();
    assert_eq!(t, Predicate::TRUE)
}

#[test]
fn test_pfalse_macro() {
    let f = pfalse!();
    assert_eq!(f, Predicate::FALSE)
}

#[test]
fn test_pass_macro() {
    let ripe_banana = new_enum_assign_c!("banana", vec!("ripe", "green", "spoiled"), "ripe", "fruit");
    let pass = pass!(&ripe_banana);
    assert_eq!(pass, Predicate::ASS(ripe_banana))
}

#[test]
fn test_pnot_macro() {
    let ripe_banana = new_enum_assign_c!("banana", vec!("ripe", "green", "spoiled"), "ripe", "fruit");
    let pass = pass!(&ripe_banana);
    let pnot = pnot!(&pass);
    assert_eq!(pnot, Predicate::NOT(Box::new(Predicate::ASS(ripe_banana))))
}

#[test]
fn test_pand_macro() {
    let ripe_banana = new_enum_assign_c!("banana", vec!("ripe", "green", "spoiled"), "ripe", "fruit");
    let spoiled_peach = new_enum_assign_c!("peach", vec!("ripe", "green", "spoiled"), "spoiled", "fruit");
    let pass_b = pass!(&ripe_banana);
    let pass_c = pass!(&spoiled_peach);
    let pand = pand!(&pass_b, &pass_c);
    assert_eq!(pand, Predicate::AND(vec!(pass_b, pass_c)))
}

#[test]
fn test_por_macro() {
    let ripe_banana = new_enum_assign_c!("banana", vec!("ripe", "green", "spoiled"), "ripe", "fruit");
    let spoiled_peach = new_enum_assign_c!("peach", vec!("ripe", "green", "spoiled"), "spoiled", "fruit");
    let pass_b = pass!(&ripe_banana);
    let pass_c = pass!(&spoiled_peach);
    let por = por!(&pass_b, &pass_c);
    assert_eq!(por, Predicate::OR(vec!(pass_b, pass_c)))
}

#[test]
fn test_ppbeq_macro() {
    let ripe_banana = new_enum_assign_c!("banana", vec!("ripe", "green", "spoiled"), "ripe", "fruit");
    let spoiled_peach = new_enum_assign_c!("peach", vec!("ripe", "green", "spoiled"), "spoiled", "fruit");
    let green_pear = new_enum_assign_c!("pear", vec!("ripe", "green", "spoiled"), "green", "fruit");
    let pass_b = pass!(&ripe_banana);
    let pass_c = pass!(&spoiled_peach);
    let pass_g = pass!(&green_pear);
    let ppbeq = ppbeq!(2, &pass_b, &pass_c, &pass_g);
    assert_eq!(ppbeq, Predicate::PBEQ(vec!(pass_b, pass_c, pass_g), 2))
}