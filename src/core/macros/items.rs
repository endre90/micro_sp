use super::*;

/// a new boolean variable of command kind
#[macro_export]
macro_rules! bool_c {
    ($name:expr, $param:expr) => {
        Variable::new(
            $name,
            &SPValueType::Bool,
            &vec![true.to_spvalue(), false.to_spvalue()],
            $param,
            None,
            Some(&Kind::Command),
        )
    };
}

/// a new boolean variable of measured kind
#[macro_export]
macro_rules! bool_m {
    ($name:expr, $param:expr) => {
        Variable::new(
            $name,
            &SPValueType::Bool,
            &vec![true.to_spvalue(), false.to_spvalue()],
            $param,
            None,
            Some(&Kind::Measured),
        )
    };
}

/// a new boolean variable of estimated kind
#[macro_export]
macro_rules! bool_e {
    ($name:expr, $param:expr) => {
        Variable::new(
            $name,
            &SPValueType::Bool,
            &vec![true.to_spvalue(), false.to_spvalue()],
            $param,
            None,
            Some(&Kind::Estimated),
        )
    };
}