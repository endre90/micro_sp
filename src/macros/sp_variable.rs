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