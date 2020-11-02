use super::*;

#[macro_export]
macro_rules! ppred {
    ( $( $x:expr ),* ) => {
        {
            let mut temp_vec = Vec::new();
            $(
                temp_vec.push($x.to_owned());
            )*
            ParamPredicate::new(&temp_vec)
        }
    };
}

#[macro_export]
macro_rules! ptrans {
    ($name:expr, $guard:expr, $update:expr) => {
        ParamTransition::new($name, $guard, $update)
    };
}