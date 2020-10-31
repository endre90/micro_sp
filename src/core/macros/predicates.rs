use super::*;

#[macro_export]
macro_rules! ptrue {
    () => {
        Predicate::TRUE
    };
}

#[macro_export]
macro_rules! pfalse {
    () => {
        Predicate::FALSE
    };
}

#[macro_export]
macro_rules! pnot {
    ($x:expr) => {
        Predicate::NOT(Box::new($x.to_owned()))
    };
}

#[macro_export]
macro_rules! pand {
    ( $( $x:expr ),* ) => {
        {
            let mut temp_vec = Vec::new();
            $(
                temp_vec.push($x.to_owned());
            )*
            Predicate::AND(temp_vec)
        }
    };
}

#[macro_export]
macro_rules! por {
    ( $( $x:expr ),* ) => {
        {
            let mut temp_vec = Vec::new();
            $(
                temp_vec.push($x.to_owned());
            )*
            Predicate::OR(temp_vec)
        }
    };
}

#[macro_export]
macro_rules! pass {
    ($x:expr) => {
        Predicate::ASS($x.to_owned())
    };
}

#[macro_export]
macro_rules! peq {
    ($x:expr, $y:expr) => {
        Predicate::EQ($x, $y)
    };
}

#[macro_export]
macro_rules! ppbeq {
    ( $n:expr, $( $x:expr ),* ) => {
        {
            let mut temp_vec = Vec::new();
            $(
                temp_vec.push($x.to_owned());
            )*
            Predicate::PBEQ(temp_vec, $n)
        }
    };
}