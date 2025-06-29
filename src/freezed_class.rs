#[derive(Debug)]
pub struct RedirectedConstructor {
    pub constructor_name: String,
}

#[derive(Debug)]
pub struct FreezedClass {
    pub name: String,
    pub positional_arguments: Vec<Argument>,
    pub optional_arguments: Vec<Argument>,
    pub named_arguments: Vec<Argument>,
    pub has_json: bool,
    pub has_const_constructor: bool,
}

#[derive(Debug)]
pub struct Annotation {
    pub field: String,
}

#[derive(Debug)]
pub struct Argument {
    pub annotations: Vec<String>,
    pub name: String,
    pub r#type: String,
    pub default_value: Option<String>,
    pub is_required: bool,
}
