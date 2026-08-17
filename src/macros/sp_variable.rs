#[macro_export]
macro_rules! v {
    ($a:expr) => {
        SPVariable::new(
            $a.clone(),
            SPValueType::String,
            // vec![],
        )
    };
}

#[macro_export]
macro_rules! bv {
    ($a:expr) => {
        SPVariable::new(
            $a.clone(),
            SPValueType::Bool,
            // vec![true.to_spvalue(), false.to_spvalue()],
        )
    };
}

#[macro_export]
macro_rules! iv {
    ($a:expr) => {
        SPVariable::new(
            $a.clone(),
            SPValueType::Int64,
            // vec![],
            // $b.iter().map(|x| x.clone().to_spvalue()).collect(),
        )
    };
}

#[macro_export]
macro_rules! fv {
    ($a:expr) => {
        SPVariable::new(
            $a.clone(),
            SPValueType::Float64,
            // vec![]
        )
    };
}

#[macro_export]
macro_rules! av {
    ($a:expr) => {
        SPVariable::new(
            $a.clone(),
            SPValueType::Array,
            // vec![],
        )
    };
}

#[macro_export]
macro_rules! tv {
    ($a:expr) => {
        SPVariable::new(
            $a.clone(),
            SPValueType::Time,
            // vec![],
        )
    };
}

#[macro_export]
macro_rules! mv {
    ($a:expr) => {
        SPVariable::new(
            $a.clone(),
            SPValueType::Map,
            // vec![],
        )
    };
}

#[macro_export]
macro_rules! tfv {
    ($a:expr) => {
        SPVariable::new(
            $a.clone(),
            SPValueType::Transform,
            // vec![],
        )
    };
}

/// Each `*v!` macro is just sugar for `SPVariable::new(name, SPValueType::X)`,
/// but nothing in the crate actually calls `tv!` (the `Time`-typed one) - every
/// other variant is exercised indirectly through model/state building code
/// elsewhere, so `tv!` was the one macro invocation that never expanded during
/// any test run. Instantiate every variant once and check both the name and
/// the resulting `SPValueType`, since a copy-pasted macro (there are eight,
/// differing only in the `SPValueType` variant) is exactly the kind of code
/// where one of them silently gets the wrong type.
#[cfg(test)]
mod tests {
    use crate::*;

    #[test]
    fn each_shorthand_macro_builds_the_matching_variable_type() {
        assert_eq!(v!("x"), SPVariable::new("x", SPValueType::String));
        assert_eq!(bv!("x"), SPVariable::new("x", SPValueType::Bool));
        assert_eq!(iv!("x"), SPVariable::new("x", SPValueType::Int64));
        assert_eq!(fv!("x"), SPVariable::new("x", SPValueType::Float64));
        assert_eq!(av!("x"), SPVariable::new("x", SPValueType::Array));
        assert_eq!(mv!("x"), SPVariable::new("x", SPValueType::Map));
        assert_eq!(tfv!("x"), SPVariable::new("x", SPValueType::Transform));

        // This is the one that was actually uncovered: the Time variant.
        let time_var = tv!("x");
        assert_eq!(time_var, SPVariable::new("x", SPValueType::Time));
        assert_eq!(time_var.has_type(), SPValueType::Time);
        assert_eq!(time_var.name, "x");
    }
}