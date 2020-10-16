use super::*;
use crate::models::blocksworld::domain;

pub fn instance_4_a() -> PlanningProblem {

    let domain = vec!["true", "false"];
    let blocks = vec!["A", "B", "C", "D"];
    let model = domain::blocksworld_model(&blocks);

    // explicitly have to say that others are not clear?
    let mut clear_predicates = vec![];
    let clear_vec = vec!["A", "B", "C", "D"];
    let unclear_vec = IterOps::difference(blocks.clone(), clear_vec.clone());
    println!("{:?}", unclear_vec);

    for x in clear_vec {
        clear_predicates.push(Predicate::EQ(EnumValue::new(
            &EnumVariable::new(
                &format!("clear_{}", x),
                &domain,
                "boolean",
                None,
                &Kind::Command,
            ),
            "true",
            None,
        )))
    }

    for x in unclear_vec {
        clear_predicates.push(Predicate::EQ(EnumValue::new(
            &EnumVariable::new(
                &format!("clear_{}", x),
                &domain,
                "boolean",
                None,
                &Kind::Command,
            ),
            "false",
            None,
        )))
    }

    let mut ontable_predicates = vec![];
    for x in vec!["A", "B", "C", "D"] {
        ontable_predicates.push(Predicate::EQ(EnumValue::new(
            &EnumVariable::new(
                &format!("ontable_{}", x),
                &domain,
                "boolean",
                None,
                &Kind::Command,
            ),
            "true",
            None,
        )))
    }

    let mut on_predicates = vec![];
    // for (b1, b2) in vec![
    //     ("A", "B")
    //     // ("C", "E"),
    //     // ("E", "J"),
    //     // ("J", "B"),
    //     // ("B", "G"),
    //     // ("G", "H"),
    //     // ("H", "A"),
    //     // ("A", "D"),
    //     // ("D", "I"),
    // ] {
    //     on_predicates.push(Predicate::EQ(EnumValue::new(
    //         &EnumVariable::new(
    //             &format!("{}_on_{}", b1, b2),
    //             &domain,
    //             "boolean",
    //             None,
    //             &Kind::Command,
    //         ),
    //         "true",
    //         None,
    //     )))
    // }

    let initial = Predicate::AND(vec![
        Predicate::AND(clear_predicates),
        Predicate::AND(ontable_predicates),
        Predicate::AND(on_predicates),
        Predicate::EQ(EnumValue::new(
            &EnumVariable::new(
                &format!("hand_empty"),
                &domain,
                "boolean",
                None,
                &Kind::Command,
            ),
            "true",
            None,
        )),
    ]);


    let mut goal_on_predicates = vec![];
    for (b1, b2) in vec![
        ("D", "C"),
        ("C", "B"),
        ("B", "A")
    ] {
        goal_on_predicates.push(Predicate::EQ(EnumValue::new(
            &EnumVariable::new(
                &format!("{}_on_{}", b1, b2),
                &domain,
                "boolean",
                None,
                &Kind::Command,
            ),
            "true",
            None,
        )))
    }

    let goal = Predicate::AND(goal_on_predicates);
    let problem = PlanningProblem::new(
        "blocks_world",
        &initial,
        &goal,
        &model.0,
        &model.1,
        &50,
        &Paradigm::Raar,
    );

    problem

}
