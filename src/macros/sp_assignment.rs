#[macro_export]
macro_rules! assign {
    ($a:expr, $b:expr) => {
        SPAssignment::new($a, $b)
    };
}
