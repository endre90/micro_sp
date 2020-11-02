use super::*;

/// make a new boolean variable of command kind
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
    ($name:expr, ($pvar:expr, $pval:expr)) => {
        Variable::new(
            $name,
            &SPValueType::Bool,
            &vec![true.to_spvalue(), false.to_spvalue()],
            Some(&Parameter::new(&$pvar, &$pval)),
            None,
            Some(&Kind::Command),
        )
    };
}

/// make a new boolean variable of measured kind
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
    ($name:expr, ($pvar:expr, $pval:expr)) => {
        Variable::new(
            $name,
            &SPValueType::Bool,
            &vec![true.to_spvalue(), false.to_spvalue()],
            Some(&Parameter::new(&$pvar, &$pval)),
            None,
            Some(&Kind::Measured),
        )
    };
}

/// make a new boolean variable of estimated kind
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
    ($name:expr, ($pvar:expr, $pval:expr)) => {
        Variable::new(
            $name,
            &SPValueType::Bool,
            &vec![true.to_spvalue(), false.to_spvalue()],
            Some(&Parameter::new(&$pvar, &$pval)),
            None,
            Some(&Kind::Estimated),
        )
    };
}

/// assign a value to a boolean variable
#[macro_export]
macro_rules! bool_assign {
    ($var:expr, $val:expr) => {
        Assignment::new(&$var, &$val.to_spvalue(), None)
    };
    ($var:expr, $val:expr, $life:expr) => {
        Assignment::new(&$var, &$val.to_spvalue(), Some(&$life))
    };
}

/// make a new boolean variable of command kind and assign a value to it
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
    ($name:expr, $val:expr, ($pvar:expr, $pval:expr)) => {
        Assignment::new(
            &Variable::new(
                $name,
                &SPValueType::Bool,
                &vec![true.to_spvalue(), false.to_spvalue()],
                Some(&Parameter::new(&$pvar, &$pval)),
                None,
                Some(&Kind::Command),
            ),
            &$val.to_spvalue(),
            None,
        )
    };
    ($name:expr, $val:expr, ($pvar:expr, $pval:expr), $life:expr) => {
        Assignment::new(
            &Variable::new(
                $name,
                &SPValueType::Bool,
                &vec![true.to_spvalue(), false.to_spvalue()],
                Some(&Parameter::new(&$pvar, &$pval)),
                None,
                Some(&Kind::Command),
            ),
            &$val.to_spvalue(),
            Some(&$life),
        )
    };
}

/// make a new boolean variable of measured kind and assign a value to it
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
    ($name:expr, $val:expr, ($pvar:expr, $pval:expr)) => {
        Assignment::new(
            &Variable::new(
                $name,
                &SPValueType::Bool,
                &vec![true.to_spvalue(), false.to_spvalue()],
                Some(&Parameter::new(&$pvar, &$pval)),
                None,
                Some(&Kind::Measured),
            ),
            &$val.to_spvalue(),
            None,
        )
    };
    ($name:expr, $val:expr, ($pvar:expr, $pval:expr), $life:expr) => {
        Assignment::new(
            &Variable::new(
                $name,
                &SPValueType::Bool,
                &vec![true.to_spvalue(), false.to_spvalue()],
                Some(&Parameter::new(&$pvar, &$pval)),
                None,
                Some(&Kind::Measured),
            ),
            &$val.to_spvalue(),
            Some(&$life),
        )
    };
}

/// make a new boolean variable of estimated kind and assign a value to it
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
    ($name:expr, $val:expr, ($pvar:expr, $pval:expr)) => {
        Assignment::new(
            &Variable::new(
                $name,
                &SPValueType::Bool,
                &vec![true.to_spvalue(), false.to_spvalue()],
                Some(&Parameter::new(&$pvar, &$pval)),
                None,
                Some(&Kind::Estimated),
            ),
            &$val.to_spvalue(),
            None,
        )
    };
    ($name:expr, $val:expr, ($pvar:expr, $pval:expr), $life:expr) => {
        Assignment::new(
            &Variable::new(
                $name,
                &SPValueType::Bool,
                &vec![true.to_spvalue(), false.to_spvalue()],
                Some(&Parameter::new(&$pvar, &$pval)),
                None,
                Some(&Kind::Estimated),
            ),
            &$val.to_spvalue(),
            Some(&$life),
        )
    };
}

/// make a new enumeration variable of command kind
#[macro_export]
macro_rules! enum_c {
    ($name:expr, $domain:expr) => {
        Variable::new(
            $name,
            &SPValueType::String,
            &$domain
                .iter()
                .map(|x| String::from(x.to_owned()).to_spvalue())
                .collect(),
            None,
            None,
            Some(&Kind::Command),
        )
    };
    ($name:expr, $r#type:expr, $domain:expr) => {
        Variable::new(
            $name,
            &SPValueType::String,
            &$domain
                .iter()
                .map(|x| String::from(x.to_owned()).to_spvalue())
                .collect(),
            None,
            Some(&String::from($r#type)),
            Some(&Kind::Command),
        )
    };
    ($name:expr, $r#type:expr, $domain:expr, ($pvar:expr, $pval:expr)) => {
        Variable::new(
            $name,
            &SPValueType::String,
            &$domain
                .iter()
                .map(|x| String::from(x.to_owned()).to_spvalue())
                .collect(),
                Some(&Parameter::new(&$pvar, &$pval)),
            Some(&String::from($r#type)),
            Some(&Kind::Command),
        )
    };
}

/// make a new enumeration variable of measured kind
#[macro_export]
macro_rules! enum_m {
    ($name:expr, $domain:expr) => {
        Variable::new(
            $name,
            &SPValueType::String,
            &$domain
                .iter()
                .map(|x| String::from(x.to_owned()).to_spvalue())
                .collect(),
            None,
            None
            Some(&Kind::Measured),
        )
    };
    ($name:expr, $r#type:expr, $domain:expr) => {
        Variable::new(
            $name,
            &SPValueType::String,
            &$domain
                .iter()
                .map(|x| String::from(x.to_owned()).to_spvalue())
                .collect(),
            None,
            Some(&String::from($r#type)),
            Some(&Kind::Measured),
        )
    };
    ($name:expr, $r#type:expr, $domain:expr, ($pvar:expr, $pval:expr)) => {
        Variable::new(
            $name,
            &SPValueType::String,
            &$domain
                .iter()
                .map(|x| String::from(x.to_owned()).to_spvalue())
                .collect(),
                Some(&Parameter::new(&$pvar, &$pval)),
            Some(&String::from($r#type)),
            Some(&Kind::Measured),
        )
    };
}

/// make a new enumeration variable of estimated kind
#[macro_export]
macro_rules! enum_e {
    ($name:expr, $domain:expr) => {
        Variable::new(
            $name,
            &SPValueType::String,
            &$domain
                .iter()
                .map(|x| String::from(x.to_owned()).to_spvalue())
                .collect(),
            None,
            None
            Some(&Kind::Estimated),
        )
    };
    ($name:expr, $r#type:expr, $domain:expr) => {
        Variable::new(
            $name,
            &SPValueType::String,
            &$domain
                .iter()
                .map(|x| String::from(x.to_owned()).to_spvalue())
                .collect(),
            None,
            Some(&String::from($r#type)),
            Some(&Kind::Estimated),
        )
    };
    ($name:expr, $r#type:expr, $domain:expr, ($pvar:expr, $pval:expr)) => {
        Variable::new(
            $name,
            &SPValueType::String,
            &$domain
                .iter()
                .map(|x| String::from(x.to_owned()).to_spvalue())
                .collect(),
                Some(&Parameter::new(&$pvar, &$pval)),
            Some(&String::from($r#type)),
            Some(&Kind::Estimated),
        )
    };
}

/// assign a value to an enumeration variable
#[macro_export]
macro_rules! enum_assign {
    ($var:expr, $val:expr) => {
        Assignment::new(&$var, &String::from($val).to_spvalue(), None)
    };
    ($var:expr, $val:expr, $life:expr) => {
        Assignment::new(&$var, &String::from($val).to_spvalue(), Some(&$life))
    };
}

/// make a new enumeration variable of command kind and assign a value to it
#[macro_export]
macro_rules! new_enum_assign_c {
    ($name:expr, $domain:expr, $val:expr) => {
        Assignment::new(
            &Variable::new(
                $name,
                &SPValueType::String,
                &$domain
                    .iter()
                    .map(|x| String::from(x.to_owned()).to_spvalue())
                    .collect(),
                None,
                None,
                Some(&Kind::Command),
            ),
            &String::from($val.to_owned()).to_spvalue(),
            None,
        )
    };
    ($name:expr, $r#type:expr, $domain:expr, $val:expr) => {
        Assignment::new(
            &Variable::new(
                $name,
                &SPValueType::String,
                &$domain
                    .iter()
                    .map(|x| String::from(x.to_owned()).to_spvalue())
                    .collect(),
                None,
                Some(&String::from($r#type)),
                Some(&Kind::Command),
            ),
            &String::from($val.to_owned()).to_spvalue(),
            None,
        )
    };
    ($name:expr, $r#type:expr, $domain:expr, $val:expr, ($pvar:expr, $pval:expr)) => {
        Assignment::new(
            &Variable::new(
                $name,
                &SPValueType::String,
                &$domain
                    .iter()
                    .map(|x| String::from(x.to_owned()).to_spvalue())
                    .collect(),
                Some(&Parameter::new(&$pvar, &$pval)),
                Some(&String::from($r#type)),
                Some(&Kind::Command),
            ),
            &String::from($val.to_owned()).to_spvalue(),
            None,
        )
    };
    ($name:expr, $r#type:expr, $domain:expr, $val:expr, ($pvar:expr, $pval:expr), $life:expr) => {
        Assignment::new(
            &Variable::new(
                $name,
                &SPValueType::String,
                &$domain
                    .iter()
                    .map(|x| String::from(x.to_owned()).to_spvalue())
                    .collect(),
                Some(&Parameter::new(&$pvar, &$pval)),
                Some(&String::from($r#type)),
                Some(&Kind::Command),
            ),
            &String::from($val.to_owned()).to_spvalue(),
            Some(&$life),
        )
    };
}

/// make a new enumeration variable of measured kind and assign a value to it
#[macro_export]
macro_rules! new_enum_assign_m {
    ($name:expr, $domain:expr, $val:expr) => {
        Assignment::new(
            &Variable::new(
                $name,
                &SPValueType::String,
                &$domain
                    .iter()
                    .map(|x| String::from(x.to_owned()).to_spvalue())
                    .collect(),
                None,
                None,
                Some(&Kind::Measured),
            ),
            &String::from($val.to_owned()).to_spvalue(),
            None,
        )
    };
    ($name:expr, $r#type:expr, $domain:expr, $val:expr) => {
        Assignment::new(
            &Variable::new(
                $name,
                &SPValueType::String,
                &$domain
                    .iter()
                    .map(|x| String::from(x.to_owned()).to_spvalue())
                    .collect(),
                None,
                Some(&String::from($r#type)),
                Some(&Kind::Measured),
            ),
            &String::from($val.to_owned()).to_spvalue(),
            None,
        )
    };
    ($name:expr, $r#type:expr, $domain:expr, $val:expr, ($pvar:expr, $pval:expr)) => {
        Assignment::new(
            &Variable::new(
                $name,
                &SPValueType::String,
                &$domain
                    .iter()
                    .map(|x| String::from(x.to_owned()).to_spvalue())
                    .collect(),
                Some(&Parameter::new(&$pvar, &$pval)),
                Some(&String::from($r#type)),
                Some(&Kind::Measured),
            ),
            &String::from($val.to_owned()).to_spvalue(),
            None,
        )
    };
    ($name:expr, $r#type:expr, $domain:expr, $val:expr, ($pvar:expr, $pval:expr), $life:expr) => {
        Assignment::new(
            &Variable::new(
                $name,
                &SPValueType::String,
                &$domain
                    .iter()
                    .map(|x| String::from(x.to_owned()).to_spvalue())
                    .collect(),
                Some(&Parameter::new(&$pvar, &$pval)),
                Some(&String::from($r#type)),
                Some(&Kind::Measured),
            ),
            &String::from($val.to_owned()).to_spvalue(),
            Some(&$life),
        )
    };
}

/// make a new enumeration variable of estimated kind and assign a value to it
#[macro_export]
macro_rules! new_enum_assign_e {
    ($name:expr, $domain:expr, $val:expr) => {
        Assignment::new(
            &Variable::new(
                $name,
                &SPValueType::String,
                &$domain
                    .iter()
                    .map(|x| String::from(x.to_owned()).to_spvalue())
                    .collect(),
                None,
                None,
                Some(&Kind::Estimated),
            ),
            &String::from($val.to_owned()).to_spvalue(),
            None,
        )
    };
    ($name:expr, $r#type:expr, $domain:expr, $val:expr) => {
        Assignment::new(
            &Variable::new(
                $name,
                &SPValueType::String,
                &$domain
                    .iter()
                    .map(|x| String::from(x.to_owned()).to_spvalue())
                    .collect(),
                None,
                Some(&String::from($r#type)),
                Some(&Kind::Estimated),
            ),
            &String::from($val.to_owned()).to_spvalue(),
            None,
        )
    };
    ($name:expr, $r#type:expr, $domain:expr, $val:expr, ($pvar:expr, $pval:expr)) => {
        Assignment::new(
            &Variable::new(
                $name,
                &SPValueType::String,
                &$domain
                    .iter()
                    .map(|x| String::from(x.to_owned()).to_spvalue())
                    .collect(),
                Some(&Parameter::new(&$pvar, &$pval)),
                Some(&String::from($r#type)),
                Some(&Kind::Estimated),
            ),
            &String::from($val.to_owned()).to_spvalue(),
            None,
        )
    };
    ($name:expr, $r#type:expr, $domain:expr, $val:expr, ($pvar:expr, $pval:expr), $life:expr) => {
        Assignment::new(
            &Variable::new(
                $name,
                &SPValueType::String,
                &$domain
                    .iter()
                    .map(|x| String::from(x.to_owned()).to_spvalue())
                    .collect(),
                Some(&Parameter::new(&$pvar, &$pval)),
                Some(&String::from($r#type)),
                Some(&Kind::Estimated),
            ),
            &String::from($val.to_owned()).to_spvalue(),
            Some(&$life),
        )
    };
}