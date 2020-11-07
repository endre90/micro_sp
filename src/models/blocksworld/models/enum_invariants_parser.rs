use super::*;

use std::fs::File;
use std::io;
use std::io::prelude::*;

pub fn parser(name: &str) -> (ParamPlanningProblem, Vec<String>) {

    let blocks = vec!("A", "B", "C", "D");
    let on_domain = vec!("A", "B", "C", "D", "GRIP", "TABLE");

    let init = ParamPredicate::new(
        &vec!(
            Predicate::AND(
                vec!(
                    pass!(&new_bool_assign_c!(&format!("clear_{}", "A"), true, "clear")),
                    pass!(&new_bool_assign_c!(&format!("clear_{}", "B"), true, "clear")),
                    pass!(&new_bool_assign_c!(&format!("clear_{}", "C"), true, "clear")),
                    pass!(&new_bool_assign_c!(&format!("clear_{}", "D"), true, "clear")),
                    pass!(&new_enum_assign_c!(&format!("{}_on", "A"), &on_domain, "TABLE", "on", "on")),
                    pass!(&new_enum_assign_c!(&format!("{}_on", "B"), &on_domain, "TABLE", "on", "on")),
                    pass!(&new_enum_assign_c!(&format!("{}_on", "C"), &on_domain, "TABLE", "on", "on")),
                    pass!(&new_enum_assign_c!(&format!("{}_on", "D"), &on_domain, "TABLE", "on", "on"))
                )
            )
        )
    );

    let goal = ParamPredicate::new(
        &vec!(
            Predicate::AND(
                vec!(
                    pass!(&new_enum_assign_c!(&format!("{}_on", "D"), &on_domain, "C", "on", "on")),
                    pass!(&new_enum_assign_c!(&format!("{}_on", "C"), &on_domain, "B", "on", "on")),
                    pass!(&new_enum_assign_c!(&format!("{}_on", "B"), &on_domain, "A", "on", "on"))
                )
            )
        )
    );

    let problem = ParamPlanningProblem::new(
        name, 
        &init, 
        &goal, 
        &vec!(), 
        &Predicate::TRUE,
        &vec!()
    );

    (problem, blocks.iter().map(|x| x.to_string()).collect())
    
}