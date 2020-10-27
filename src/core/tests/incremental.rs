use super::*;
use std::time::Duration;
use z3_v2::*;

fn make_fruit(fruit: &str, kind: &Kind) -> EnumVariable {
    EnumVariable::new(
        fruit,
        &vec!["green", "ripe", "spoiled"],
        "fruit",
        None,
        &kind,
    )
}

fn is_fruit(fruit: &str, kind: &Kind) -> EnumVariable {
    EnumVariable {
        name: String::from(fruit),
        r#type: String::from("fruit"),
        domain: vec![
            String::from("green"),
            String::from("ripe"),
            String::from("spoiled"),
        ],
        param: Parameter {
            name: String::from("NONE"),
            value: true,
        },
        kind: kind.to_owned(),
    }
}

fn ripen(fruit: &str) -> Transition {
    Transition::new(
        "ripen",
        &Predicate::SET(EnumValue::new(
            &make_fruit(fruit, &Kind::Command),
            "green",
            Some(&Duration::from_millis(3000)),
        )),
        &Predicate::SET(EnumValue::new(
            &make_fruit(fruit, &Kind::Command),
            "ripe",
            Some(&Duration::from_millis(3000)),
        )),
    )
}

#[test]
fn test_new_transition() {
    assert_eq!(
        ripen("banana"),
        Transition {
            name: String::from("ripen"),
            guard: Predicate::SET(EnumValue {
                var: is_fruit("banana", &Kind::Command),
                val: String::from("green"),
                lifetime: Duration::from_millis(3000)
            }),
            update: Predicate::SET(EnumValue {
                var: make_fruit("banana", &Kind::Command),
                val: String::from("ripe"),
                lifetime: Duration::from_millis(3000)
            }),
            // kind: Kind::Command
        }
    )
}

#[test]
fn test_keep_variable_values() {
    let cfg = ConfigZ3::new();
    let ctx = ContextZ3::new(&cfg);
    let keep = keep_variable_values(
        &ctx,
        &vec![
            make_fruit("banana", &Kind::Command),
            make_fruit("peach", &Kind::Command),
        ],
        &ripen("banana"),
        &5,
    );
    assert_eq!("(and (= peach_s5 peach_s4))", ast_to_string_z3!(&ctx, keep));
}

#[test]
fn test_incremental() {
    pprint_result(&incremental(&models::dummy_robot::dummy_robot::raar_model()));
} 