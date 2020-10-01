use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Clone, PartialOrd, Eq, Ord, Serialize, Deserialize)]
pub struct Parameter {
    pub name: String,
    pub value: bool,
}

#[derive(Debug, PartialEq, Clone, PartialOrd, Eq, Ord, Serialize, Deserialize)]
pub enum ControlKind {
    Measured,
    Command,
    Estimated,
    None,
}

#[derive(Debug, PartialEq, Clone, PartialOrd, Eq, Ord, Serialize, Deserialize)]
pub struct EnumVariable {
    pub name: String,
    pub r#type: String,
    pub domain: Vec<String>,
    pub param: Parameter,
    pub kind: ControlKind,
}

#[derive(Debug, PartialEq, Clone, PartialOrd, Eq, Ord, Serialize, Deserialize)]
pub struct EnumVariableValue {
    pub var: EnumVariable,
    pub val: String,
}

#[derive(Debug, PartialEq, Clone, PartialOrd, Eq, Ord, Serialize, Deserialize)]
pub struct State {
    pub measured: Vec<EnumVariableValue>,
    pub control: Vec<EnumVariableValue>,
    pub estimated: Vec<EnumVariableValue>,
}

impl Parameter {
    pub fn new(name: &str, value: &bool) -> Parameter {
        Parameter {
            name: name.to_owned(),
            value: *value,
        }
    }
}

impl EnumVariable {
    pub fn new(
        name: &str,
        domain: &Vec<&str>,
        param: Option<&Parameter>,
        kind: Option<&ControlKind>,
    ) -> EnumVariable {
        EnumVariable {
            name: match name == "EMPTY" {
                true => panic!(
                    "Error 69e2abf9-498b-4d5c-88c7-30ea70ed27fb: 
                EnumVariable name 'EMPTY' is reserved."
                ), // why?
                false => name.to_owned(),
            },
            r#type: name.to_owned(),
            domain: domain
                .iter()
                .map(|x| x.to_string())
                .collect::<Vec<String>>(),
            param: match param {
                Some(x) => x.to_owned(),
                None => Parameter::new("TRUE", &true),
            },
            kind: match kind {
                Some(x) => x.to_owned(),
                None => ControlKind::None,
            },
        }
    }
}

impl EnumVariableValue {
    pub fn new(var: &EnumVariable, val: &str) -> EnumVariableValue {
        EnumVariableValue {
            var: var.to_owned(),
            val: val.to_owned(),
        }
    }
}

#[test]
fn test_new_enum_variable() {
    let var1 = EnumVariable::new("var1", &vec!["a", "b", "c"], None, None);
    println!("{:?}", var1);
}
