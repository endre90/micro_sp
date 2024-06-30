// #[macro_export]
// macro_rules! v_command {
//     ($a:expr, $b:expr) => {
//         SPVariable::new(
//             $a.clone(),
//             SPVariableType::Command, 
//             SPValueType::String,
//             $b.iter().map(|x| x.clone().to_spvalue()).collect(),
//         )
//     };
// }

// // #[macro_export]
// // macro_rules! v_measured {
// //     ($a:expr, $b:expr) => {
// //         SPVariable::new(
// //             $a.clone(),
// //             SPVariableType::Measured, 
// //             SPValueType::String,
// //             $b.iter().map(|x| x.clone().to_spvalue()).collect(),
// //         )
// //     };
// // }

// #[macro_export]
// macro_rules! v_measured {
//     ($a:expr) => {
//         SPVariable::new(
//             $a.clone(),
//             SPVariableType::Measured, 
//             SPValueType::String,
//             vec![],
//             // $b.iter().map(|x| x.clone().to_spvalue()).collect(),
//         )
//     };
// }


// #[macro_export]
// macro_rules! v_estimated {
//     ($a:expr, $b:expr) => {
//         SPVariable::new(
//             $a.clone(),
//             SPVariableType::Estimated, 
//             SPValueType::String,
//             $b.iter().map(|x| x.clone().to_spvalue()).collect(),
//         )
//     };
// }

// #[macro_export]
// macro_rules! v_runner {
//     ($a:expr) => {
//         SPVariable::new(
//             $a.clone(),
//             SPVariableType::Runner,
//             SPValueType::String,
//             vec![],
//         )
//     };
// }

#[macro_export]
macro_rules! v {
    ($a:expr) => {
        SPVariable::new(
            $a.clone(),
            SPValueType::String,
            vec![],
        )
    };
}

#[macro_export]
macro_rules! bv {
    ($a:expr) => {
        SPVariable::new(
            $a.clone(),
            SPValueType::Bool,
            vec![true.to_spvalue(), false.to_spvalue()],
        )
    };
}

// #[macro_export]
// macro_rules! bv_command {
//     ($a:expr) => {
//         SPVariable::new(
//             $a.clone(),
//             SPVariableType::Command,
//             SPValueType::Bool,
//             vec![true.to_spvalue(), false.to_spvalue()],
//         )
//     };
// }

// #[macro_export]
// macro_rules! bv_measured {
//     ($a:expr) => {
//         SPVariable::new(
//             $a.clone(),
//             SPVariableType::Measured,
//             SPValueType::Bool,
//             vec![true.to_spvalue(), false.to_spvalue()],
//         )
//     };
// }

// #[macro_export]
// macro_rules! bv_estimated {
//     ($a:expr) => {
//         SPVariable::new(
//             $a.clone(),
//             SPVariableType::Estimated,
//             SPValueType::Bool,
//             vec![true.to_spvalue(), false.to_spvalue()],
//         )
//     };
// }

// #[macro_export]
// macro_rules! bv_runner {
//     ($a:expr) => {
//         SPVariable::new(
//             $a.clone(),
//             SPVariableType::Runner,
//             SPValueType::Bool,
//             vec![true.to_spvalue(), false.to_spvalue()],
//         )
//     };
// }

// #[macro_export]
// macro_rules! iv_command {
//     ($a:expr, $b:expr) => {
//         SPVariable::new(
//             $a.clone(),
//             SPVariableType::Command,
//             SPValueType::Int64,
//             $b.iter().map(|x| x.clone().to_spvalue()).collect(),
//         )
//     };
// }

#[macro_export]
macro_rules! iv {
    ($a:expr) => {
        SPVariable::new(
            $a.clone(),
            SPValueType::Int64,
            vec![],
            // $b.iter().map(|x| x.clone().to_spvalue()).collect(),
        )
    };
}

// #[macro_export]
// macro_rules! iv_measured {
//     ($a:expr, $b:expr) => {
//         SPVariable::new(
//             $a.clone(),
//             SPVariableType::Measured,
//             SPValueType::Int64,
//             $b.iter().map(|x| x.clone().to_spvalue()).collect(),
//         )
//     };
// }

// #[macro_export]
// macro_rules! iv_estimated {
//     ($a:expr, $b:expr) => {
//         SPVariable::new(
//             $a.clone(),
//             SPVariableType::Estimated,
//             SPValueType::Int64,
//             $b.iter().map(|x| x.clone().to_spvalue()).collect(),
//         )
//     };
// }

// #[macro_export]
// macro_rules! iv_runner {
//     ($a:expr) => {
//         SPVariable::new(
//             $a.clone(),
//             SPVariableType::Runner,
//             SPValueType::Int64,
//             vec![],
//         )
//     };
// }

// #[macro_export]
// macro_rules! fv_command {
//     ($a:expr, $b:expr) => {
//         SPVariable::new(
//             $a.clone(),
//             SPVariableType::Command,
//             SPValueType::Float64,
//             $b.iter().map(|x| x.clone().to_spvalue()).collect(),
//         )
//     };
// }

// #[macro_export]
// macro_rules! fv_measured {
//     ($a:expr, $b:expr) => {
//         SPVariable::new(
//             $a.clone(),
//             SPVariableType::Measured,
//             SPValueType::Float64,
//             $b.iter().map(|x| x.clone().to_spvalue()).collect(),
//         )
//     };
// }

// #[macro_export]
// macro_rules! fv_measured {
//     ($a:expr) => {
//         SPVariable::new(
//             $a.clone(),
//             SPVariableType::Measured,
//             SPValueType::Float64,
//             vec![],
//             // $b.iter().map(|x| x.clone().to_spvalue()).collect(),
//         )
//     };
// }

#[macro_export]
macro_rules! fv {
    ($a:expr) => {
        SPVariable::new(
            $a.clone(),
            SPValueType::Float64,
            vec![]
        )
    };
}

// #[macro_export]
// macro_rules! fv {
//     ($a:expr, $b:expr) => {
//         SPVariable::new(
//             $a.clone(),
//             SPValueType::Float64,
//             $b.iter().map(|x| x.clone().to_spvalue()).collect(),
//         )
//     };
// }

// #[macro_export]
// macro_rules! fv_estimated {
//     ($a:expr, $b:expr) => {
//         SPVariable::new(
//             $a.clone(),
//             SPVariableType::Estimated,
//             SPValueType::Float64,
//             $b.iter().map(|x| x.clone().to_spvalue()).collect(),
//         )
//     };
// }

// #[macro_export]
// macro_rules! fv_runner {
//     ($a:expr) => {
//         SPVariable::new(
//             $a.clone(),
//             SPVariableType::Runner,
//             SPValueType::Float64,
//             vec![],
//         )
//     };
// }

// #[macro_export]
// macro_rules! av_command {
//     ($a:expr, $b:expr) => {
//         SPVariable::new(
//             $a.clone(),
//             SPVariableType::Command,
//             SPValueType::Array,
//             $b.iter().map(|x| x.clone().to_spvalue()).collect(),
//         )
//     };
// }

// #[macro_export]
// macro_rules! av_measured {
//     ($a:expr, $b:expr) => {
//         SPVariable::new(
//             $a.clone(),
//             SPVariableType::Measured,
//             SPValueType::Array,
//             $b.iter().map(|x| x.clone().to_spvalue()).collect(),
//         )
//     };
// }

// #[macro_export]
// macro_rules! av_estimated {
//     ($a:expr, $b:expr) => {
//         SPVariable::new(
//             $a.clone(),
//             SPVariableType::Estimated,
//             SPValueType::Array,
//             $b.iter().map(|x| x.clone().to_spvalue()).collect(),
//         )
//     };
// }

// #[macro_export]
// macro_rules! av_runner {
//     ($a:expr) => {
//         SPVariable::new(
//             $a.clone(),
//             SPVariableType::Runner,
//             SPValueType::Array,
//             vec![],
//         )
//     };
// }

#[macro_export]
macro_rules! av {
    ($a:expr) => {
        SPVariable::new(
            $a.clone(),
            SPValueType::Array,
            vec![],
        )
    };
}
