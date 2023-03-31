use crate::{Action, Predicate, SPValue, SPVariable, SPWrapped, State, ToSPValue, ToSPWrapped};

peg::parser!(pub grammar pred_parser() for str {

    rule _() =  quiet!{[' ' | '\t']*}

    pub rule variable(state: &State) -> SPVariable =
    "var:" _ n:$(['a'..='z' | 'A'..='Z' | '0'..='9' | '_' | '/']+) {
        state.get_all(n).var
    }

    pub rule value(state: &State) -> SPWrapped
        = _ var:variable(&state) _ { SPWrapped::SPVariable(var) }
        / _ "[unknown]" _ { SPWrapped::SPValue(SPValue::Unknown) }
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
            let i: i32 = n.parse().unwrap();
            SPWrapped::SPValue(i.to_spvalue())
        }
        // this is not tested, probably doesn't work
        / _ var:variable(&state) "+" _ n:$(['0'..='9']+) {
            let i: i32 = n.parse().unwrap();
            let new_val = match state.get_value(&var.name) {
                SPValue::Int32(val) => (val + i).to_spvalue(),
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
    = p1:variable(&state) _ "<-" _ p2:variable(&state) { Action::new(p1, state.get_value(&p2.name).wrap()) }
    / p1:variable(&state) _ "<-" _ p2:value(&state) { Action::new(p1, p2) }
});
