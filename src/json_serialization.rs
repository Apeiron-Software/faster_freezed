//! JSON serialization code generation helpers for Dart @freezed classes
use crate::freezed_class::Argument;

/// Generate JSON serialization code for a single field type
pub fn generate_field_from_json(field_name: &str, field_type: &str, arg: &Argument) -> String {
    let is_nullable = field_type.contains('?');
    // Check if field has a JsonConverter decorator
    let json_converter = arg.annotations.iter().find(|annotation| {
        annotation.contains("JsonConverter")
    });
    if let Some(converter) = json_converter {
        let converter_name = converter
            .trim_start_matches('@')
            .trim_end_matches("()")
            .trim_end_matches('(');
        if is_nullable {
            format!("{}: json['{}'] == null ? null : const {}().fromJson((json['{}'] as num).toInt())", 
                field_name, field_name, converter_name, field_name)
        } else {
            format!("{}: const {}().fromJson((json['{}'] as num).toInt())", 
                field_name, converter_name, field_name)
        }
    } else {
        if field_type == "int" {
            format!("{}: (json['{}'] as num).toInt()", field_name, field_name)
        } else if field_type == "int?" {
            format!("{}: (json['{}'] as num?)?.toInt()", field_name, field_name)
        } else if field_type == "double" {
            format!("{}: (json['{}'] as num).toDouble()", field_name, field_name)
        } else if field_type == "double?" {
            format!("{}: (json['{}'] as num?)?.toDouble()", field_name, field_name)
        } else if field_type == "String" {
            format!("{}: json['{}'] as String", field_name, field_name)
        } else if field_type == "String?" {
            format!("{}: json['{}'] as String?", field_name, field_name)
        } else if field_type.starts_with("List<") {
            generate_list_from_json(field_name, field_type)
        } else if field_type.starts_with("Map<") {
            generate_map_from_json(field_name, field_type)
        } else if field_type.starts_with("Set<") {
            generate_set_from_json(field_name, field_type)
        } else {
            generate_object_from_json(field_name, field_type)
        }
    }
}

pub fn generate_list_from_json(field_name: &str, field_type: &str) -> String {
    let is_list_nullable = field_type.ends_with('?');
    let list_content = if is_list_nullable { 
        &field_type[5..field_type.len()-2]
    } else { 
        &field_type[5..field_type.len()-1]
    };
    if !is_list_nullable {
        if list_content == "int" {
            format!("{}: (json['{}'] as List<dynamic>).map((e) => (e as num).toInt()).toList()", field_name, field_name)
        } else if list_content == "double" {
            format!("{}: (json['{}'] as List<dynamic>).map((e) => (e as num).toDouble()).toList()", field_name, field_name)
        } else if list_content == "String" {
            format!("{}: (json['{}'] as List<dynamic>).map((e) => e as String).toList()", field_name, field_name)
        } else {
            format!("{}: (json['{}'] as List<dynamic>).map((e) => {}.fromJson(e as Map<String, dynamic>)).toList()", field_name, field_name, list_content)
        }
    } else {
        if list_content == "int" {
            format!("{}: (json['{}'] as List<dynamic>?)?.map((e) => (e as num).toInt()).toList()", field_name, field_name)
        } else if list_content == "double" {
            format!("{}: (json['{}'] as List<dynamic>?)?.map((e) => (e as num).toDouble()).toList()", field_name, field_name)
        } else if list_content == "String" {
            format!("{}: (json['{}'] as List<dynamic>?)?.map((e) => e as String).toList()", field_name, field_name)
        } else {
            format!("{}: (json['{}'] as List<dynamic>?)?.map((e) => {}.fromJson(e as Map<String, dynamic>)).toList()", field_name, field_name, list_content)
        }
    }
}

pub fn generate_map_from_json(field_name: &str, field_type: &str) -> String {
    let is_nullable = field_type.contains('?');
    if is_nullable {
        format!("{}: json['{}'] as Map<String, dynamic>?", field_name, field_name)
    } else {
        format!("{}: json['{}'] as Map<String, dynamic>", field_name, field_name)
    }
}

pub fn generate_set_from_json(field_name: &str, field_type: &str) -> String {
    let is_nullable = field_type.contains('?');
    let inner_type = &field_type[4..field_type.len()-1];
    let is_set_nullable = inner_type.ends_with('?');
    let base_inner_type = if is_set_nullable { &inner_type[..inner_type.len()-1] } else { inner_type };
    if !is_nullable {
        if base_inner_type == "int" {
            format!("{}: (json['{}'] as List<dynamic>).map((e) => (e as num).toInt()).toSet()", field_name, field_name)
        } else if base_inner_type == "double" {
            format!("{}: (json['{}'] as List<dynamic>).map((e) => (e as num).toDouble()).toSet()", field_name, field_name)
        } else if base_inner_type == "String" {
            format!("{}: (json['{}'] as List<dynamic>).map((e) => e as String).toSet()", field_name, field_name)
        } else {
            format!("{}: (json['{}'] as List<dynamic>).map((e) => {}.fromJson(e as Map<String, dynamic>)).toSet()", field_name, field_name, base_inner_type)
        }
    } else {
        if base_inner_type == "int" {
            format!("{}: (json['{}'] as List<dynamic>?)?.map((e) => (e as num).toInt()).toSet()", field_name, field_name)
        } else if base_inner_type == "double" {
            format!("{}: (json['{}'] as List<dynamic>?)?.map((e) => (e as num).toDouble()).toSet()", field_name, field_name)
        } else if base_inner_type == "String" {
            format!("{}: (json['{}'] as List<dynamic>?)?.map((e) => e as String).toSet()", field_name, field_name)
        } else {
            format!("{}: (json['{}'] as List<dynamic>?)?.map((e) => {}.fromJson(e as Map<String, dynamic>)).toSet()", field_name, field_name, base_inner_type)
        }
    }
}

pub fn generate_object_from_json(field_name: &str, field_type: &str) -> String {
    let is_nullable = field_type.contains('?');
    if is_nullable {
        let base_type = &field_type[..field_type.len()-1];
        format!("{}: json['{}'] == null ? null : {}.fromJson(json['{}'] as Map<String, dynamic>)", field_name, field_name, base_type, field_name)
    } else {
        format!("{}: {}.fromJson(json['{}'] as Map<String, dynamic>)", field_name, field_type, field_name)
    }
}

pub fn generate_field_to_json(field_name: &str, arg: &Argument) -> String {
    let json_converter = arg.annotations.iter().find(|annotation| {
        annotation.contains("JsonConverter")
    });
    if let Some(converter) = json_converter {
        let converter_name = converter
            .trim_start_matches('@')
            .trim_end_matches("()")
            .trim_end_matches('(');
        format!("  '{}': const {}().toJson(instance.{})", field_name, converter_name, field_name)
    } else {
        format!("  '{}': instance.{}", field_name, field_name)
    }
} 