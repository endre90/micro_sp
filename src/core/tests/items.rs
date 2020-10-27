use super::*;
use std::time::Duration;

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

#[test]
fn new_variable() {
    assert_eq!(
        make_fruit("banana", &Kind::Command),
        is_fruit("banana", &Kind::Command)
    )
}

#[test]
fn new_value() {
    assert_eq!(
        EnumValue::new(
            &make_fruit("banana", &Kind::Command),
            "green",
            Some(&Duration::from_millis(3000)),
        ),
        EnumValue {
            var: is_fruit("banana", &Kind::Command),
            val: String::from("green"),
            lifetime: Duration::from_millis(3000)
        }
    );
}

#[test]
#[should_panic]
fn new_value_panic() {
    EnumValue::new(
        &make_fruit("banana", &Kind::Command),
        "not_in_domain",
        Some(&Duration::from_millis(3000)),
    );
}

#[test]
fn new_state_empty() {
    assert_eq!(
        State::new(&vec!(), &Kind::Command),
        State {
            vec: vec![],
            kind: Kind::Command
        }
    )
}

#[test]
fn new_state() {
    assert_eq!(
        State::new(
            &vec!(
                EnumValue::new(
                    &make_fruit("banana", &Kind::Command),
                    "green",
                    Some(&Duration::from_millis(3000)),
                ),
                EnumValue::new(
                    &make_fruit("peach", &Kind::Command),
                    "green",
                    Some(&Duration::from_millis(3000)),
                ),
            ),
            &Kind::Command
        ),
        State {
            vec: vec![
                EnumValue {
                    var: is_fruit("banana", &Kind::Command),
                    val: String::from("green"),
                    lifetime: Duration::from_millis(3000)
                },
                EnumValue {
                    var: is_fruit("peach", &Kind::Command),
                    val: String::from("green"),
                    lifetime: Duration::from_millis(3000)
                }
            ],
            kind: Kind::Command
        }
    )
}

#[test]
#[should_panic]
fn new_state_panic() {
    State::new(
        &vec![EnumValue::new(
            &make_fruit("banana", &Kind::Estimated),
            "green",
            Some(&Duration::from_millis(3000)),
        )],
        &Kind::Command,
    );
}

#[test]
fn new_complete_state_empty() {
    assert_eq!(
        CompleteState::empty(),
        CompleteState {
            measured: State {
                vec: vec![],
                kind: Kind::Measured
            },
            command: State {
                vec: vec![],
                kind: Kind::Command
            },
            estimated: State {
                vec: vec![],
                kind: Kind::Estimated
            }
        }
    )
}

#[test]
fn new_complete_state_from_states() {
    assert_eq!(
        CompleteState::from_states(
            &State::new(&vec!(), &Kind::Measured),
            &State::new(&vec!(), &Kind::Command),
            &State::new(&vec!(), &Kind::Estimated)
        ),
        CompleteState {
            measured: State {
                vec: vec![],
                kind: Kind::Measured
            },
            command: State {
                vec: vec![],
                kind: Kind::Command
            },
            estimated: State {
                vec: vec![],
                kind: Kind::Estimated
            }
        }
    )
}

#[test]
#[should_panic]
fn new_complete_state_panic() {
    CompleteState::from_states(
        &State::new(&vec![], &Kind::Measured),
        &State::new(
            &vec![EnumValue::new(
                &make_fruit("banana", &Kind::Estimated),
                "green",
                Some(&Duration::from_millis(3000)),
            )],
            &Kind::Command,
        ),
        &State::new(&vec![], &Kind::Estimated),
    );
}

#[test]
fn new_complete_state_from_vec() {
    assert_eq!(
        CompleteState::from_vec(&vec!(
            EnumValue::new(
                &make_fruit("banana", &Kind::Estimated),
                "green",
                Some(&Duration::from_millis(3000)),
            ),
            EnumValue::new(
                &make_fruit("peach", &Kind::Command),
                "green",
                Some(&Duration::from_millis(3000)),
            ),
            EnumValue::new(
                &make_fruit("pear", &Kind::Measured),
                "green",
                Some(&Duration::from_millis(3000)),
            ),
        )),
        CompleteState {
            measured: State {
                vec: vec![EnumValue {
                    var: is_fruit("pear", &Kind::Measured),
                    val: String::from("green"),
                    lifetime: Duration::from_millis(3000)
                }],
                kind: Kind::Measured
            },
            command: State {
                vec: vec![EnumValue {
                    var: is_fruit("peach", &Kind::Command),
                    val: String::from("green"),
                    lifetime: Duration::from_millis(3000)
                }],
                kind: Kind::Command
            },
            estimated: State {
                vec: vec![EnumValue {
                    var: is_fruit("banana", &Kind::Estimated),
                    val: String::from("green"),
                    lifetime: Duration::from_millis(3000)
                }],
                kind: Kind::Estimated
            }
        }
    )
}
