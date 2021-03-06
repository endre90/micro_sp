// use super::*;

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
    ($name:expr, $domain:expr, $r#type:expr) => {
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
    ($name:expr, $domain:expr, $r#type:expr, $param:expr) => {
        Variable::new(
            $name,
            &SPValueType::String,
            &$domain
                .iter()
                .map(|x| String::from(x.to_owned()).to_spvalue())
                .collect(),
                Some(&Parameter::new(&$param, &true)),
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
    ($name:expr, $domain:expr, $r#type:expr) => {
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
    ($name:expr, $domain:expr, $r#type:expr, $param:expr) => {
        Variable::new(
            $name,
            &SPValueType::String,
            &$domain
                .iter()
                .map(|x| String::from(x.to_owned()).to_spvalue())
                .collect(),
                Some(&Parameter::new(&$param, &true)),
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
    ($name:expr, $domain:expr, $r#type:expr) => {
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
    ($name:expr, $domain:expr, $r#type:expr, $param:expr) => {
        Variable::new(
            $name,
            &SPValueType::String,
            &$domain
                .iter()
                .map(|x| String::from(x.to_owned()).to_spvalue())
                .collect(),
                Some(&Parameter::new(&$param, &true)),
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
    ($name:expr, $domain:expr, $val:expr, $r#type:expr) => {
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
    ($name:expr, $domain:expr, $val:expr, $r#type:expr, $param:expr) => {
        Assignment::new(
            &Variable::new(
                $name,
                &SPValueType::String,
                &$domain
                    .iter()
                    .map(|x| String::from(x.to_owned()).to_spvalue())
                    .collect(),
                Some(&Parameter::new(&$param, &true)),
                Some(&String::from($r#type)),
                Some(&Kind::Command),
            ),
            &String::from($val.to_owned()).to_spvalue(),
            None,
        )
    };
    ($name:expr, $domain:expr, $val:expr, $r#type:expr, $param:expr, $life:expr) => {
        Assignment::new(
            &Variable::new(
                $name,
                &SPValueType::String,
                &$domain
                    .iter()
                    .map(|x| String::from(x.to_owned()).to_spvalue())
                    .collect(),
                Some(&Parameter::new(&$param, &true)),
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
    ($name:expr, $domain:expr, $val:expr, $r#type:expr) => {
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
    ($name:expr, $domain:expr, $val:expr, $r#type:expr, $param:expr) => {
        Assignment::new(
            &Variable::new(
                $name,
                &SPValueType::String,
                &$domain
                    .iter()
                    .map(|x| String::from(x.to_owned()).to_spvalue())
                    .collect(),
                Some(&Parameter::new(&$param, &true)),
                Some(&String::from($r#type)),
                Some(&Kind::Measured),
            ),
            &String::from($val.to_owned()).to_spvalue(),
            None,
        )
    };
    ($name:expr, $domain:expr, $val:expr, $r#type:expr, $param:expr, $life:expr) => {
        Assignment::new(
            &Variable::new(
                $name,
                &SPValueType::String,
                &$domain
                    .iter()
                    .map(|x| String::from(x.to_owned()).to_spvalue())
                    .collect(),
                Some(&Parameter::new(&$param, &true)),
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
    ($name:expr, $domain:expr, $val:expr, $r#type:expr) => {
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
    ($name:expr, $domain:expr, $val:expr, $r#type:expr, $param:expr) => {
        Assignment::new(
            &Variable::new(
                $name,
                &SPValueType::String,
                &$domain
                    .iter()
                    .map(|x| String::from(x.to_owned()).to_spvalue())
                    .collect(),
                Some(&Parameter::new(&$param, &true)),
                Some(&String::from($r#type)),
                Some(&Kind::Estimated),
            ),
            &String::from($val.to_owned()).to_spvalue(),
            None,
        )
    };
    ($name:expr, $domain:expr, $val:expr, $r#type:expr, $param:expr, $life:expr) => {
        Assignment::new(
            &Variable::new(
                $name,
                &SPValueType::String,
                &$domain
                    .iter()
                    .map(|x| String::from(x.to_owned()).to_spvalue())
                    .collect(),
                Some(&Parameter::new(&$param, &true)),
                Some(&String::from($r#type)),
                Some(&Kind::Estimated),
            ),
            &String::from($val.to_owned()).to_spvalue(),
            Some(&$life),
        )
    };
}

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