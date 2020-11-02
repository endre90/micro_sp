use super::*;
use std::time::Duration;
use z3_v2::*;

// fn is_fruit(fruit: &str, kind: &Kind) -> EnumVariable {
//     EnumVariable {
//         name: String::from(fruit),
//         r#type: String::from("fruit"),
//         domain: vec![
//             String::from("green"),
//             String::from("ripe"),
//             String::from("spoiled"),
//         ],
//         param: Parameter {
//             name: String::from("NONE"),
//             value: true,
//         },
//         kind: kind.to_owned(),
//     }
// }

fn ripen(fruit: &str) -> Transition {
    Transition::new(
        "ripen",
        &Predicate::ASS(new_enum_assign_c!(
            fruit,
            "fruit",
            vec!("green", "ripe", "spoiled"),
            "green",
            ("p1", &true)
        )),
        &Predicate::ASS(new_enum_assign_c!(
            fruit,
            "fruit",
            vec!("green", "ripe", "spoiled"),
            "ripe",
            ("p1", &true)
        )),
    )
}

// #[test]
// fn test_new_transition() {
//     assert_eq!(
//         ripen("banana"),
//         Transition {
//             name: String::from("ripen"),
//             guard: Predicate::ASS(Assignment {
//                 var: is_fruit("banana", &Kind::Command),
//                 val: String::from("green"),
//                 lifetime: Duration::from_millis(3000)
//             }),
//             update: Predicate::SET(EnumValue {
//                 var: make_fruit("banana", &Kind::Command),
//                 val: String::from("ripe"),
//                 lifetime: Duration::from_millis(3000)
//             }),
//             // kind: Kind::Command
//         }
//     )
// }

#[test]
fn test_keep_variable_values() {
    let cfg = ConfigZ3::new();
    let ctx = ContextZ3::new(&cfg);
    let keep = keep_variable_values(
        &ctx,
        &vec![
            enum_c!("banana", "fruit", vec!("green", "ripe", "spoiled"), ("p1", &true)),
            enum_c!("peach", "fruit", vec!("green", "ripe", "spoiled"), ("p1", &true)),
        ],
        &ripen("banana"),
        &5,
    );
    assert_eq!("(and (= peach_s5 peach_s4))", ast_to_string_z3!(&ctx, keep));
}

// #[test]
// fn test_incremental() {
//     pprint_result(&incremental(&models::dummy_robot::dummy_robot::model()));
// } 