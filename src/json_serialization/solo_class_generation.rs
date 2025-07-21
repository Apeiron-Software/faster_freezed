use crate::dart_types::{
    DartType, NamedParameter, ParameterList, RedirectedConstructor, get_generic_string,
};
use std::fmt::Write;

use super::{
    generate_eq_operator, generate_hash_operator, generate_mixin_copywith_function,
    generate_to_string, to_json_method_generator,
};

pub enum JsonMethod {
    None,
    Signature,
    Impl(String),
}

// maybe later?
#[allow(clippy::too_many_arguments)]
pub fn generate_solo_class(
    output: &mut String,
    class_name: &str,
    class_generics: &[DartType],
    class_to_json: JsonMethod,
    redirected_name: &str,
    parameters: &ParameterList,
    is_const: bool,
    unnamed_constructor: &Option<RedirectedConstructor>,
) {
    let mut copywith_generics = class_generics.to_owned();
    copywith_generics.push(DartType {
        name: "$Res".to_string(),
        ..Default::default()
    });

    let just_generics = get_generic_string(class_generics);
    let redirected_type = format!("{redirected_name}{just_generics}");

    let superclass = if unnamed_constructor.is_some() {
        format!("extends {class_name}{just_generics}")
    } else {
        format!("implements {class_name}{just_generics}")
    };

    let _ = writeln!(
        output,
        "class {redirected_name}{just_generics} {superclass} {{"
    );
    if is_const {
        let _ = writeln!(output, "const");
    }
    let _ = writeln!(output, "{redirected_name}(");

    for pos_field in &parameters.positional_parameters {
        let mut default_value: String = String::new();
        if let Some(default) = pos_field.annotations.iter().find(|e| e.name == "Default") {
            default_value = format!(" = {}", default.get_default_value());
        }

        let _ = write!(output, "this.{}, ", pos_field.name);
        let _ = write!(output, "{default_value}");
    }

    if !parameters.named_parameters.is_empty() {
        let _ = write!(output, "{{");
        for field in &parameters.named_parameters {
            generate_named_parameter(output, field);
            let _ = write!(output, ",");
        }
        let _ = write!(output, "}}");
    }
    if unnamed_constructor.is_some() {
        let _ = writeln!(output, "): super._();");
    } else {
        let _ = writeln!(output, ");");
    }

    // Here @override delection
    for parameter in parameters.get_all_params() {
        let _ = write!(
            output,
            "  final {} {};",
            parameter.dart_type.as_raw(),
            parameter.name
        );
    }

    generate_eq_operator(output, &redirected_type, &parameters.get_all_params());
    let _ = writeln!(output);
    generate_hash_operator(output, &parameters.get_all_params());
    let _ = writeln!(output);
    generate_to_string(output, class_name, &parameters.get_all_params(), true);
    let _ = writeln!(output);

    let mut copywith_use_generics = class_generics.to_owned();
    // That's a warcrime, check if can be done without nesting, just plain
    copywith_use_generics.push(DartType {
        name: DartType {
            name: class_name.to_string(),
            type_arguments: class_generics.to_owned(),
            nullable: false,
        }
        .as_raw(), // I could be worse, but like the fuck?
        ..Default::default()
    });
    let copywith_use_generics = get_generic_string(&copywith_use_generics);

    if !parameters.is_empty() {
        generate_mixin_copywith_function(
            output,
            redirected_name,
            &copywith_use_generics,
            &just_generics,
        );
        let _ = writeln!(output);
    }

    match class_to_json {
        JsonMethod::None => {}
        JsonMethod::Signature => {
            to_json_method_generator(output, None);
            let _ = writeln!(output);
        }
        JsonMethod::Impl(name) => {
            to_json_method_generator(output, Some(&name));
            let _ = writeln!(output);
        }
    }

    let _ = writeln!(output, "}}");
}

fn generate_named_parameter(output: &mut String, parameter: &NamedParameter) {
    if parameter.is_required {
        let _ = write!(output, "required ");
    }
    let _ = write!(output, "this.{}", parameter.name);

    let mut default_value: String = String::new();
    if let Some(default) = parameter.annotations.iter().find(|e| e.name == "Default") {
        default_value = format!(" = {}", default.get_default_value());
    }
    let _ = write!(output, "{default_value}");
}
