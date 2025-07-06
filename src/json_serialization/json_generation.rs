use crate::dart_types::{Annotation, DartType, ParameterList, PositionalParameter};
use std::fmt::Write;

pub fn from_json_method_generator(output: &mut String, class_name: &str, from_json_name: &str) {
    let _ = writeln!(
        output,
        "  factory {class_name}.fromJson(Map<String, dynamic> json)"
    );
    let _ = writeln!(output, "   => {from_json_name}(json);");
}

pub fn from_json_function_generator(
    output: &mut String,
    class_name: &str,
    from_json_name: &str,
    parameters: &ParameterList,
) {
    let _ = writeln!(
        output,
        "{class_name} _${from_json_name}FromJson(Map<String, dynamic> json) =>"
    );

    let _ = writeln!(output, "{class_name}(");
    for parameter in &parameters.positional_parameters {
        let from_item = format!("json['{0}']", parameter.name);
        from_json_field_gen(
            output,
            &from_item,
            &parameter.dart_type,
            &parameter.annotations,
        );
        let _ = writeln!(output, ", ");
    }

    if !parameters.named_parameters.is_empty() {
        for parameter in &parameters.named_parameters {
            let from_item = format!("json['{0}']", parameter.name);
            let _ = writeln!(output, "{}: ", parameter.name);
            from_json_field_gen(
                output,
                &from_item,
                &parameter.dart_type,
                &parameter.annotations,
            );
            let _ = writeln!(output, ", ");
        }
    }

    let _ = writeln!(output, ");",);
}

fn from_json_field_gen(
    output: &mut String,
    from_item: &str,
    dart_type: &DartType,
    annotations: &[Annotation],
) {
    let _ = writeln!(output, "({from_item} as {})", dart_type.as_raw());
    // TODO
}

pub fn to_json_method_generator(output: &mut String, class_name: Option<&str>) {
    if let Some(class_name) = class_name {
        let _ = writeln!(output, "  @override");
        let _ = writeln!(output, "  Map<String, dynamic> toJson() {{");
        let _ = writeln!(output, "    return _${class_name}ToJson(this);");
        let _ = writeln!(output, "  }}");
    } else {
        let _ = writeln!(output, "  Map<String, dynamic> toJson();");
    }
}

pub fn to_json_function_generator(
    output: &mut String,
    to_json_name: &str,
    class_name: &str,
    fields: &[PositionalParameter],
) {
    let _ = writeln!(
        output,
        "Map<String, dynamic> _${to_json_name}ToJson({class_name} instance) =>"
    );
    let _ = writeln!(output, "    <String, dynamic>{{");

    for parameter in fields {
        let _ = write!(output, "    ");
        to_json_field_gen(output, parameter);
        let _ = writeln!(output, ",");
    }

    let _ = writeln!(output, "    }};");
}

fn to_json_field_gen(output: &mut String, parameter: &PositionalParameter) {
    if parameter.dart_type.name == "DateTime" {
        let nullable = if parameter.dart_type.nullable {
            "?"
        } else {
            ""
        };

        let _ = write!(
            output,
            "'{0}': instance.{0}{nullable}.toIso8601String()",
            parameter.name
        );
    } else {
        let _ = write!(output, "'{0}': instance.{0}", parameter.name);
    }
}
