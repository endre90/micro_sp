#[macro_export]
macro_rules! s {
    ($a:expr) => {
        State::from_vec(
            &($a.iter()
                .map(|(var, val)| (var.clone().clone(), val.clone()))
                .collect::<Vec<(SPVariable, SPValue)>>()),
        )
    };
}
