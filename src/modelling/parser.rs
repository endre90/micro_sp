// use crate::{Action, Predicate, SPValue, SPVariable, SPWrapped, State, ToSPValue, ToSPWrapped};
use crate::*;

peg::parser!(pub grammar pred_parser() for str {

    rule _() =  quiet!{[' ' | '\t']*}

    pub rule variable(state: &State) -> SPVariable =
    "var:" _ n:$(['a'..='z' | 'A'..='Z' | '0'..='9' | '_' | '/']+) {
        state.get_assignment(n).var
    }

    pub rule value(state: &State) -> SPWrapped
        = _ var:variable(&state) _ { SPWrapped::SPVariable(var) }
        / _ "UNKNOWN" _ { SPWrapped::SPValue(SPValue::UNKNOWN) }
        / _ "true" _ { SPWrapped::SPValue(true.to_spvalue()) }
        / _ "TRUE" _ { SPWrapped::SPValue(true.to_spvalue()) }
        / _ "false" _ { SPWrapped::SPValue(false.to_spvalue()) }
        / _ "FALSE" _ { SPWrapped::SPValue(false.to_spvalue()) }
        / _ n:$(['a'..='z' | 'A'..='Z' | '_']+) _ { SPWrapped::SPValue(n.to_spvalue()) }
        / _ "\"" n:$(!['"'] [_])* "\"" _ {
            SPWrapped::SPValue(n.into_iter().collect::<Vec<_>>().join("").to_spvalue())
        }
        / _ n:$(['0'..='9']+ "." ['0'..='9']+) _ {
            let f: f64 = n.parse().unwrap();
            SPWrapped::SPValue(f.to_spvalue())
        }
        / _ n:$(['0'..='9']+) _ {
            let i: i64 = n.parse().unwrap();
            SPWrapped::SPValue(i.to_spvalue())
        }
        // this is not tested, probably doesn't work
        / _ var:variable(&state) "+" _ n:$(['0'..='9']+) {
            let i: i64 = n.parse().unwrap();
            let new_val = match state.get_value(&var.name) {
                SPValue::Int64(val) => (val + i).to_spvalue(),
                _ => panic!("Can't increment non-integer variable")
            };
            SPWrapped::SPValue(new_val)
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
    // = p1:variable(&state) _ "<-" _ p2:variable(&state) { Action::new(p1, state.get_value(&p2.name).wrap()) }
    = p1:variable(&state) _ "<-" _ p2:variable(&state) { Action::new(p1, p2.wrap()) }
    / p1:variable(&state) _ "<-" _ p2:value(&state) { Action::new(p1, p2) }
});

#[cfg(test)]
mod tests {

    use ordered_float::OrderedFloat;

    use crate::{Predicate::*, *};

    fn john_doe() -> Vec<(SPVariable, SPValue)> {
        let name = v!("name");
        let surname = v!("surname");
        let height = iv!("height");
        let weight = fv!("weight");
        let smart = bv!("smart");
        let alive = bv!("alive");

        vec![
            (name, "John".to_spvalue()),
            (surname, "Doe".to_spvalue()),
            (height, 185.to_spvalue()),
            (weight, 80.0.to_spvalue()),
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
        assert_eq!(
            pred_parser::value("hej", &s),
            Ok(SPWrapped::SPValue("hej".to_spvalue()))
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
            pred_parser::value("UNKNOWN", &s),
            Ok(SPWrapped::SPValue(SPValue::UNKNOWN))
        );
    }

    #[test]
    fn parse_variables() {
        let s = State::from_vec(&john_doe());
        assert_eq!(
            pred_parser::variable("var: height", &s),
            Ok(s.get_assignment("height").var)
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
        let hej = s.get_assignment("smart").var.wrap();
        let eq2 = EQ(hej.clone(), true.to_spvalue().wrap());
        let eq3 = NEQ(false.to_spvalue().wrap(), hej);
        let or = OR(vec![eq2, NOT(Box::new(eq3))]);
        assert_eq!(pred_parser::pred(eq1, &s), Ok(or));

        let impl1 = " var:smart == TRUE ->  var:alive == FALSE || TRUE  ";
        let hej = s.get_assignment("smart").var.wrap();
        let hopp = s.get_assignment("alive").var.wrap();
        let eq1 = EQ(hej, true.to_spvalue().wrap());
        let eq2 = EQ(hopp, false.to_spvalue().wrap());
        let impl2 = OR(vec![NOT(Box::new(eq1)), OR(vec![eq2, TRUE])]);
        assert_eq!(pred_parser::pred(impl1, &s), Ok(impl2.clone()));
        let impl1 = "var:smart == TRUE -> (var:alive == FALSE || TRUE)";
        assert_eq!(pred_parser::pred(impl1, &s), Ok(impl2));
    }

    #[test]
    fn parse_actions() {
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
                    domain: [].to_vec()
                },
                var_or_val: SPVariable {
                    name: "weight_2".to_string(),
                    value_type: SPValueType::Float64,
                    domain: vec!()
                }
                .wrap()
            })
        );
    }
}
