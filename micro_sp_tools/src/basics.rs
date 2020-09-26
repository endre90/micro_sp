use arrayvec::ArrayString;

#[derive(Copy, Debug, PartialEq, Clone, PartialOrd, Eq, Ord)]
pub struct Parameter {
    pub name: ArrayString::<[u8; 32]>,
    pub value: bool
}

#[derive(Copy, Eq, Debug, PartialEq, Clone, PartialOrd, Ord)]
pub enum ControlKind {
    Measured,
    Command,
    Estimated,
    None
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct KeyValuePair {
    pub key: ArrayString::<[u8; 32]>,
    pub value: ArrayString::<[u8; 32]>,
}   

#[derive(Eq, Debug, PartialEq, Clone, PartialOrd, Ord)]
pub struct EnumVariable {
    pub name: KeyValuePair,
    pub r#type: ArrayString::<[u8; 32]>,
    pub domain: Vec<ArrayString::<[u8; 32]>>,
    pub param: Parameter,
    pub kind: ControlKind
}

#[derive(Eq, Debug, PartialEq, Clone, PartialOrd, Ord)]
pub struct State {
    pub pairs: Vec<KeyValuePair>
}

impl KeyValuePair {

    pub fn new(key: &str, value: &str) -> KeyValuePair {
        KeyValuePair {
            key: match key.len() > 32 {
                false => ArrayString::<[_; 32]>::from(key).unwrap_or_default(),
                true => panic!("Error 5d3311f5-79c3-4225-a9d2-f2a440b3f3c5: Variable name length too big.")
            },
            value: match value.len() > 32 {
                false => ArrayString::<[_; 32]>::from(value).unwrap_or_default(),
                true => panic!("Error 6e61625f-ec4c-4c25-81d7-38182a287c4e: Value name lenght too big.")
            }
        }
    }

    pub fn dummy(key: &str) -> KeyValuePair {
        KeyValuePair {
            key: match key.len() > 32 {
                false => ArrayString::<[_; 32]>::from(key).unwrap_or_default(),
                true => panic!("Error 5d3311f5-79c3-4225-a9d2-f2a440b3f3c5: Variable name length too big.")
            },
            value: ArrayString::<[_; 32]>::from("dummy_value").unwrap_or_default(),
        }
    }
}

impl State {
    pub fn new(pairs: &Vec<KeyValuePair>) -> State {
        State {
            pairs: pairs.iter().map(|x| x.to_owned()).collect()
        }
    }
}

impl Parameter {
    pub fn new(name: &str, value: &bool) -> Parameter {
        match name == "TRUE" {
            true => panic!("Error 5b376941-3c6e-4b52-bec3-49eb8d9991bb: Parameter name 'TRUE' is reserved."),
            false => {
                Parameter {
                    name: ArrayString::<[_; 32]>::from(name).unwrap(),
                    value: *value
                }
            }
        }
    }
}

impl Default for Parameter {
    fn default() -> Self {
        Parameter {
            name: ArrayString::<[_; 32]>::from("TRUE").unwrap(),
            value: true
        }
    }
}

impl EnumVariable{
    pub fn new(name: &str, r#type: &str, domain: &Vec<&str>, param: Option<&Parameter>, kind: &ControlKind) -> EnumVariable {
        EnumVariable { 
            param: match param {
                Some(x) => x.to_owned(),
                None => Parameter::default()
            },
            name: match name == "EMPTY" {
                true => panic!("Error 69e2abf9-498b-4d5c-88c7-30ea70ed27fb: EnumVariable name 'EMPTY' is reserved."),
                false => KeyValuePair::dummy(name)
            },

            r#type: ArrayString::<[_; 32]>::from(r#type).unwrap(),
            domain: domain.iter().map(|x| ArrayString::<[_; 32]>::from(x).unwrap()).collect::<Vec<ArrayString::<[u8; 32]>>>(),
            kind: kind.to_owned()
        }
    }
}

impl Default for EnumVariable {
    fn default() -> Self {
        EnumVariable {
            param: Parameter::default(),
            name: KeyValuePair::dummy("EMPTY"),
            r#type: ArrayString::<[_; 32]>::from("EMPTY").unwrap(),
            domain: vec!(),
            kind: ControlKind::None
        }
    }
}

#[test]
fn test_new_enum_variable(){
    let param = Parameter::new("param1", &false);
    assert_eq!("EnumVariable { name: KeyValuePair { key: \"z\", value: \"dummy_value\" }, type: \"letters\", domain: [\"a\", \"b\", \"c\", \"d\"], param: Parameter { name: \"TRUE\", value: true }, kind: None }", 
        &format!("{:?}", EnumVariable::new("z", "letters", &vec!("a", "b", "c", "d"), None, &ControlKind::None)));
    assert_eq!("EnumVariable { name: KeyValuePair { key: \"z\", value: \"dummy_value\" }, type: \"letters\", domain: [\"a\", \"b\", \"c\", \"d\"], param: Parameter { name: \"param1\", value: false }, kind: None }", 
        &format!("{:?}", EnumVariable::new("z", "letters", &vec!("a", "b", "c", "d"), Some(&param), &ControlKind::None)));

}