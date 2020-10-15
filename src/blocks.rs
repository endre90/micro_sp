// (define (domain BLOCKS)
//   (:requirements :strips)
//   (:predicates (on ?x ?y)
// 	       (ontable ?x)
// 	       (clear ?x)
// 	       (handempty)
// 	       (holding ?x)
// 	       )

//   (:action pick-up
// 	     :parameters (?x)
// 	     :precondition (and (clear ?x) (ontable ?x) (handempty))
// 	     :effect
// 	     (and (not (ontable ?x))
// 		   (not (clear ?x))
// 		   (not (handempty))
// 		   (holding ?x)))

//   (:action put-down
// 	     :parameters (?x)
// 	     :precondition (holding ?x)
// 	     :effect
// 	     (and (not (holding ?x))
// 		   (clear ?x)
// 		   (handempty)
// 		   (ontable ?x)))
//   (:action stack
// 	     :parameters (?x ?y)
// 	     :precondition (and (holding ?x) (clear ?y))
// 	     :effect
// 	     (and (not (holding ?x))
// 		   (not (clear ?y))
// 		   (clear ?x)
// 		   (handempty)
// 		   (on ?x ?y)))
//   (:action unstack
// 	     :parameters (?x ?y)
// 	     :precondition (and (on ?x ?y) (clear ?x) (handempty))
// 	     :effect
// 	     (and (holding ?x)
// 		   (clear ?y)
// 		   (not (clear ?x))
// 		   (not (handempty))
// 		   (not (on ?x ?y)))))

use super::*;

pub fn blocks_model() -> PlanningProblem {
    let domain = vec!["true", "false"];
    let boolean = "boolean";
    let hand = EnumVariable::new("hand", &domain, boolean, None, &Kind::Command);
    let hand_empty = Predicate::EQ(EnumValue::new(&hand, "true", None));
    let hand_full = Predicate::EQ(EnumValue::new(&hand, "false", None));

    let blocks = vec!["A", "B", "C", "D", "E", "F", "G", "H", "I", "J"];
    let mut pick_up_transitions = vec![];
    let mut put_down_transitions = vec![];
    let mut stack_transitions = vec![];
    let mut unstack_transitions = vec![];

    for block in &blocks {
        pick_up_transitions.push(Transition::new(
            &format!("pick_up_{}", block),
            &Predicate::AND(vec![
                hand_empty.clone(),
                Predicate::EQ(EnumValue::new(
                    &EnumVariable::new(
                        &format!("clear_{}", block),
                        &domain,
                        boolean,
                        None,
                        &Kind::Command,
                    ),
                    "true",
                    None,
                )),
                Predicate::EQ(EnumValue::new(
                    &EnumVariable::new(
                        &format!("ontable_{}", block),
                        &domain,
                        boolean,
                        None,
                        &Kind::Command,
                    ),
                    "true",
                    None,
                )),
            ]),
            &Predicate::AND(vec![
                hand_full.clone(),
                Predicate::EQ(EnumValue::new(
                    &EnumVariable::new(
                        &format!("clear_{}", block),
                        &domain,
                        boolean,
                        None,
                        &Kind::Command,
                    ),
                    "false",
                    None,
                )),
                Predicate::EQ(EnumValue::new(
                    &EnumVariable::new(
                        &format!("ontable_{}", block),
                        &domain,
                        boolean,
                        None,
                        &Kind::Command,
                    ),
                    "false",
                    None,
                )),
                Predicate::EQ(EnumValue::new(
                    &EnumVariable::new(
                        &format!("holding_{}", block),
                        &domain,
                        boolean,
                        None,
                        &Kind::Command,
                    ),
                    "true",
                    None,
                )),
            ]),
        ))
    }

    for block in &blocks {
        put_down_transitions.push(Transition::new(
            &format!("put_down_{}", block),
            &Predicate::EQ(EnumValue::new(
                &EnumVariable::new(
                    &format!("holding_{}", block),
                    &domain,
                    boolean,
                    None,
                    &Kind::Command,
                ),
                "true",
                None,
            )),
            &Predicate::AND(vec![
                hand_empty.clone(),
                Predicate::EQ(EnumValue::new(
                    &EnumVariable::new(
                        &format!("clear_{}", block),
                        &domain,
                        boolean,
                        None,
                        &Kind::Command,
                    ),
                    "true",
                    None,
                )),
                Predicate::EQ(EnumValue::new(
                    &EnumVariable::new(
                        &format!("ontable_{}", block),
                        &domain,
                        boolean,
                        None,
                        &Kind::Command,
                    ),
                    "true",
                    None,
                )),
                Predicate::EQ(EnumValue::new(
                    &EnumVariable::new(
                        &format!("holding_{}", block),
                        &domain,
                        boolean,
                        None,
                        &Kind::Command,
                    ),
                    "false",
                    None,
                )),
            ]),
        ))
    }

    for b1 in &blocks {
        for b2 in &blocks {
            stack_transitions.push(Transition::new(
                &format!("stack_{}_on_{}", b1, b2),
                &Predicate::AND(vec![
                    Predicate::EQ(EnumValue::new(
                        &EnumVariable::new(
                            &format!("clear_{}", b2),
                            &domain,
                            boolean,
                            None,
                            &Kind::Command,
                        ),
                        "true",
                        None,
                    )),
                    Predicate::EQ(EnumValue::new(
                        &EnumVariable::new(
                            &format!("holding_{}", b1),
                            &domain,
                            boolean,
                            None,
                            &Kind::Command,
                        ),
                        "true",
                        None,
                    )),
                ]),
                &Predicate::AND(vec![
                    hand_empty.clone(),
                    Predicate::EQ(EnumValue::new(
                        &EnumVariable::new(
                            &format!("clear_{}", b2),
                            &domain,
                            boolean,
                            None,
                            &Kind::Command,
                        ),
                        "false",
                        None,
                    )),
                    Predicate::EQ(EnumValue::new(
                        &EnumVariable::new(
                            &format!("holding_{}", b1),
                            &domain,
                            boolean,
                            None,
                            &Kind::Command,
                        ),
                        "false",
                        None,
                    )),
                    Predicate::EQ(EnumValue::new(
                        &EnumVariable::new(
                            &format!("clear_{}", b1),
                            &domain,
                            boolean,
                            None,
                            &Kind::Command,
                        ),
                        "true",
                        None,
                    )),
                    Predicate::EQ(EnumValue::new(
                        &EnumVariable::new(
                            &format!("{}_on_{}", b1, b2),
                            &domain,
                            boolean,
                            None,
                            &Kind::Command,
                        ),
                        "true",
                        None,
                    )),
                ]),
            ))
        }
    }

    for b1 in &blocks {
        for b2 in &blocks {
            stack_transitions.push(Transition::new(
                &format!("unstack_{}_from_{}", b1, b2),
                &Predicate::AND(vec![
                    hand_empty.clone(),
                    Predicate::EQ(EnumValue::new(
                        &EnumVariable::new(
                            &format!("{}_on_{}", b1, b2),
                            &domain,
                            boolean,
                            None,
                            &Kind::Command,
                        ),
                        "true",
                        None,
                    )),
                    Predicate::EQ(EnumValue::new(
                        &EnumVariable::new(
                            &format!("clear_{}", b1),
                            &domain,
                            boolean,
                            None,
                            &Kind::Command,
                        ),
                        "true",
                        None,
                    )),
                ]),
                &Predicate::AND(vec![
                    hand_full.clone(),
                    Predicate::EQ(EnumValue::new(
                        &EnumVariable::new(
                            &format!("clear_{}", b2),
                            &domain,
                            boolean,
                            None,
                            &Kind::Command,
                        ),
                        "true",
                        None,
                    )),
                    Predicate::EQ(EnumValue::new(
                        &EnumVariable::new(
                            &format!("clear_{}", b1),
                            &domain,
                            boolean,
                            None,
                            &Kind::Command,
                        ),
                        "false",
                        None,
                    )),
                    Predicate::EQ(EnumValue::new(
                        &EnumVariable::new(
                            &format!("holding_{}", b1),
                            &domain,
                            boolean,
                            None,
                            &Kind::Command,
                        ),
                        "true",
                        None,
                    )),
                    Predicate::EQ(EnumValue::new(
                        &EnumVariable::new(
                            &format!("{}_on_{}", b1, b2),
                            &domain,
                            boolean,
                            None,
                            &Kind::Command,
                        ),
                        "false",
                        None,
                    )),
                ]),
            ))
        }
    }

    let mut clear_predicates = vec![];
    for x in vec!["C", "F"] {
        clear_predicates.push(Predicate::EQ(EnumValue::new(
            &EnumVariable::new(
                &format!("clear_{}", x),
                &domain,
                boolean,
                None,
                &Kind::Command,
            ),
            "true",
            None,
        )))
    }

    let mut ontable_predicates = vec![];
    for x in vec!["I", "F"] {
        ontable_predicates.push(Predicate::EQ(EnumValue::new(
            &EnumVariable::new(
                &format!("ontable_{}", x),
                &domain,
                boolean,
                None,
                &Kind::Command,
            ),
            "true",
            None,
        )))
    }

    let mut on_predicates = vec![];
    for (b1, b2) in vec![
        ("C", "E"),
        ("E", "J"),
        ("J", "B"),
        ("B", "G"),
        ("G", "H"),
        ("H", "A"),
        ("A", "D"),
        ("D", "I"),
    ] {
        on_predicates.push(Predicate::EQ(EnumValue::new(
            &EnumVariable::new(
                &format!("{}_on_{}", b2, b1),
                &domain,
                boolean,
                None,
                &Kind::Command,
            ),
            "true",
            None,
        )))
    }

    let initial = Predicate::AND(vec![
        Predicate::AND(clear_predicates),
        Predicate::AND(ontable_predicates),
        Predicate::AND(on_predicates),
        hand_empty,
    ]);

    let mut goal_on_predicates = vec![];
    for (b1, b2) in vec![
        ("D", "C"),
        ("C", "F"),
        ("F", "J"),
        ("J", "E"),
        ("E", "H"),
        ("H", "B"),
        ("B", "A"),
        ("A", "G"),
        ("G", "I"),
    ] {
        goal_on_predicates.push(Predicate::EQ(EnumValue::new(
            &EnumVariable::new(
                &format!("{}_on_{}", b2, b1),
                &domain,
                boolean,
                None,
                &Kind::Command,
            ),
            "true",
            None,
        )))
    }

    let mut transitions = vec![];
    for t in vec![
        pick_up_transitions,
        put_down_transitions,
        stack_transitions,
        unstack_transitions,
    ] {
        transitions.extend(t)
    }

    let goal = Predicate::AND(goal_on_predicates);
    let problem = PlanningProblem::new(
        "blocks_world",
        &initial,
        &goal,
        &transitions,
        &50,
        &Paradigm::Raar,
    );

    problem
}
