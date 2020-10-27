use super::*;

// #[test]
// fn new_transition() {
//     assert_eq!(
//         Transition::new(
//             "ripen",
//             &Predicate::EQ(EnumValue::new(
//                 &EnumVariable::new(
//                     "banana",
//                     &vec!["green", "ripe", "spoiled"],
//                     None,
//                     &Kind::Estimated,
//                 ),
//                 "green",
//                 Some(&Duration::from_millis(3000)),
//             )),
//             &Predicate::EQ(EnumValue::new(
//                 &EnumVariable::new(
//                     "banana",
//                     &vec!["green", "ripe", "spoiled"],
//                     None,
//                     &Kind::Estimated,
//                 ),
//                 "ripe",
//                 Some(&Duration::from_millis(3000)),
//             ))
//         ),
//         Transition {
//             name: String::from("ripen"),
//             guard: Predicate::EQ(EnumValue {
//                 var: EnumVariable {
//                     name: String::from("banana"),
//                     r#type: String::from("banana"),
//                     domain: vec![
//                         String::from("green"),
//                         String::from("ripe"),
//                         String::from("spoiled")
//                     ],
//                     param: Parameter {
//                         name: String::from("NONE"),
//                         value: true
//                     },
//                     kind: Kind::Estimated
//                 },
//                 val: String::from("green"),
//                 lifetime: Duration::from_millis(3000)
//             }),
//             update: Predicate::EQ(EnumValue {
//                 var: EnumVariable {
//                     name: String::from("banana"),
//                     r#type: String::from("banana"),
//                     domain: vec![
//                         String::from("green"),
//                         String::from("ripe"),
//                         String::from("spoiled")
//                     ],
//                     param: Parameter {
//                         name: String::from("NONE"),
//                         value: true
//                     },
//                     kind: Kind::Estimated
//                 },
//                 val: String::from("ripe"),
//                 lifetime: Duration::from_millis(3000)
//             }),
//             kind: Kind::Estimated
//         }
//     )
// }
