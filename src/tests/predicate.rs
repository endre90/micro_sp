#![allow(unused_imports)]
#![allow(dead_code)]
use micro_sp::{SPValue, State, ToSPValue, VarOrVal, Predicate, ToVal, ToVar};
use std::collections::{HashMap, HashSet};

fn john_doe() -> HashMap<String, SPValue> {
    HashMap::from([
        ("name".to_string(), "John".to_spvalue()),
        ("surname".to_string(), "Doe".to_spvalue()),
        ("height".to_string(), 185.to_spvalue()),
        ("weight".to_string(), 80.5.to_spvalue()),
        ("smart".to_string(), true.to_spvalue()),
    ])
}

#[test]
fn test_predicate_eq() {
    let s1 = State::new(john_doe());
    let s2 = State::new(john_doe());
    let eq = Predicate::EQ("name".to_var(), "name".to_var());
    let eq2 = Predicate::EQ("height".to_var(), 175.to_val());
    assert!(eq.eval(&s1));
    assert_ne!(true, eq2.eval(&s2));
}

#[test]
#[should_panic]
fn test_predicate_eq_panic() {
    let s1 = State::new(john_doe());
    let eq = Predicate::EQ("v10".to_var(), "v11".to_var());
    eq.eval(&s1);
}



// def test_guards_not():
//     """
//     Testing the class Not
//     """

//     s1 = State(v1 = False, v2 = True, v3 = "open")
//     eq = Eq("v1", False)
//     eq2 = Eq("v3", "closed")
//     assert not Not(eq).eval(s1)
//     assert Not(eq2).eval(s1)
//     eq3 = Eq("v10", "open")
//     with pytest.raises(NotInStateException) as e:
//         Not(eq3).eval(s1)

//     # these lasts tests checks the _eq__ and __hash__ implementation. 
//     assert Not(eq) == Not(eq) 
//     assert hash(Not(eq)) == hash(Not(eq))
//     assert Not(eq) != Not(eq2) 
//     assert hash(Not(eq)) != hash(Not(eq2))

// def test_guards_and():
//     """
//     Testing the class And
//     """

//     s1 = State(v1 = False, v2 = True, v3 = "open")
//     s2 = State(v1 = True, v2 = True, v3 = "open")
//     eq = Eq("v1", "v2")
//     eq2 = Eq("v3", "open")
//     eq3 = Eq("v1", True)
//     eq4 = Eq("v2", True)
//     assert not And(eq, eq2, eq3).eval(s1)
//     assert And(eq, eq2, eq3, eq4).eval(s2)

//     eq5 = Eq("v10", "open")
//     with pytest.raises(NotInStateException) as e:
//         And(eq, eq2, eq3, eq4, eq5).eval(s2)

//     # these lasts tests checks the _eq__ and __hash__ implementation. 
//     assert And(eq, eq2) == And(eq, eq2)
//     assert hash(And(eq, eq2)) == hash(And(eq, eq2))
//     assert And(eq, eq2) != And(eq, eq3)
//     assert hash(And(eq, eq2)) != hash(And(eq, eq3))

// def test_guards_or():
//     """
//     Testing the class Or
//     """

//     s1 = State(v1 = False, v2 = True, v3 = "open")
//     eq = Eq("v1", "v2")
//     eq2 = Eq("v3", "open")
//     eq3 = Eq("v1", True)
//     eq4 = Eq("v2", True)
//     eq5 = And(eq, eq2, eq3)
//     assert not Or(eq, eq3, eq5).eval(s1)
//     assert Or(eq, eq2, eq3, eq4).eval(s1)

//     eq6 = Eq("v10", "open")
//     with pytest.raises(NotInStateException) as e:
//         Or(eq, eq2, eq3, eq4, eq6).eval(s1)

//     # these lasts tests checks the _eq__ and __hash__ implementation. 
//     assert Or(eq, eq2) == Or(eq, eq2)
//     assert hash(Or(eq, eq2)) == hash(Or(eq, eq2))
//     assert Or(eq, eq2) != Or(eq, eq3)
//     assert hash(Or(eq, eq2)) != hash(Or(eq, eq3))


// def test_guards_complex():
//     """
//     ...
//     """
//     s1 = State(a = False, b = True, c = False, d = True)
//     s2 = State(a = True, b = True, c = False, d = True)
    
//     g = guards.from_str('!a && (b || c || d) && (d != False)')
//     assert g.eval(s1) and not g.eval(s2)

//     # these lasts tests checks the _eq__ and __hash__ implementation. 
//     assert g == g
//     assert hash(g) == hash(g)
//     g2 = guards.from_str('!a && (b || c || d) && (d != True)')
//     assert g != g2
//     assert hash(g) != hash(g2)