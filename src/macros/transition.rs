#[macro_export]
macro_rules! t_plan {
    ($a:expr, $b:expr, $c:expr) => {
        Transition::new(
            $a,
            $b.clone(),
            Predicate::TRUE,
            $c.iter().map(|x| x.to_owned()).collect::<Vec<Action>>(),
            Vec::<Action>::new()
        )
    };
}

#[macro_export]
macro_rules! t {
    ($name:expr, $guard:expr, $runner_guard:expr, $actions:expr, $runner_actions: expr, $state:expr) => {
        Transition::new(
            $name,
            pred_parser::pred($guard.clone(), $state).unwrap(),
            pred_parser::pred($runner_guard.clone(), $state).unwrap(),
            $actions
                .iter()
                .map(|action| pred_parser::action(action.to_owned(), $state).unwrap())
                .collect::<Vec<Action>>(),
            $runner_actions
                .iter()
                .map(|action| pred_parser::action(action.to_owned(), $state).unwrap())
                .collect::<Vec<Action>>(),
        )
    };
}