use super::*;
use crate::models::blocksworld::domain;

pub fn setup(
    model: (Vec<Transition>, Predicate),
    blocks: &Vec<&str>,
    clear_vec: &Vec<&str>,
    ontable_vec: &Vec<&str>,
    hand_empty_init: bool,
    on_init: &Vec<(&str, &str)>,
    on_goal: &Vec<(&str, &str)>,
) -> PlanningProblem {
    // explicitly have to say that others are not clear?
    let mut clear_predicates = vec![];
    let domain = vec!["true", "false"];

    let unclear_vec = IterOps::difference(blocks.clone(), clear_vec.clone());

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
    for x in ontable_vec {
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
    for (b1, b2) in on_init {
        on_predicates.push(Predicate::EQ(EnumValue::new(
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
    for (b1, b2) in on_goal {
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
