#[macro_export]
macro_rules! t {
    ($a:expr, $b:expr, $c:expr) => {
        Transition::new($a, $b.clone(), $c.iter().map(|x| x.to_owned()).collect())
    };
}
