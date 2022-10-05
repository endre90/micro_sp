#[macro_export]
macro_rules! eq {
    ($a:expr, $b:expr) => {
        Predicate::EQ($a, $b)
    };
}

#[macro_export]
macro_rules! eq2 {
    ($a:expr, $b:expr) => {
        Predicate::EQ(SPCommon::SPVariable($a.clone()), $b)
    };
}

#[macro_export]
macro_rules! not {
    ($a:expr) => {
        Predicate::NOT(Box::new($a.clone()))
    };
}

#[macro_export]
macro_rules! and {
    ($a:expr) => {
        Predicate::AND($a.to_owned())
    };
    ($( $x:expr ),* ) => {
        {
            let mut temp_vec = Vec::new();
            $(
                temp_vec.push($x.clone());
            )*
            Predicate::AND(temp_vec)
        }
    };
}

#[macro_export]
macro_rules! or {
    ($a:expr) => {
        Predicate::OR($a.to_owned())
    };
    ($( $x:expr ),* ) => {
        {
            let mut temp_vec: Vec<Predicate> = Vec::new();
            $(
                temp_vec.push($x.clone());
            )*
            Predicate::OR(temp_vec)
        }
    };
}
