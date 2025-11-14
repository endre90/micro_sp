// use crate::{Action, Predicate, SPValue, SPVariable, SPWrapped, State, ToSPValue, ToSPWrapped};
use crate::*;

peg::parser!(pub grammar pred_parser() for str {

    rule _() =  quiet!{[' ' | '\t']*}


    pub rule variable(state: &State) -> SPVariable =
        "var:" _ n:$(['a'..='z' | 'A'..='Z' | '0'..='9' | '_' | '/']+) !(['a'..='z' | 'A'..='Z' | '0'..='9' | '_']) {
        state.get_assignment(n, "parser").var
    }

    pub rule array_element(state: &State) -> SPValue =
            v:value(state) {
                match v {
                    SPWrapped::SPValue(val) => val,
                    SPWrapped::SPVariable(sp_var) => panic!("ASDFASDF")
                    }
                }


    pub rule value(state: &State) -> SPWrapped
        = _ var:variable(&state) _ { SPWrapped::SPVariable(var) }
        / _ "UNKNOWN_bool" _ { SPWrapped::SPValue(SPValue::Bool(BoolOrUnknown::UNKNOWN)) }
        / _ "UNKNOWN_int" _ { SPWrapped::SPValue(SPValue::Int64(IntOrUnknown::UNKNOWN)) }
        / _ "UNKNOWN_float" _ { SPWrapped::SPValue(SPValue::Float64(FloatOrUnknown::UNKNOWN)) }
        / _ "UNKNOWN_time" _ { SPWrapped::SPValue(SPValue::Time(TimeOrUnknown::UNKNOWN)) }
        / _ "UNKNOWN_string" _ { SPWrapped::SPValue(SPValue::String(StringOrUnknown::UNKNOWN)) }
        / _ "UNKNOWN_array" _ { SPWrapped::SPValue(SPValue::Array(ArrayOrUnknown::UNKNOWN)) }
        / _ "true" _ { SPWrapped::SPValue(true.to_spvalue()) }
        / _ "TRUE" _ { SPWrapped::SPValue(true.to_spvalue()) }
        / _ "false" _ { SPWrapped::SPValue(false.to_spvalue()) }
        / _ "FALSE" _ { SPWrapped::SPValue(false.to_spvalue()) }
        / _ "[" _ items:(array_element(state) ** (_ "," _))? _ "]" _ {
            SPWrapped::SPValue(SPValue::Array(ArrayOrUnknown::Array(
                items.unwrap_or_else(Vec::new)
            )))
        }
        / _ "\"" n:$(!['"'] [_])* "\"" _ { // Quoted string
            SPWrapped::SPValue(n.into_iter().collect::<Vec<_>>().join("").to_spvalue())
        }
        / _ "ip:" _ "[" _ content:$([^']']*) _ "]" _ {
            SPWrapped::SPValue(content.to_spvalue())
        }
        / _ n:$("-"? ['0'..='9']+ "." ['0'..='9']+) !(['a'..='z' | 'A'..='Z' | '0'..='9' | '_']) _ { // Float
            let f: f64 = n.parse().expect("Failed to parse float");
            SPWrapped::SPValue(f.to_spvalue())
        }
        / _ n:$("-"? ['0'..='9']+) !(['a'..='z' | 'A'..='Z' | '0'..='9' | '_']) _ { // Integer
            let i: i64 = n.parse().expect("Failed to parse integer");
            SPWrapped::SPValue(i.to_spvalue())
        }
        / _ n:$(['a'..='z' | 'A'..='Z' | '0'..='9' | '_' | '-']+) _ {
            SPWrapped::SPValue(n.to_spvalue())
        }

    pub rule eq(state: &State) -> Predicate
        = p1:value(&state) _ "==" _ p2:value(&state) { Predicate::EQ(p1,p2) }
        / p1:value(&state) _ "!=" _ p2:value(&state) { Predicate::NEQ(p1,p2) }

    pub rule pred(state: &State) -> Predicate = precedence!{
        _ p:eq(&state) { p }
        --
            a:@ _ "->" _ b:(@) { Predicate::OR(vec![Predicate::NOT(Box::new(a)), b]) }
        --
            a:@ _ "||" _ b:(@) {
                match b {
                    Predicate::OR(x) => {
                        let mut v = vec![a];
                        v.extend(x);
                        Predicate::OR(v)
                    }
                    _ => Predicate::OR(vec![a, b])
                }
            }
        --
            a:@ _ "&&" _ b:(@) {
                match b {
                    Predicate::AND(x) => {
                        let mut v = vec![a];
                        v.extend(x);
                        Predicate::AND(v)
                    }
                    _ => Predicate::AND(vec![a, b])
                }
            }
        --
            _ "!" _ p:pred(&state) { Predicate::NOT(Box::new(p)) }
        --
            _ "(" _ p:pred(&state) _ ")" _ { p }
        --
        _ "TRUE" _ { Predicate::TRUE }
        _ "true" _ { Predicate::TRUE }
        _ "FALSE" _ { Predicate::FALSE }
        _ "false" _ { Predicate::FALSE }
    }

    pub rule action(state: &State) -> Action
        = p1:variable(&state) _ "<-" _ p2:variable(&state) _ "+" _ p3:value(&state) {
            let p2_val = state.get_value(&p2.name, "parser");

            let p3_val = match p3 {
                SPWrapped::SPValue(val) => val,
                SPWrapped::SPVariable(var) => {
                    state.get_value(&var.name, "parser").expect("variable not found")
                }
            };

            let new_val = match (p2_val, p3_val) {
                (Some(SPValue::Int64(v2)), SPValue::Int64(v3)) => {
                    let i2 = match v2 {
                        IntOrUnknown::Int64(i) => i,
                        IntOrUnknown::UNKNOWN => 0,
                    };
                    let i3 = match v3 {
                        IntOrUnknown::Int64(i) => i,
                        IntOrUnknown::UNKNOWN => 0,
                    };
                    (i2 + i3).to_spvalue()
                },
                (Some(SPValue::Float64(v2)), SPValue::Float64(v3)) => {
                    let f2 = match v2 {
                        FloatOrUnknown::Float64(ordered_float::OrderedFloat(f)) => f,
                        FloatOrUnknown::UNKNOWN => 0.0,
                    };
                    let f3 = match v3 {
                        FloatOrUnknown::Float64(ordered_float::OrderedFloat(f)) => f,
                        FloatOrUnknown::UNKNOWN => 0.0,
                    };
                    (f2 + f3).to_spvalue()
                },
                _ => panic!("Can only add int to int or float to float")
            };

            Action::new(p1, SPWrapped::SPValue(new_val))
        }

        / p1:variable(&state) _ "<-" _ p2:variable(&state) _ "-" _ p3:value(&state) {
            let p2_val = state.get_value(&p2.name, "parser");

            let p3_val = match p3 {
                SPWrapped::SPValue(val) => val,
                SPWrapped::SPVariable(var) => {
                    state.get_value(&var.name, "parser").expect("variable not found")
                }
            };

            let new_val = match (p2_val, p3_val) {
                (Some(SPValue::Int64(v2)), SPValue::Int64(v3)) => {
                    let i2 = match v2 {
                        IntOrUnknown::Int64(i) => i,
                        IntOrUnknown::UNKNOWN => 0,
                    };
                    let i3 = match v3 {
                        IntOrUnknown::Int64(i) => i,
                        IntOrUnknown::UNKNOWN => 0,
                    };
                    (i2 - i3).to_spvalue()
                },
                (Some(SPValue::Float64(v2)), SPValue::Float64(v3)) => {
                    let f2 = match v2 {
                        FloatOrUnknown::Float64(ordered_float::OrderedFloat(f)) => f,
                        FloatOrUnknown::UNKNOWN => 0.0,
                    };
                    let f3 = match v3 {
                        FloatOrUnknown::Float64(ordered_float::OrderedFloat(f)) => f,
                        FloatOrUnknown::UNKNOWN => 0.0,
                    };
                    (f2 - f3).to_spvalue()
                },
                _ => panic!("Can only add int to int or float to float")
            };

            Action::new(p1, SPWrapped::SPValue(new_val))
        }
        / p1:variable(&state) _ "<-" _ p2:variable(&state) { Action::new(p1, p2.wrap()) }
        / p1:variable(&state) _ "<-" _ p2:value(&state) { Action::new(p1, p2) }
    }
);

#[cfg(test)]
mod tests {

    // use ordered_float::OrderedFloat;

    use crate::{Predicate::*, *};

    fn john_doe() -> Vec<(SPVariable, SPValue)> {
        let name = v!("name");
        let surname = v!("surname");
        let height = iv!("height");
        let age = fv!("age");
        let weight = fv!("weight");
        let smart = bv!("smart");
        let alive = bv!("alive");

        vec![
            (name, "John".to_spvalue()),
            (surname, "Doe".to_spvalue()),
            (height, 185.to_spvalue()),
            (weight, 80.0.to_spvalue()),
            (age, 30.0.to_spvalue()),
            (smart, true.to_spvalue()),
            (alive, true.to_spvalue()),
        ]
    }

    #[test]
    fn parse_values() {
        let s = State::new();
        assert_eq!(
            pred_parser::value("9", &s),
            Ok(SPWrapped::SPValue(9.to_spvalue()))
        );
        assert_eq!(
            pred_parser::value("19.123", &s),
            Ok(SPWrapped::SPValue(19.123.to_spvalue()))
        );
        let asdf = -19.123;
        assert_eq!(
            pred_parser::value("-19.123", &s),
            Ok(SPWrapped::SPValue(asdf.to_spvalue()))
        );
        assert_eq!(
            pred_parser::value("hej", &s),
            Ok(SPWrapped::SPValue("hej".to_spvalue()))
        );

        assert_eq!(
            pred_parser::value("[0.3, 0.7, -12.67]", &s),
            Ok(SPWrapped::SPValue(vec![0.3, 0.7, -12.67].to_spvalue()))
        );

        assert_eq!(
            pred_parser::value("true", &s),
            Ok(SPWrapped::SPValue(true.to_spvalue()))
        );
        assert_eq!(
            pred_parser::value("TRUE", &s),
            Ok(SPWrapped::SPValue(true.to_spvalue()))
        );
        assert_eq!(
            pred_parser::value("false", &s),
            Ok(SPWrapped::SPValue(false.to_spvalue()))
        );
        assert_eq!(
            pred_parser::value("FALSE", &s),
            Ok(SPWrapped::SPValue(false.to_spvalue()))
        );
        assert_eq!(
            pred_parser::value("UNKNOWN_bool", &s),
            Ok(SPWrapped::SPValue(SPValue::Bool(BoolOrUnknown::UNKNOWN)))
        );
        assert_eq!(
            pred_parser::value("UNKNOWN_int", &s),
            Ok(SPWrapped::SPValue(SPValue::Int64(IntOrUnknown::UNKNOWN)))
        );
        assert_eq!(
            pred_parser::value("UNKNOWN_float", &s),
            Ok(SPWrapped::SPValue(SPValue::Float64(
                FloatOrUnknown::UNKNOWN
            )))
        );
        assert_eq!(
            pred_parser::value("UNKNOWN_string", &s),
            Ok(SPWrapped::SPValue(SPValue::String(
                StringOrUnknown::UNKNOWN
            )))
        );
        assert_eq!(
            pred_parser::value("UNKNOWN_time", &s),
            Ok(SPWrapped::SPValue(SPValue::Time(TimeOrUnknown::UNKNOWN)))
        );
        assert_eq!(
            pred_parser::value("UNKNOWN_array", &s),
            Ok(SPWrapped::SPValue(SPValue::Array(ArrayOrUnknown::UNKNOWN)))
        );
    }

    #[test]
    fn parse_variables() {
        let s = State::from_vec(&john_doe());
        assert_eq!(
            pred_parser::variable("var: height", &s),
            Ok(s.get_assignment("height", "t").var)
        );
    }

    #[test]
    #[should_panic]
    fn parse_variables_panic() {
        let s = State::from_vec(&john_doe());
        let _ = pred_parser::variable("var: wealth", &s);
    }

    #[test]
    fn parse_predicates() {
        let s = State::from_vec(&john_doe());
        let and = "TRUE && TRUE";
        let and2 = AND(vec![TRUE, TRUE]);
        assert_eq!(pred_parser::pred(and, &s), Ok(and2));

        let and = "TRUE  && TRUE && FALSE ";
        let and2 = AND(vec![TRUE, TRUE, FALSE]);
        assert_eq!(pred_parser::pred(and, &s), Ok(and2));

        let or = "TRUE || TRUE || FALSE";
        let or2 = OR(vec![TRUE, TRUE, FALSE]);
        assert_eq!(pred_parser::pred(or, &s), Ok(or2));

        let not_or = "TRUE || ! ( TRUE || FALSE && TRUE)";
        let not_or2 = OR(vec![
            TRUE,
            NOT(Box::new(OR(vec![TRUE, AND(vec![FALSE, TRUE])]))),
        ]);
        assert_eq!(pred_parser::pred(not_or, &s), Ok(not_or2));

        let eq1 = "TRUE == TRUE";
        let eq2 = EQ(
            SPWrapped::SPValue(true.to_spvalue()),
            SPWrapped::SPValue(true.to_spvalue()),
        );
        assert_eq!(pred_parser::eq(eq1, &s), Ok(eq2));

        let eq1 = "var:smart == FALSE";
        let eq2 = EQ(bv!("smart").wrap(), false.wrap());
        assert_eq!(pred_parser::eq(eq1, &s), Ok(eq2));

        let eq1 = "var:smart == true";
        let eq2 = EQ(bv!("smart").wrap(), true.wrap());
        assert_eq!(pred_parser::eq(eq1, &s), Ok(eq2));

        let neq1 = "var:smart != true";
        let neq2 = NEQ(bv!("smart").wrap(), true.wrap());
        assert_eq!(pred_parser::eq(neq1, &s), Ok(neq2));

        let eq1 = "TRUE == TRUE || FALSE != FALSE";
        let eq2 = EQ(
            SPWrapped::SPValue(true.to_spvalue()),
            SPWrapped::SPValue(true.to_spvalue()),
        );
        let eq3 = NEQ(
            SPWrapped::SPValue(false.to_spvalue()),
            SPWrapped::SPValue(false.to_spvalue()),
        );
        let or = OR(vec![eq2, eq3]);
        assert_eq!(pred_parser::pred(eq1, &s), Ok(or));

        let eq1 = "TRUE == TRUE || !(FALSE != FALSE)";
        let eq2 = EQ(
            SPWrapped::SPValue(true.to_spvalue()),
            SPWrapped::SPValue(true.to_spvalue()),
        );
        let eq3 = NEQ(
            SPWrapped::SPValue(false.to_spvalue()),
            SPWrapped::SPValue(false.to_spvalue()),
        );
        let or = OR(vec![eq2, NOT(Box::new(eq3))]);
        assert_eq!(pred_parser::pred(eq1, &s), Ok(or));

        let eq1 = "var:smart == TRUE || !(FALSE != var:smart)";
        let hej = s.get_assignment("smart", "t").var.wrap();
        let eq2 = EQ(hej.clone(), true.to_spvalue().wrap());
        let eq3 = NEQ(false.to_spvalue().wrap(), hej);
        let or = OR(vec![eq2, NOT(Box::new(eq3))]);
        assert_eq!(pred_parser::pred(eq1, &s), Ok(or));

        let impl1 = " var:smart == TRUE ->  var:alive == FALSE || TRUE  ";
        let hej = s.get_assignment("smart", "t").var.wrap();
        let hopp = s.get_assignment("alive", "t").var.wrap();
        let eq1 = EQ(hej, true.to_spvalue().wrap());
        let eq2 = EQ(hopp, false.to_spvalue().wrap());
        let impl2 = OR(vec![NOT(Box::new(eq1)), OR(vec![eq2, TRUE])]);
        assert_eq!(pred_parser::pred(impl1, &s), Ok(impl2.clone()));
        let impl1 = "var:smart == TRUE -> (var:alive == FALSE || TRUE)";
        assert_eq!(pred_parser::pred(impl1, &s), Ok(impl2));
    }

    // #[test]
    // fn parse_actions() {
    //     let s = State::from_vec(&john_doe());
    //     let weight = fv!("weight");
    //     let weight_2 = fv!("weight_2");
    //     let s_new = s.add(SPAssignment::new(weight_2, 85.0.to_spvalue()));
    //     let a1 = a!(weight.clone(), 82.5.wrap());
    //     let _a2 = a!(weight.clone(), 85.0.wrap());
    //     assert_eq!(pred_parser::action("var:weight <- 82.5", &s), Ok(a1));
    //     assert_eq!(
    //         pred_parser::action("var:weight <- var:weight_2", &s_new),
    //         Ok(Action {
    //             var: SPVariable {
    //                 name: "weight".to_string(),
    //                 value_type: SPValueType::Float64,
    //             },
    //             var_or_val: SPVariable {
    //                 name: "weight_2".to_string(),
    //                 value_type: SPValueType::Float64,
    //             }
    //             .wrap()
    //         })
    //     );

    //     let counter_var = SPVariable {
    //         name: "counter".to_string(),
    //         value_type: SPValueType::Int64,
    //     };

    //     let counter_var_2 = SPVariable {
    //         name: "counter_2".to_string(),
    //         value_type: SPValueType::Int64,
    //     };

    //     let counter_var_f = SPVariable {
    //         name: "counter_f".to_string(),
    //         value_type: SPValueType::Float64,
    //     };

    //     let counter_var_2_f = SPVariable {
    //         name: "counter_2_f".to_string(),
    //         value_type: SPValueType::Float64,
    //     };

    //     let s_counter = s.add(SPAssignment::new(counter_var.clone(), (10i64).to_spvalue()));
    //     let s_counter = s_counter.add(SPAssignment::new(counter_var_2.clone(), (3i64).to_spvalue()));

    //     let s_counter_f = s.add(SPAssignment::new(counter_var_f.clone(), (10f64).to_spvalue()));
    //     let s_counter_f = s_counter_f.add(SPAssignment::new(counter_var_2_f.clone(), (9f64).to_spvalue()));

    //     let expected_action_1 = Action::new(
    //         counter_var.clone(),
    //         SPWrapped::SPValue((11i64).to_spvalue()),
    //     );
    //     assert_eq!(
    //         pred_parser::action("var:counter <- var:counter + 1", &s_counter),
    //         Ok(expected_action_1)
    //     );

    //     let expected_action_5 = Action::new(
    //         counter_var.clone(),
    //         SPWrapped::SPValue((15i64).to_spvalue()),
    //     );
    //     assert_eq!(
    //         pred_parser::action("var:counter <- var:counter + 5", &s_counter),
    //         Ok(expected_action_5)
    //     );

    //     let expected_action_5f = Action::new(
    //         counter_var_f.clone(),
    //         SPWrapped::SPValue((15f64).to_spvalue()),
    //     );
    //     assert_eq!(
    //         pred_parser::action("var:counter_f <- var:counter_f + 5.0", &s_counter_f),
    //         Ok(expected_action_5f)
    //     );

    //     let a3 = pred_parser::action("var:counter <- var:counter + 5", &s_counter).unwrap();
    //     let a4 = pred_parser::action("var:counter_f <- var:counter_f + 7.0", &s_counter_f).unwrap();

    //     let a5 = pred_parser::action("var:counter <- var:counter + var:counter_2", &s_counter).unwrap();
    //     let a6 = pred_parser::action("var:counter_f <- var:counter_f + var:counter_2_f", &s_counter_f).unwrap();

    //     let s_next_1 = a3.assign(&s_counter, "t");
    //     assert_eq!(s_next_1.get_value("counter", "t"), Some(15.to_spvalue()));

    //     let s_next_2 = a4.assign(&s_counter_f, "t");
    //     assert_eq!(s_next_2.get_value("counter_f", "t"), Some(17.0.to_spvalue()));

    //     let s_next_3 = a5.assign(&s_counter, "t");
    //     assert_eq!(s_next_3.get_value("counter", "t"), Some(13.to_spvalue()));

    //     let s_next_4 = a6.assign(&s_counter_f, "t");
    //     assert_eq!(s_next_4.get_value("counter_f", "t"), Some(19.0.to_spvalue()));

    // }

    #[test]
    fn parse_basic_actions() {
        let s = State::from_vec(&john_doe());
        let weight = fv!("weight");
        let weight_2 = fv!("weight_2");
        let s_new = s.add(SPAssignment::new(weight_2, 85.0.to_spvalue()));
        let a1 = a!(weight.clone(), 82.5.wrap());
        let _a2 = a!(weight.clone(), 85.0.wrap());
        assert_eq!(pred_parser::action("var:weight <- 82.5", &s), Ok(a1));
        assert_eq!(
            pred_parser::action("var:weight <- var:weight_2", &s_new),
            Ok(Action {
                var: SPVariable {
                    name: "weight".to_string(),
                    value_type: SPValueType::Float64,
                },
                var_or_val: SPVariable {
                    name: "weight_2".to_string(),
                    value_type: SPValueType::Float64,
                }
                .wrap()
            })
        );
    }

    #[test]
    fn parse_increment_actions() {
        let s = State::from_vec(&john_doe());
        let counter_var = SPVariable {
            name: "counter".to_string(),
            value_type: SPValueType::Int64,
        };
        let counter_var_2 = SPVariable {
            name: "counter_2".to_string(),
            value_type: SPValueType::Int64,
        };
        let counter_var_f = SPVariable {
            name: "counter_f".to_string(),
            value_type: SPValueType::Float64,
        };
        let counter_var_2_f = SPVariable {
            name: "counter_2_f".to_string(),
            value_type: SPValueType::Float64,
        };
        let i_unknown_var = SPVariable {
            name: "i_unknown".to_string(),
            value_type: SPValueType::Int64,
        };

        let s_counter = s
            .add(SPAssignment::new(counter_var.clone(), (10i64).to_spvalue()))
            .add(SPAssignment::new(
                counter_var_2.clone(),
                (3i64).to_spvalue(),
            ))
            .add(SPAssignment::new(
                i_unknown_var.clone(),
                SPValue::Int64(IntOrUnknown::UNKNOWN),
            ));

        let s_counter_f = s
            .add(SPAssignment::new(
                counter_var_f.clone(),
                (10.0f64).to_spvalue(),
            ))
            .add(SPAssignment::new(
                counter_var_2_f.clone(),
                (9.0f64).to_spvalue(),
            ));

        // --- Test Parsing (Resulting Action object) ---
        let expected_action_1 = Action::new(
            counter_var.clone(),
            SPWrapped::SPValue((11i64).to_spvalue()),
        );
        assert_eq!(
            pred_parser::action("var:counter <- var:counter + 1", &s_counter),
            Ok(expected_action_1)
        );

        let expected_action_5f = Action::new(
            counter_var_f.clone(),
            SPWrapped::SPValue((15.0f64).to_spvalue()),
        );
        assert_eq!(
            pred_parser::action("var:counter_f <- var:counter_f + 5.0", &s_counter_f),
            Ok(expected_action_5f)
        );

        let expected_action_var_i = Action::new(
            counter_var.clone(),
            SPWrapped::SPValue((13i64).to_spvalue()),
        );
        assert_eq!(
            pred_parser::action("var:counter <- var:counter + var:counter_2", &s_counter),
            Ok(expected_action_var_i)
        );

        let expected_action_var_f = Action::new(
            counter_var_f.clone(),
            SPWrapped::SPValue((19.0f64).to_spvalue()),
        );
        assert_eq!(
            pred_parser::action(
                "var:counter_f <- var:counter_f + var:counter_2_f",
                &s_counter_f
            ),
            Ok(expected_action_var_f)
        );

        // --- Test Execution (Resulting State value) ---
        let a3 = pred_parser::action("var:counter <- var:counter + 5", &s_counter).unwrap();
        let s_next_1 = a3.assign(&s_counter, "t");
        assert_eq!(s_next_1.get_value("counter", "t"), Some(15.to_spvalue()));

        let a4 = pred_parser::action("var:counter_f <- var:counter_f + 7.0", &s_counter_f).unwrap();
        let s_next_2 = a4.assign(&s_counter_f, "t");
        assert_eq!(
            s_next_2.get_value("counter_f", "t"),
            Some(17.0.to_spvalue())
        );

        // --- Test UNKNOWN (as 0) ---
        // UNKNOWN_var (0) + Int (10) = 10
        let expected_unknown_1 = Action::new(
            i_unknown_var.clone(),
            SPWrapped::SPValue((10i64).to_spvalue()),
        );
        assert_eq!(
            pred_parser::action("var:i_unknown <- var:i_unknown + var:counter", &s_counter),
            Ok(expected_unknown_1)
        );

        // Int (10) + UNKNOWN_literal (0) = 10
        let expected_unknown_2 = Action::new(
            counter_var.clone(),
            SPWrapped::SPValue((10i64).to_spvalue()),
        );
        assert_eq!(
            pred_parser::action("var:counter <- var:counter + UNKNOWN_int", &s_counter),
            Ok(expected_unknown_2)
        );
    }

    #[test]
    fn parse_decrement_actions() {
        let s = State::from_vec(&john_doe());
        let counter_var = SPVariable {
            name: "counter".to_string(),
            value_type: SPValueType::Int64,
        };
        let counter_var_2 = SPVariable {
            name: "counter_2".to_string(),
            value_type: SPValueType::Int64,
        };
        let counter_var_f = SPVariable {
            name: "counter_f".to_string(),
            value_type: SPValueType::Float64,
        };
        let counter_var_2_f = SPVariable {
            name: "counter_2_f".to_string(),
            value_type: SPValueType::Float64,
        };
        let i_unknown_var = SPVariable {
            name: "i_unknown".to_string(),
            value_type: SPValueType::Int64,
        };

        let s_counter = s
            .add(SPAssignment::new(counter_var.clone(), (10i64).to_spvalue()))
            .add(SPAssignment::new(
                counter_var_2.clone(),
                (3i64).to_spvalue(),
            ))
            .add(SPAssignment::new(
                i_unknown_var.clone(),
                SPValue::Int64(IntOrUnknown::UNKNOWN),
            ));

        let s_counter_f = s
            .add(SPAssignment::new(
                counter_var_f.clone(),
                (10.0f64).to_spvalue(),
            ))
            .add(SPAssignment::new(
                counter_var_2_f.clone(),
                (9.0f64).to_spvalue(),
            ));

        // --- Test Parsing (Resulting Action object) ---
        // Int - literal (10 - 1 = 9)
        let expected_action_1 =
            Action::new(counter_var.clone(), SPWrapped::SPValue((9i64).to_spvalue()));
        assert_eq!(
            pred_parser::action("var:counter <- var:counter - 1", &s_counter),
            Ok(expected_action_1)
        );

        // Float - literal (10.0 - 1.5 = 8.5)
        let expected_action_f = Action::new(
            counter_var_f.clone(),
            SPWrapped::SPValue((8.5f64).to_spvalue()),
        );
        assert_eq!(
            pred_parser::action("var:counter_f <- var:counter_f - 1.5", &s_counter_f),
            Ok(expected_action_f)
        );

        // Int - variable (10 - 3 = 7)
        let expected_action_var_i =
            Action::new(counter_var.clone(), SPWrapped::SPValue((7i64).to_spvalue()));
        assert_eq!(
            pred_parser::action("var:counter <- var:counter - var:counter_2", &s_counter),
            Ok(expected_action_var_i)
        );

        // Float - variable (10.0 - 9.0 = 1.0)
        let expected_action_var_f = Action::new(
            counter_var_f.clone(),
            SPWrapped::SPValue((1.0f64).to_spvalue()),
        );
        assert_eq!(
            pred_parser::action(
                "var:counter_f <- var:counter_f - var:counter_2_f",
                &s_counter_f
            ),
            Ok(expected_action_var_f)
        );

        // --- Test Execution (Resulting State value) ---
        let a1 = pred_parser::action("var:counter <- var:counter - 5", &s_counter).unwrap(); // 10 - 5 = 5
        let s_next_1 = a1.assign(&s_counter, "t");
        assert_eq!(s_next_1.get_value("counter", "t"), Some(5.to_spvalue()));

        let a2 = pred_parser::action("var:counter_f <- var:counter_f - 2.5", &s_counter_f).unwrap(); // 10.0 - 2.5 = 7.5
        let s_next_2 = a2.assign(&s_counter_f, "t");
        assert_eq!(s_next_2.get_value("counter_f", "t"), Some(7.5.to_spvalue()));

        // --- Test UNKNOWN (as 0) ---
        // UNKNOWN_var (0) - Int (10) = -10
        let expected_unknown_1 = Action::new(
            i_unknown_var.clone(),
            SPWrapped::SPValue((-10i64).to_spvalue()),
        );
        assert_eq!(
            pred_parser::action("var:i_unknown <- var:i_unknown - var:counter", &s_counter),
            Ok(expected_unknown_1)
        );

        // Int (10) - UNKNOWN_var (0) = 10
        let expected_unknown_2 = Action::new(
            counter_var.clone(),
            SPWrapped::SPValue((10i64).to_spvalue()),
        );
        assert_eq!(
            pred_parser::action("var:counter <- var:counter - var:i_unknown", &s_counter),
            Ok(expected_unknown_2)
        );
    }
}
