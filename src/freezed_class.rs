#[derive(Debug)]
pub struct RedirectedConstructor {
    pub is_const: bool,
    pub class_name: String,
    pub constructor_name: Option<String>,
    pub named_arguments: Vec<NamedArgument>,
    pub assigned_type: DartType,
}

// #[derive(Debug)]
// pub struct Factory {
//     pub is_const: bool,
//     pub class_name: String,
//     pub constructor_name: String,
//     pub named_arguments: Vec<NamedArgument>,
// }

#[derive(Debug, Default)]
pub struct NamedArgument {
    pub annotations: Vec<Annotation>,
    pub is_required: bool,
    pub argument_type: DartType,
    pub name: String,
    pub default: Option<String>,
}

#[derive(Debug, Default)]
pub struct Annotation {
    pub name: String,
    pub arguments: Vec<String>,
}

#[derive(Debug, Default)]
pub struct DartType {
    pub name: String,
    pub nullable: bool,
    pub type_arguments: Vec<DartType>,
}

impl DartType {
    pub fn as_raw(&self) -> String {
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

pub struct FreezedClass2 {
    pub name: String,
    pub redirecting_constructors: Vec<RedirectedConstructor>,
}

impl FreezedClass2 {
    pub fn has_json(&self) -> bool {
        for c in &self.redirecting_constructors {
            if c.class_name == "fromJson" {
                return true;
            }
        }
        false
    }

    pub fn has_const_constructor(&self) -> bool {
        for c in &self.redirecting_constructors {
            if c.class_name == "_" {
                return c.is_const;
            }
        }
        false
    }
}
