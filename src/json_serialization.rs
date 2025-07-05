//! JSON serialization code generation helpers for Dart @freezed classes
use crate::freezed_class::NamedArgument;

/// Generate Dart code to convert a value of the given type from JSON, using expr as the value
pub fn generate_element_from_json(element_type: &str, expr: &str) -> String {
    let is_nullable = element_type.ends_with('?');
    let base_type = if is_nullable {
        &element_type[..element_type.len() - 1]
    } else {
        element_type
    };
    match base_type {
        "int" => {
            if is_nullable {
                format!("({} as num?)?.toInt()", expr)
            } else {
                format!("({} as num).toInt()", expr)
            }
        }
        "double" => {
            if is_nullable {
                format!("({} as num?)?.toDouble()", expr)
            } else {
                format!("({} as num).toDouble()", expr)
            }
        }
        "String" => {
            if is_nullable {
                format!("{} as String?", expr)
            } else {
                format!("{} as String", expr)
            }
        }
        "bool" => {
            if is_nullable {
                format!("{} as bool?", expr)
            } else {
                format!("{} as bool", expr)
            }
        }
        "DateTime" => {
            if is_nullable {
                format!(
                    "{} == null ? null : DateTime.parse({} as String)",
                    expr, expr
                )
            } else {
                format!("DateTime.parse({} as String)", expr)
            }
        }
        _ => {
            if is_nullable {
                format!(
                    "{} == null ? null : {}.fromJson({} as Map<String, dynamic>)",
                    expr, base_type, expr
                )
            } else {
                format!("{}.fromJson({} as Map<String, dynamic>)", base_type, expr)
            }
        }
    }
}

/// Generate JSON serialization code for a single field type
pub fn generate_field_from_json(field_name: &str, arg: &NamedArgument) -> String {
    // let json_converter = arg.annotations.iter().find(|annotation| {
    //     annotation.contains("JsonConverter")
    // });
    //
    // if let Some(converter) = json_converter {
    //     let converter_name = converter
    //         .trim_start_matches('@')
    //         .trim_end_matches("()")
    //         .trim_end_matches('(');
    //     if is_nullable {
    //         format!("{}: json['{}'] == null ? null : const {}().fromJson((json['{}'] as num).toInt())",
    //             field_name, field_name, converter_name, field_name)
    //     } else {
    //         format!("{}: const {}().fromJson((json['{}'] as num).toInt())",
    //             field_name, converter_name, field_name)
    //     }
    //} else
    if arg.argument_type.is_collection() {
        todo!("Cursor horror are deleted, need good implementation");
    } else {
        format!(
            "{}: {}",
            field_name,
            generate_element_from_json(
                &arg.argument_type.as_raw(),
                &format!("json['{}']", field_name)
            )
        )
    }
}

pub fn generate_object_from_json(field_name: &str, field_type: &str) -> String {
    let is_nullable = field_type.contains('?');
    if is_nullable {
        let base_type = &field_type[..field_type.len() - 1];
        format!(
            "{}: json['{}'] == null ? null : {}.fromJson(json['{}'] as Map<String, dynamic>)",
            field_name, field_name, base_type, field_name
        )
    } else {
        format!(
            "{}: {}.fromJson(json['{}'] as Map<String, dynamic>)",
            field_name, field_type, field_name
        )
    }
}

pub fn generate_field_to_json(field_name: &str) -> String {
    format!("  '{}': instance.{}", field_name, field_name)
}

