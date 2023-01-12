#[macro_export]
macro_rules! t {
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
macro_rules! t_plus {
    ($a:expr, $b:expr, $c:expr, $d:expr, $e:expr) => {
        Transition::new(
            $a,
            $b.clone(),
            $c.clone(),
            $d.iter().map(|x| x.to_owned()).collect::<Vec<Action>>(),
            $e.iter().map(|y| y.to_owned()).collect::<Vec<Action>>()
        )
    };
}
