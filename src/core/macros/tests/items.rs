use super::*;
use std::time::Duration;

fn is_bool_fruit(fruit: &str, param: &str, kind: &Kind) -> Variable {
    Variable {
        name: fruit.to_owned(),
        value_type: SPValueType::Bool,
        domain: vec![SPValue::Bool(true), SPValue::Bool(false)],
        param: Parameter::new(&param, &true),
        r#type: String::from("NONE"),
        kind: kind.to_owned(),
    }
}

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

fn is_bool_fruit_true(fruit: &str, param: &str, kind: &Kind, life: u64) -> Assignment {
    Assignment {
        var: Variable {
            name: fruit.to_owned(),
            value_type: SPValueType::Bool,
            domain: vec![SPValue::Bool(true), SPValue::Bool(false)],
            param: Parameter::new(&param, &true),
            r#type: String::from("NONE"),
            kind: kind.to_owned(),
        },
        val: SPValue::Bool(true),
        lifetime: Duration::from_millis(life),
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
fn test_bool_c_macro_1() {
    let banana = bool_c!("banana");
    assert_eq!(banana, is_bool_fruit("banana", "NONE", &Kind::Command))
}

#[test]
fn test_bool_c_macro_2() {
    let banana = bool_c!("banana", "p1");
    assert_eq!(banana, is_bool_fruit("banana", "p1", &Kind::Command))
}

#[test]
fn test_bool_m_macro_1() {
    let banana = bool_m!("banana");
    assert_eq!(banana, is_bool_fruit("banana", "NONE", &Kind::Measured))
}

#[test]
fn test_bool_m_macro_2() {
    let banana = bool_m!("banana", "p1");
    assert_eq!(banana, is_bool_fruit("banana", "p1", &Kind::Measured))
}

#[test]
fn test_bool_e_macro_1() {
    let banana = bool_e!("banana");
    assert_eq!(banana, is_bool_fruit("banana", "NONE", &Kind::Estimated))
}

#[test]
fn test_bool_e_macro_2() {
    let banana = bool_e!("banana", "p1");
    assert_eq!(banana, is_bool_fruit("banana", "p1", &Kind::Estimated))
}

#[test]
fn test_bool_assign_macro_1() {
    let banana = bool_c!("banana");
    let true_banana = bool_assign!(banana, true);
    assert_eq!(
        true_banana,
        is_bool_fruit_true("banana", "NONE", &Kind::Command, 6000)
    )
}

#[test]
fn test_bool_assign_macro_2() {
    let banana = bool_c!("banana", "p1");
    let true_banana = bool_assign!(banana, true, Duration::from_millis(1234));
    assert_eq!(
        true_banana,
        is_bool_fruit_true("banana", "p1", &Kind::Command, 1234)
    )
}

#[test]
fn test_new_bool_assign_c_macro_1() {
    let true_banana = new_bool_assign_c!("banana", true);
    assert_eq!(
        true_banana,
        is_bool_fruit_true("banana", "NONE", &Kind::Command, 6000)
    )
}

#[test]
fn test_new_bool_assign_c_macro_2() {
    let true_banana = new_bool_assign_c!("banana", true, "p1");
    assert_eq!(
        true_banana,
        is_bool_fruit_true("banana", "p1", &Kind::Command, 6000)
    )
}

#[test]
fn test_new_bool_assign_c_macro_3() {
    let true_banana = new_bool_assign_c!("banana", true, "p1", Duration::from_millis(1234));
    assert_eq!(
        true_banana,
        is_bool_fruit_true("banana", "p1", &Kind::Command, 1234)
    )
}

#[test]
fn test_new_bool_assign_m_macro_1() {
    let true_banana = new_bool_assign_m!("banana", true);
    assert_eq!(
        true_banana,
        is_bool_fruit_true("banana", "NONE", &Kind::Measured, 6000)
    )
}

#[test]
fn test_new_bool_assign_m_macro_2() {
    let true_banana = new_bool_assign_m!("banana", true, "p1");
    assert_eq!(
        true_banana,
        is_bool_fruit_true("banana", "p1", &Kind::Measured, 6000)
    )
}

#[test]
fn test_new_bool_assign_m_macro_3() {
    let true_banana = new_bool_assign_m!("banana", true, "p1", Duration::from_millis(1234));
    assert_eq!(
        true_banana,
        is_bool_fruit_true("banana", "p1", &Kind::Measured, 1234)
    )
}

#[test]
fn test_new_bool_assign_e_macro_1() {
    let true_banana = new_bool_assign_e!("banana", true);
    assert_eq!(
        true_banana,
        is_bool_fruit_true("banana", "NONE", &Kind::Estimated, 6000)
    )
}

#[test]
fn test_new_bool_assign_e_macro_2() {
    let true_banana = new_bool_assign_e!("banana", true, "p1");
    assert_eq!(
        true_banana,
        is_bool_fruit_true("banana", "p1", &Kind::Estimated, 6000)
    )
}

#[test]
fn test_new_bool_assign_e_macro_3() {
    let true_banana = new_bool_assign_e!("banana", true, "p1", Duration::from_millis(1234));
    assert_eq!(
        true_banana,
        is_bool_fruit_true("banana", "p1", &Kind::Estimated, 1234)
    )
}

#[test]
fn test_new_enum_c_macro_1() {
    let banana = enum_c!("banana", "fruit", vec!("green", "ripe", "spoiled"));
    assert_eq!(
        banana,
        is_enum_fruit(
            "banana",
            "fruit",
            &vec!("green", "ripe", "spoiled"),
            "NONE",
            &Kind::Command
        )
    )
}

#[test]
fn test_new_enum_c_macro_2() {
    let banana = enum_c!("banana", "fruit", vec!("green", "ripe", "spoiled"), "p1");
    assert_eq!(
        banana,
        is_enum_fruit(
            "banana",
            "fruit",
            &vec!("green", "ripe", "spoiled"),
            "p1",
            &Kind::Command
        )
    )
}

#[test]
fn test_new_enum_m_macro_1() {
    let banana = enum_m!("banana", "fruit", vec!("green", "ripe", "spoiled"));
    assert_eq!(
        banana,
        is_enum_fruit(
            "banana",
            "fruit",
            &vec!("green", "ripe", "spoiled"),
            "NONE",
            &Kind::Measured
        )
    )
}

#[test]
fn test_new_enum_m_macro_2() {
    let banana = enum_m!("banana", "fruit", vec!("green", "ripe", "spoiled"), "p1");
    assert_eq!(
        banana,
        is_enum_fruit(
            "banana",
            "fruit",
            &vec!("green", "ripe", "spoiled"),
            "p1",
            &Kind::Measured
        )
    )
}

#[test]
fn test_new_enum_e_macro_1() {
    let banana = enum_e!("banana", "fruit", vec!("green", "ripe", "spoiled"));
    assert_eq!(
        banana,
        is_enum_fruit(
            "banana",
            "fruit",
            &vec!("green", "ripe", "spoiled"),
            "NONE",
            &Kind::Estimated
        )
    )
}

#[test]
fn test_new_enum_e_macro_2() {
    let banana = enum_e!("banana", "fruit", vec!("green", "ripe", "spoiled"), "p1");
    assert_eq!(
        banana,
        is_enum_fruit(
            "banana",
            "fruit",
            &vec!("green", "ripe", "spoiled"),
            "p1",
            &Kind::Estimated
        )
    )
}

#[test]
fn test_enum_assign_macro_1() {
    let banana = enum_c!("banana", "fruit", vec!("green", "ripe", "spoiled"));
    let ripe_banana = enum_assign!(banana, "ripe");
    assert_eq!(
        ripe_banana,
        is_enum_fruit_something(
            "banana",
            "fruit",
            &vec!("green", "ripe", "spoiled"),
            "NONE",
            &Kind::Command,
            "ripe",
            6000
        )
    )
}

#[test]
fn test_enum_assign_macro_2() {
    let banana = enum_c!("banana", "fruit", vec!("green", "ripe", "spoiled"));
    let ripe_banana = enum_assign!(banana, "ripe", Duration::from_millis(1234));
    assert_eq!(
        ripe_banana,
        is_enum_fruit_something(
            "banana",
            "fruit",
            &vec!("green", "ripe", "spoiled"),
            "NONE",
            &Kind::Command,
            "ripe",
            1234
        )
    )
}

#[test]
fn test_new_enum_assign_c_macro_0() {
    let true_banana =
        new_enum_assign_c!("banana", vec!("green", "ripe", "spoiled"), "ripe");
    assert_eq!(
        true_banana,
        is_enum_fruit_something(
            "banana",
            "NONE",
            &vec!("green", "ripe", "spoiled"),
            "NONE",
            &Kind::Command,
            "ripe",
            6000
        )
    )
}

#[test]
fn test_new_enum_assign_c_macro_1() {
    let true_banana =
        new_enum_assign_c!("banana", "fruit", vec!("green", "ripe", "spoiled"), "ripe");
    assert_eq!(
        true_banana,
        is_enum_fruit_something(
            "banana",
            "fruit",
            &vec!("green", "ripe", "spoiled"),
            "NONE",
            &Kind::Command,
            "ripe",
            6000
        )
    )
}

#[test]
fn test_new_enum_assign_c_macro_2() {
    let true_banana = new_enum_assign_c!(
        "banana",
        "fruit",
        vec!("green", "ripe", "spoiled"),
        "ripe",
        "p1"
    );
    assert_eq!(
        true_banana,
        is_enum_fruit_something(
            "banana",
            "fruit",
            &vec!("green", "ripe", "spoiled"),
            "p1",
            &Kind::Command,
            "ripe",
            6000
        )
    )
}

#[test]
fn test_new_enum_assign_c_macro_3() {
    let true_banana = new_enum_assign_c!(
        "banana",
        "fruit",
        vec!("green", "ripe", "spoiled"),
        "ripe",
        "p1",
        Duration::from_millis(1234)
    );
    assert_eq!(
        true_banana,
        is_enum_fruit_something(
            "banana",
            "fruit",
            &vec!("green", "ripe", "spoiled"),
            "p1",
            &Kind::Command,
            "ripe",
            1234
        )
    )
}

#[test]
fn test_new_enum_assign_m_macro_0() {
    let true_banana =
        new_enum_assign_m!("banana", vec!("green", "ripe", "spoiled"), "ripe");
    assert_eq!(
        true_banana,
        is_enum_fruit_something(
            "banana",
            "NONE",
            &vec!("green", "ripe", "spoiled"),
            "NONE",
            &Kind::Measured,
            "ripe",
            6000
        )
    )
}

#[test]
fn test_new_enum_assign_m_macro_1() {
    let true_banana =
        new_enum_assign_m!("banana", "fruit", vec!("green", "ripe", "spoiled"), "ripe");
    assert_eq!(
        true_banana,
        is_enum_fruit_something(
            "banana",
            "fruit",
            &vec!("green", "ripe", "spoiled"),
            "NONE",
            &Kind::Measured,
            "ripe",
            6000
        )
    )
}

#[test]
fn test_new_enum_assign_m_macro_2() {
    let true_banana = new_enum_assign_m!(
        "banana",
        "fruit",
        vec!("green", "ripe", "spoiled"),
        "ripe",
        "p1"
    );
    assert_eq!(
        true_banana,
        is_enum_fruit_something(
            "banana",
            "fruit",
            &vec!("green", "ripe", "spoiled"),
            "p1",
            &Kind::Measured,
            "ripe",
            6000
        )
    )
}

#[test]
fn test_new_enum_assign_m_macro_3() {
    let true_banana = new_enum_assign_m!(
        "banana",
        "fruit",
        vec!("green", "ripe", "spoiled"),
        "ripe",
        "p1",
        Duration::from_millis(1234)
    );
    assert_eq!(
        true_banana,
        is_enum_fruit_something(
            "banana",
            "fruit",
            &vec!("green", "ripe", "spoiled"),
            "p1",
            &Kind::Measured,
            "ripe",
            1234
        )
    )
}

#[test]
fn test_new_enum_assign_e_macro_0() {
    let true_banana =
        new_enum_assign_e!("banana", vec!("green", "ripe", "spoiled"), "ripe");
    assert_eq!(
        true_banana,
        is_enum_fruit_something(
            "banana",
            "NONE",
            &vec!("green", "ripe", "spoiled"),
            "NONE",
            &Kind::Estimated,
            "ripe",
            6000
        )
    )
}

#[test]
fn test_new_enum_assign_e_macro_1() {
    let true_banana =
        new_enum_assign_e!("banana", "fruit", vec!("green", "ripe", "spoiled"), "ripe");
    assert_eq!(
        true_banana,
        is_enum_fruit_something(
            "banana",
            "fruit",
            &vec!("green", "ripe", "spoiled"),
            "NONE",
            &Kind::Estimated,
            "ripe",
            6000
        )
    )
}

#[test]
fn test_new_enum_assign_e_macro_2() {
    let true_banana = new_enum_assign_e!(
        "banana",
        "fruit",
        vec!("green", "ripe", "spoiled"),
        "ripe",
        "p1"
    );
    assert_eq!(
        true_banana,
        is_enum_fruit_something(
            "banana",
            "fruit",
            &vec!("green", "ripe", "spoiled"),
            "p1",
            &Kind::Estimated,
            "ripe",
            6000
        )
    )
}

#[test]
fn test_new_enum_assign_e_macro_3() {
    let true_banana = new_enum_assign_e!(
        "banana",
        "fruit",
        vec!("green", "ripe", "spoiled"),
        "ripe",
        "p1",
        Duration::from_millis(1234)
    );
    assert_eq!(
        true_banana,
        is_enum_fruit_something(
            "banana",
            "fruit",
            &vec!("green", "ripe", "spoiled"),
            "p1",
            &Kind::Estimated,
            "ripe",
            1234
        )
    )
}