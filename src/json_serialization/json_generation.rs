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
    let mut is_nullable = dart_type.nullable;
    let mut default_value: String = String::new();

    if let Some(default) = annotations.iter().find(|e| e.name == "Default") {
        default_value = format!(" ?? {}", default.get_default_value());
        is_nullable = true;
    }

    if let Some(converter) = annotations
        .iter()
        .find(|e| e.name.ends_with("JsonConverter"))
    {
        if is_nullable {
            let _ = writeln!(
                output,
                "
                ({from_item} == null ? null : const {0}().fromJson({from_item}))
                ",
                converter.name
            );
        } else {
            let _ = writeln!(output, "const {0}().fromJson({from_item})", converter.name);
        }
    }

    let nullable = if is_nullable { "?" } else { "" };

    match dart_type.name.as_str() {
        "int" => {
            assert!(dart_type.type_arguments.is_empty());
            let _ = writeln!(output, "({from_item} as num{nullable}){nullable}.toInt()");
        }
        "double" => {
            assert!(dart_type.type_arguments.is_empty());
            let _ = writeln!(
                output,
                "({from_item} as num{nullable}){nullable}.toDouble()"
            );
        }

        "bool" => {
            assert!(dart_type.type_arguments.is_empty());
            let _ = writeln!(output, "(({from_item}) as bool{nullable})");
        }
        "String" => {
            assert!(dart_type.type_arguments.is_empty());
            let _ = writeln!(output, "(({from_item}) as String{nullable})");
        }
        "DateTime" => {
            assert!(dart_type.type_arguments.is_empty());
            if is_nullable {
                let _ = writeln!(
                    output,
                    "({from_item} == null ? null : DateTime.parse({from_item} as String))"
                );
            } else {
                let _ = writeln!(output, "DateTime.parse({from_item} as String)");
            }
        }
        "List" => {
            assert_eq!(dart_type.type_arguments.len(), 1);
            let mut inner_output = String::new();
            let inner_type = dart_type.type_arguments.first().unwrap();
            from_json_field_gen(&mut inner_output, "e", inner_type, &[]);

            let _ = writeln!(
                output,
                "({from_item} as List<dynamic>{nullable}){nullable}.map(
    (e) => {0} ).toList()",
                &inner_output
            );
        }

        "" | "dynamic" => {
            let _ = writeln!(output, "{from_item}");
        }

        // If there's no pattern matched, then assume this is an object with .fromJson method
        _ => {
            let _ = writeln!(output, "{0}.fromJson({from_item})", dart_type.as_raw());
        }
    }

    let _ = writeln!(output, "{default_value}");
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
    if let Some(converter) = parameter
        .annotations
        .iter()
        .find(|e| e.name.ends_with("JsonConverter"))
    {
        if parameter.dart_type.nullable {
            let _ = writeln!(
                output,
                "
                (instance.{0} == null ? null : const {1}().toJson(instance.{0}))
                ",
                parameter.name, converter.name
            );
        } else {
            let _ = writeln!(
                output,
                "const {1}().toJson(instance.{0})",
                parameter.name, converter.name
            );
        }
    }

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
