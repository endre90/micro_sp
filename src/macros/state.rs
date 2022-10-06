#[macro_export]
macro_rules! s {
    ($a:expr) => {
        State::new(&HashMap::from($a))
        // Action::new($a, $b)
    };
}

// let s = State::new(&HashMap::from([(pos.clone(), "a".to_spval()), (stat.clone(), "off".to_spval())]));