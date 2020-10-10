use lib::*;
use std::time::Duration;

#[test]
fn new_parameter() {
    assert_eq!(
        Parameter::new("some_name", &false),
        Parameter {
            name: String::from("some_name"),
            value: false
        }
    )
}

#[test]
fn none_parameter() {
    assert_eq!(
        Parameter::none(),
        Parameter {
            name: String::from("NONE"),
            value: true
        }
    )
}

#[test]
fn new_variable() {
    assert_eq!(
        EnumVariable::new(
            "banana",
            &vec!["green", "ripe", "spoiled"],
            None,
            &Kind::Estimated,
        ),
        EnumVariable {
            name: String::from("banana"),
            r#type: String::from("banana"),
            domain: vec![
                String::from("green"),
                String::from("ripe"),
                String::from("spoiled")
            ],
            param: Parameter {
                name: String::from("NONE"),
                value: true
            },
            kind: Kind::Estimated
        }
    )
}

#[test]
fn new_value() {
    assert_eq!(
        EnumValue::new(
            &EnumVariable::new(
                "banana",
                &vec!["green", "ripe", "spoiled"],
                None,
                &Kind::Estimated,
            ),
            "green",
            Some(&Duration::from_millis(3000)),
        ),
        EnumValue {
            var: EnumVariable {
                name: String::from("banana"),
                r#type: String::from("banana"),
                domain: vec![
                    String::from("green"),
                    String::from("ripe"),
                    String::from("spoiled")
                ],
                param: Parameter {
                    name: String::from("NONE"),
                    value: true
                },
                kind: Kind::Estimated
            },
            val: String::from("green"),
            lifetime: Duration::from_millis(3000)
        }
    );
}

#[test]
#[should_panic]
fn new_value_panic() {
    EnumValue::new(
        &EnumVariable::new(
            "banana",
            &vec!["green", "ripe", "spoiled"],
            None,
            &Kind::Estimated,
        ),
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
                    &EnumVariable::new(
                        "banana",
                        &vec!["green", "ripe", "spoiled"],
                        None,
                        &Kind::Estimated,
                    ),
                    "green",
                    Some(&Duration::from_millis(3000)),
                ),
                EnumValue::new(
                    &EnumVariable::new(
                        "peach",
                        &vec!["green", "ripe", "spoiled"],
                        None,
                        &Kind::Estimated,
                    ),
                    "green",
                    Some(&Duration::from_millis(3000)),
                ),
            ),
            &Kind::Estimated
        ),
        State {
            vec: vec![
                EnumValue {
                    var: EnumVariable {
                        name: String::from("banana"),
                        r#type: String::from("banana"),
                        domain: vec![
                            String::from("green"),
                            String::from("ripe"),
                            String::from("spoiled")
                        ],
                        param: Parameter {
                            name: String::from("NONE"),
                            value: true
                        },
                        kind: Kind::Estimated
                    },
                    val: String::from("green"),
                    lifetime: Duration::from_millis(3000)
                },
                EnumValue {
                    var: EnumVariable {
                        name: String::from("peach"),
                        r#type: String::from("peach"),
                        domain: vec![
                            String::from("green"),
                            String::from("ripe"),
                            String::from("spoiled")
                        ],
                        param: Parameter {
                            name: String::from("NONE"),
                            value: true
                        },
                        kind: Kind::Estimated
                    },
                    val: String::from("green"),
                    lifetime: Duration::from_millis(3000)
                }
            ],
            kind: Kind::Estimated
        }
    )
}

#[test]
#[should_panic]
fn new_state_panic() {
    State::new(
        &vec![EnumValue::new(
            &EnumVariable::new(
                "banana",
                &vec!["green", "ripe", "spoiled"],
                None,
                &Kind::Estimated,
            ),
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
fn new_complete_state() {
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
                &EnumVariable::new(
                    "banana",
                    &vec!["green", "ripe", "spoiled"],
                    None,
                    &Kind::Estimated,
                ),
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
                &EnumVariable::new(
                    "banana",
                    &vec!["green", "ripe", "spoiled"],
                    None,
                    &Kind::Estimated,
                ),
                "green",
                Some(&Duration::from_millis(3000)),
            ),
            EnumValue::new(
                &EnumVariable::new(
                    "peach",
                    &vec!["green", "ripe", "spoiled"],
                    None,
                    &Kind::Command,
                ),
                "green",
                Some(&Duration::from_millis(3000)),
            ),
            EnumValue::new(
                &EnumVariable::new(
                    "pear",
                    &vec!["green", "ripe", "spoiled"],
                    None,
                    &Kind::Measured,
                ),
                "green",
                Some(&Duration::from_millis(3000)),
            ),
        )),
        CompleteState {
            measured: State {
                vec: vec![EnumValue {
                    var: EnumVariable {
                        name: String::from("pear"),
                        r#type: String::from("pear"),
                        domain: vec![
                            String::from("green"),
                            String::from("ripe"),
                            String::from("spoiled")
                        ],
                        param: Parameter {
                            name: String::from("NONE"),
                            value: true
                        },
                        kind: Kind::Measured
                    },
                    val: String::from("green"),
                    lifetime: Duration::from_millis(3000)
                }],
                kind: Kind::Measured
            },
            command: State {
                vec: vec![EnumValue {
                    var: EnumVariable {
                        name: String::from("peach"),
                        r#type: String::from("peach"),
                        domain: vec![
                            String::from("green"),
                            String::from("ripe"),
                            String::from("spoiled")
                        ],
                        param: Parameter {
                            name: String::from("NONE"),
                            value: true
                        },
                        kind: Kind::Command
                    },
                    val: String::from("green"),
                    lifetime: Duration::from_millis(3000)
                }],
                kind: Kind::Command
            },
            estimated: State {
                vec: vec![EnumValue {
                    var: EnumVariable {
                        name: String::from("banana"),
                        r#type: String::from("banana"),
                        domain: vec![
                            String::from("green"),
                            String::from("ripe"),
                            String::from("spoiled")
                        ],
                        param: Parameter {
                            name: String::from("NONE"),
                            value: true
                        },
                        kind: Kind::Estimated
                    },
                    val: String::from("green"),
                    lifetime: Duration::from_millis(3000)
                }],
                kind: Kind::Estimated
            }
        }
    )
}
