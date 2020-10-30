use super::*;

/// a new boolean variable of command kind
#[macro_export]
macro_rules! bool_c {
    ($name:expr) => {
        Variable::new(
            $name,
            &SPValueType::Bool,
            &vec![true.to_spvalue(), false.to_spvalue()],
            None,
            None,
            Some(&Kind::Command),
        )
    };
    ($name:expr, $param:expr) => {
        Variable::new(
            $name,
            &SPValueType::Bool,
            &vec![true.to_spvalue(), false.to_spvalue()],
            Some(&Parameter::new(&$param, &true)),
            None,
            Some(&Kind::Command),
        )
    };
}

/// a new boolean variable of measured kind
#[macro_export]
macro_rules! bool_m {
    ($name:expr) => {
        Variable::new(
            $name,
            &SPValueType::Bool,
            &vec![true.to_spvalue(), false.to_spvalue()],
            None,
            None,
            Some(&Kind::Measured),
        )
    };
    ($name:expr, $param:expr) => {
        Variable::new(
            $name,
            &SPValueType::Bool,
            &vec![true.to_spvalue(), false.to_spvalue()],
            Some(&Parameter::new(&$param, &true)),
            None,
            Some(&Kind::Measured),
        )
    };
}

/// a new boolean variable of estimated kind
#[macro_export]
macro_rules! bool_e {
    ($name:expr) => {
        Variable::new(
            $name,
            &SPValueType::Bool,
            &vec![true.to_spvalue(), false.to_spvalue()],
            None,
            None,
            Some(&Kind::Estimated),
        )
    };
    ($name:expr, $param:expr) => {
        Variable::new(
            $name,
            &SPValueType::Bool,
            &vec![true.to_spvalue(), false.to_spvalue()],
            Some(&Parameter::new(&$param, &true)),
            None,
            Some(&Kind::Estimated),
        )
    };
}

/// assign a value to a variable
#[macro_export]
macro_rules! bool_assign {
    ($var:expr, $val:expr) => {
        Assignment::new(&$var, &$val.to_spvalue(), None)
    };
    ($var:expr, $val:expr, $life:expr) => {
        Assignment::new(&$var, &$val.to_spvalue(), Some(&$life))
    };
}

/// make a new variable and assign a value to it
#[macro_export]
macro_rules! new_bool_assign_c {
    ($name:expr, $val:expr) => {
        Assignment::new(
            &Variable::new(
                $name,
                &SPValueType::Bool,
                &vec![true.to_spvalue(), false.to_spvalue()],
                None,
                None,
                Some(&Kind::Command),
            ),
            &$val.to_spvalue(),
            None,
        )
    };
    ($name:expr, $val:expr, $param:expr) => {
        Assignment::new(
            &Variable::new(
                $name,
                &SPValueType::Bool,
                &vec![true.to_spvalue(), false.to_spvalue()],
                Some(&Parameter::new(&$param, &true)),
                None,
                Some(&Kind::Command),
            ),
            &$val.to_spvalue(),
            None,
        )
    };
    ($name:expr, $val:expr, $param:expr, $life:expr) => {
        Assignment::new(
            &Variable::new(
                $name,
                &SPValueType::Bool,
                &vec![true.to_spvalue(), false.to_spvalue()],
                Some(&Parameter::new(&$param, &true)),
                None,
                Some(&Kind::Command),
            ),
            &$val.to_spvalue(),
            Some(&$life),
        )
    };
}

/// make a new variable and assign a value to it
#[macro_export]
macro_rules! new_bool_assign_m {
    ($name:expr, $val:expr) => {
        Assignment::new(
            &Variable::new(
                $name,
                &SPValueType::Bool,
                &vec![true.to_spvalue(), false.to_spvalue()],
                None,
                None,
                Some(&Kind::Measured),
            ),
            &$val.to_spvalue(),
            None,
        )
    };
    ($name:expr, $val:expr, $param:expr) => {
        Assignment::new(
            &Variable::new(
                $name,
                &SPValueType::Bool,
                &vec![true.to_spvalue(), false.to_spvalue()],
                Some(&Parameter::new(&$param, &true)),
                None,
                Some(&Kind::Measured),
            ),
            &$val.to_spvalue(),
            None,
        )
    };
    ($name:expr, $val:expr, $param:expr, $life:expr) => {
        Assignment::new(
            &Variable::new(
                $name,
                &SPValueType::Bool,
                &vec![true.to_spvalue(), false.to_spvalue()],
                Some(&Parameter::new(&$param, &true)),
                None,
                Some(&Kind::Measured),
            ),
            &$val.to_spvalue(),
            Some(&$life),
        )
    };
}

/// make a new variable and assign a value to it
#[macro_export]
macro_rules! new_bool_assign_e {
    ($name:expr, $val:expr) => {
        Assignment::new(
            &Variable::new(
                $name,
                &SPValueType::Bool,
                &vec![true.to_spvalue(), false.to_spvalue()],
                None,
                None,
                Some(&Kind::Estimated),
            ),
            &$val.to_spvalue(),
            None,
        )
    };
    ($name:expr, $val:expr, $param:expr) => {
        Assignment::new(
            &Variable::new(
                $name,
                &SPValueType::Bool,
                &vec![true.to_spvalue(), false.to_spvalue()],
                Some(&Parameter::new(&$param, &true)),
                None,
                Some(&Kind::Estimated),
            ),
            &$val.to_spvalue(),
            None,
        )
    };
    ($name:expr, $val:expr, $param:expr, $life:expr) => {
        Assignment::new(
            &Variable::new(
                $name,
                &SPValueType::Bool,
                &vec![true.to_spvalue(), false.to_spvalue()],
                Some(&Parameter::new(&$param, &true)),
                None,
                Some(&Kind::Estimated),
            ),
            &$val.to_spvalue(),
            Some(&$life),
        )
    };
}
