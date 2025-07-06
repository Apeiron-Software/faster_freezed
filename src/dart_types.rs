#[derive(Debug, Default)]
pub struct RedirectedConstructor {
    pub is_const: bool,
    pub class_name: String,
    pub constructor_name: Option<String>,
    pub parameters: ParameterList,
    pub assigned_type: DartType,
}

#[derive(Debug, Default)]
pub struct ParameterList {
    pub positional_parameters: Vec<PositionalParameter>,
    pub named_parameters: Vec<NamedParameter>,
}

impl ParameterList {
    pub fn new(
        positional_parameters: Vec<PositionalParameter>,
        named_parameters: Vec<NamedParameter>,
    ) -> Self {
        Self {
            positional_parameters,
            named_parameters,
        }
    }

    pub fn get_all_params(&self) -> Vec<PositionalParameter> {
        self.positional_parameters
            .clone()
            .into_iter()
            .chain(self.named_parameters.iter().map(|e| e.to_positional()))
            .to_owned()
            .collect()
    }
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct PositionalParameter {
    pub name: String,
    pub dart_type: DartType,
    pub annotations: Vec<Annotation>,
}

impl NamedParameter {
    pub fn get_default_value(&self) -> Option<String> {
        // TODO!!!
        // If annotations contains default, bla bla
        return None;
    }
}

#[derive(Debug, Default)]
pub struct NamedParameter {
    pub annotations: Vec<Annotation>,
    pub is_required: bool,
    pub dart_type: DartType,
    pub name: String,
    pub default: Option<String>,
}

impl NamedParameter {
    pub fn to_positional(&self) -> PositionalParameter {
        PositionalParameter {
            name: self.name.clone(),
            dart_type: self.dart_type.clone(),
            annotations: self.annotations.clone(),
        }
    }
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct Annotation {
    pub name: String,
    pub arguments: Vec<String>,
}

#[derive(Debug, Default, Clone, Eq, PartialEq)]
pub struct DartType {
    pub name: String,
    pub nullable: bool,
    pub type_arguments: Vec<DartType>,
}

impl DartType {
    pub fn as_raw(&self) -> String {
        if self.name.is_empty() {
            return "dynamic".to_owned();
        }

        let mut raw = String::new();
        raw.push_str(&self.name);
        if !self.type_arguments.is_empty() {
            raw.push('<');
            for t in &self.type_arguments {
                raw.push_str(&t.as_raw());
                raw.push(',');
            }
            raw.pop(); // popping training ,
            raw.push('>');
        }
        if self.nullable {
            raw.push('?');
        }
        raw
    }

    // Temporary
    pub fn is_collection(&self) -> bool {
        ["List", "Map", "Set"].contains(&self.name.as_str())
    }
}

#[derive(Debug)]
pub struct ClassDefinition {
    pub name: String,
    // pub type_parameters: Vec<String>, // FEAT: propper generic types
    pub mixins: Vec<DartType>,
    pub json_constructor: Option<RedirectedConstructor>,
    pub unnamed_constructor: Option<RedirectedConstructor>,
    pub redirecting_constructors: Vec<RedirectedConstructor>,
}

impl ClassDefinition {
}
