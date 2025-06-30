pub mod freezed_class;
pub mod parser;

use freezed_class::{FreezedClass, Argument};

/// Parse Dart code and extract all classes with @freezed annotation
/// 
/// # Arguments
/// * `code` - The Dart source code as a string
/// 
/// # Returns
/// * A vector of FreezedClass instances representing the parsed classes
/// 
/// # Example
/// ```
/// use faster_freezed::parse_freezed_classes;
/// 
/// let dart_code = r#"
/// @freezed
/// class Person with _$Person {
///   const factory Person({
///     required String name,
///     int age,
///   }) = _Person;
/// }
/// "#;
/// 
/// let classes = parse_freezed_classes(dart_code.to_string());
/// assert_eq!(classes.len(), 1);
/// ```
pub fn parse_freezed_classes(code: String) -> Vec<FreezedClass> {
    parser::parse_dart_code(&code)
}

/// Generate getter declarations for all fields in the mixin
fn generate_getters(all_fields: &[&Argument]) -> String {
    let getters: Vec<String> = all_fields
        .iter()
        .map(|arg| format!("  {} get {};", arg.r#type.replace("required ", "").trim(), arg.name))
        .collect();
    getters.join("\n")
}

/// Generate equality operator for the mixin
fn generate_equality_operator(class_name: &str, all_fields: &[&Argument]) -> String {
    let mut output = String::new();
    output.push_str("  @override\n");
    output.push_str("  bool operator ==(Object other) {\n");
    output.push_str("    return identical(this, other) ||\n");
    output.push_str("        (other.runtimeType == runtimeType &&\n");
    output.push_str(&format!("            other is {}\n", class_name));
    
    let equality_checks: Vec<String> = all_fields
        .iter()
        .map(|arg| {
            let is_collection = arg.r#type.starts_with("List<") || 
                               arg.r#type.starts_with("Map<") || 
                               arg.r#type.starts_with("Set<") ||
                               arg.r#type == "List" ||
                               arg.r#type == "Map" ||
                               arg.r#type == "Set";
            if is_collection {
                format!("            && const DeepCollectionEquality().equals(other.{0}, {0})", arg.name)
            } else {
                format!("            && (identical(other.{0}, {0}) || other.{0} == {0})", arg.name)
            }
        })
        .collect();
    output.push_str(&equality_checks.join("\n"));
    output.push_str(");\n");
    output.push_str("  }\n");
    output
}

/// Generate hashCode getter for the mixin
fn generate_hash_code(all_fields: &[&Argument]) -> String {
    let mut output = String::new();
    output.push_str("\n  @override\n");
    output.push_str("  int get hashCode => Object.hash(runtimeType, ");
    let hash_fields: Vec<String> = all_fields
        .iter()
        .map(|arg| arg.name.clone())
        .collect();
    output.push_str(&hash_fields.join(", "));
    output.push_str(");\n");
    output
}

/// Generate toString method for the mixin
fn generate_to_string(class_name: &str, all_fields: &[&Argument]) -> String {
    let mut output = String::new();
    output.push_str("\n  @override\n");
    output.push_str("  String toString() {\n");
    output.push_str(&format!("    return '{}({})';\n", 
        class_name, 
        all_fields.iter().map(|arg| format!("{}: ${}", arg.name, arg.name)).collect::<Vec<_>>().join(", ")
    ));
    output.push_str("  }\n");
    output
}

/// Generate copyWith method declaration in the mixin
fn generate_copy_with_declaration(class_name: &str, all_fields: &[&Argument]) -> String {
    let mut output = String::new();
    output.push_str(&format!("  {} copyWith({{", class_name));
    let copy_with_params: Vec<String> = all_fields
        .iter()
        .map(|arg| {
            let is_nullable = arg.r#type.contains('?');
            if is_nullable {
                format!("Object? {} = freezed", arg.name)
            } else {
                format!("{}? {}", arg.r#type, arg.name)
            }
        })
        .collect();
    output.push_str(&copy_with_params.join(", "));
    output.push_str("});\n");
    output
}

/// Generate toJson method declaration in the mixin
fn generate_to_json_declaration() -> String {
    "  Map<String, dynamic> toJson();\n".to_string()
}

/// Generate constructor for the class implementation
fn generate_constructor(class: &FreezedClass) -> String {
    let const_keyword = if class.has_const_constructor { "const " } else { "" };
    let mut output = String::new();
    
    if class.positional_arguments.is_empty() {
        // Only named parameters
        let named_params: Vec<String> = class.named_arguments
            .iter()
            .map(|arg| {
                if arg.is_required {
                    format!("required this.{}", arg.name)
                } else if let Some(default) = &arg.default_value {
                    format!("this.{} = {}", arg.name, default)
                } else {
                    format!("this.{}", arg.name)
                }
            })
            .collect();
        output.push_str(&format!("  {} _{} ({{{}}}) : super._();\n", 
            const_keyword, class.name, named_params.join(", ")));
    } else if class.named_arguments.is_empty() {
        // Only positional parameters
        let pos_params: Vec<String> = class.positional_arguments
            .iter()
            .map(|arg| {
                if arg.is_required {
                    format!("required this.{}", arg.name)
                } else if let Some(default) = &arg.default_value {
                    format!("this.{} = {}", arg.name, default)
                } else {
                    format!("this.{}", arg.name)
                }
            })
            .collect();
        output.push_str(&format!("  {}const _{} ({}) : super._();\n", 
            const_keyword, class.name, pos_params.join(", ")));
    } else {
        // Both positional and named parameters
        let pos_params: Vec<String> = class.positional_arguments
            .iter()
            .map(|arg| {
                if arg.is_required {
                    format!("required this.{}", arg.name)
                } else if let Some(default) = &arg.default_value {
                    format!("this.{} = {}", arg.name, default)
                } else {
                    format!("this.{}", arg.name)
                }
            })
            .collect();
        let named_params: Vec<String> = class.named_arguments
            .iter()
            .map(|arg| {
                if arg.is_required {
                    format!("required this.{}", arg.name)
                } else if let Some(default) = &arg.default_value {
                    format!("this.{} = {}", arg.name, default)
                } else {
                    format!("this.{}", arg.name)
                }
            })
            .collect();
        output.push_str(&format!("  {} _{} ({}, {{{}}}) : super._();\n", 
            const_keyword, class.name, pos_params.join(", "), named_params.join(", ")));
    }
    output
}

/// Generate final field declarations for the class implementation
fn generate_field_declarations(all_fields: &[&Argument]) -> String {
    let mut output = String::new();
    for field in all_fields {
        output.push_str(&format!("  @override\n  final {} {};\n", field.r#type, field.name));
    }
    output
}

/// Generate copyWith method implementation in the class
fn generate_copy_with_implementation(class_name: &str, all_fields: &[&Argument]) -> String {
    let mut output = String::new();
    output.push_str("\n  @override\n");
    output.push_str(&format!("  {} copyWith({{", class_name));
    let copy_with_params_impl: Vec<String> = all_fields
        .iter()
        .map(|arg| {
            let is_nullable = arg.r#type.contains('?');
            if is_nullable {
                format!("Object? {} = freezed", arg.name)
            } else {
                format!("Object? {}", arg.name)
            }
        })
        .collect();
    output.push_str(&copy_with_params_impl.join(", "));
    output.push_str("}) {\n");
    output.push_str(&format!("    return _{}(", class_name));
    let copy_with_args: Vec<String> = all_fields
        .iter()
        .map(|arg| {
            let is_nullable = arg.r#type.contains('?');
            if is_nullable {
                format!("{}: freezed == {} ? this.{} : {} as {}", 
                    arg.name, arg.name, arg.name, arg.name, arg.r#type)
            } else {
                format!("{}: {} == null ? this.{} : {} as {}", 
                    arg.name, arg.name, arg.name, arg.name, arg.r#type)
            }
        })
        .collect();
    output.push_str(&copy_with_args.join(", "));
    output.push_str(");\n");
    output.push_str("  }\n");
    output
}

/// Generate fromJson factory method for the class
fn generate_from_json_factory(class_name: &str) -> String {
    format!("  factory _{}.fromJson(Map<String, dynamic> json) => _${}FromJson(json);\n", class_name, class_name)
}

/// Generate toJson method implementation in the class
fn generate_to_json_implementation(class_name: &str) -> String {
    let mut output = String::new();
    output.push_str("\n  @override\n");
    output.push_str("  Map<String, dynamic> toJson() {\n");
    output.push_str(&format!("    return _${}ToJson(this);\n", class_name));
    output.push_str("  }\n");
    output
}

/// Generate the fromJson function implementation
fn generate_from_json_function(class_name: &str, all_fields: &[&Argument]) -> String {
    let mut output = String::new();
    output.push_str(&format!("_{} _${}FromJson(Map<String, dynamic> json) => _{}(", 
        class_name, class_name, class_name));
    
    let from_json_args: Vec<String> = all_fields
        .iter()
        .map(|arg| {
            let field_name = &arg.name;
            let field_type = &arg.r#type;
            let is_nullable = field_type.contains('?');
            
            // Check if field has a JsonConverter decorator
            let json_converter = arg.annotations.iter().find(|annotation| {
                annotation.contains("JsonConverter")
            });
            
            if let Some(converter) = json_converter {
                // Extract converter name (e.g., "DurationJsonConverter" from "@DurationJsonConverter()")
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
                // Original logic for non-converter fields
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
                    // Handle List<T> where T could be primitive or object
                    let is_list_nullable = field_type.ends_with('?');
                    let list_content = if is_list_nullable { 
                        &field_type[5..field_type.len()-2] // Remove "List<" and ">?"
                    } else { 
                        &field_type[5..field_type.len()-1] // Remove "List<" and ">"
                    };
                    
                    if !is_list_nullable {
                        // Non-nullable List
                        if list_content == "int" {
                            format!("{}: (json['{}'] as List<dynamic>).map((e) => (e as num).toInt()).toList()", field_name, field_name)
                        } else if list_content == "double" {
                            format!("{}: (json['{}'] as List<dynamic>).map((e) => (e as num).toDouble()).toList()", field_name, field_name)
                        } else if list_content == "String" {
                            format!("{}: (json['{}'] as List<dynamic>).map((e) => e as String).toList()", field_name, field_name)
                        } else {
                            // Assume it's an object with fromJson
                            format!("{}: (json['{}'] as List<dynamic>).map((e) => {}.fromJson(e as Map<String, dynamic>)).toList()", field_name, field_name, list_content)
                        }
                    } else {
                        // Nullable List
                        if list_content == "int" {
                            format!("{}: (json['{}'] as List<dynamic>?)?.map((e) => (e as num).toInt()).toList()", field_name, field_name)
                        } else if list_content == "double" {
                            format!("{}: (json['{}'] as List<dynamic>?)?.map((e) => (e as num).toDouble()).toList()", field_name, field_name)
                        } else if list_content == "String" {
                            format!("{}: (json['{}'] as List<dynamic>?)?.map((e) => e as String).toList()", field_name, field_name)
                        } else {
                            // Assume it's an object with fromJson
                            format!("{}: (json['{}'] as List<dynamic>?)?.map((e) => {}.fromJson(e as Map<String, dynamic>)).toList()", field_name, field_name, list_content)
                        }
                    }
                } else if field_type.starts_with("Map<") {
                    // Handle Map<K, V> - for now, treat as generic object
                    if is_nullable {
                        format!("{}: json['{}'] as Map<String, dynamic>?", field_name, field_name)
                    } else {
                        format!("{}: json['{}'] as Map<String, dynamic>", field_name, field_name)
                    }
                } else if field_type.starts_with("Set<") {
                    // Handle Set<T> - convert to list first
                    let inner_type = &field_type[4..field_type.len()-1]; // Remove "Set<" and ">"
                    let is_set_nullable = inner_type.ends_with('?');
                    let base_inner_type = if is_set_nullable { &inner_type[..inner_type.len()-1] } else { inner_type };
                    
                    if !is_nullable {
                        // Non-nullable Set
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
                        // Nullable Set
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
                } else {
                    // Handle objects with fromJson method
                    if is_nullable {
                        // Nullable object - use null check
                        let base_type = &field_type[..field_type.len()-1]; // Remove the '?'
                        format!("{}: json['{}'] == null ? null : {}.fromJson(json['{}'] as Map<String, dynamic>)", field_name, field_name, base_type, field_name)
                    } else {
                        // Non-nullable object
                        format!("{}: {}.fromJson(json['{}'] as Map<String, dynamic>)", field_name, field_type, field_name)
                    }
                }
            }
        })
        .collect();
    
    output.push_str(&from_json_args.join(",\n  "));
    output.push_str(");\n");
    output
}

/// Generate the toJson function implementation
fn generate_to_json_function(class_name: &str, all_fields: &[&Argument]) -> String {
    let mut output = String::new();
    output.push_str(&format!("Map<String, dynamic> _${}ToJson(_{} instance) => <String, dynamic>{{", 
        class_name, class_name));
    
    let to_json_fields: Vec<String> = all_fields
        .iter()
        .map(|arg| {
            let field_name = &arg.name;
            
            // Check if field has a JsonConverter decorator
            let json_converter = arg.annotations.iter().find(|annotation| {
                annotation.contains("JsonConverter")
            });
            
            if let Some(converter) = json_converter {
                // Extract converter name (e.g., "DurationJsonConverter" from "@DurationJsonConverter()")
                let converter_name = converter
                    .trim_start_matches('@')
                    .trim_end_matches("()")
                    .trim_end_matches('(');
                
                format!("  '{}': const {}().toJson(instance.{})", field_name, converter_name, field_name)
            } else {
                // Original logic for non-converter fields
                format!("  '{}': instance.{}", field_name, field_name)
            }
        })
        .collect();
    
    output.push_str(&format!("\n{}", to_json_fields.join(",\n")));
    output.push_str(",\n};\n");
    output
}

/// Generate the complete mixin for a single class
fn generate_single_mixin(class: &FreezedClass) -> String {
    let mut all_fields = Vec::new();
    all_fields.extend(&class.positional_arguments);
    all_fields.extend(&class.named_arguments);
    
    if all_fields.is_empty() {
        return String::new();
    }
    
    let mut output = String::new();
    
    // Generate mixin name and opening brace
    output.push_str(&format!("mixin _${} {{\n", class.name));
    
    // Generate getter declarations
    output.push_str(&generate_getters(&all_fields[..]));
    output.push('\n');
    
    // Generate equality operator
    output.push_str(&generate_equality_operator(&class.name, &all_fields[..]));
    
    // Generate hashCode
    output.push_str(&generate_hash_code(&all_fields[..]));
    
    // Generate toString
    output.push_str(&generate_to_string(&class.name, &all_fields[..]));
    
    // Generate copyWith method declaration in mixin
    output.push_str(&generate_copy_with_declaration(&class.name, &all_fields[..]));
    
    // Generate toJson method declaration in mixin (only if has_json is true)
    if class.has_json {
        output.push_str(&generate_to_json_declaration());
    }
    
    // Close the mixin
    output.push_str("}\n\n");
    
    // Generate the class implementation
    output.push_str(&format!("class _{} extends {} {{\n", class.name, class.name));
    
    // Generate constructor
    output.push_str(&generate_constructor(class));
    output.push_str("\n");
    
    // Generate final field declarations
    output.push_str(&generate_field_declarations(&all_fields[..]));
    
    // Generate copyWith method in the class
    output.push_str(&generate_copy_with_implementation(&class.name, &all_fields[..]));
    
    // Generate fromJson factory method for the class (only if has_json is true)
    if class.has_json {
        output.push_str(&generate_from_json_factory(&class.name));
    }
    
    // Generate toJson method in the class (only if has_json is true)
    if class.has_json {
        output.push_str(&generate_to_json_implementation(&class.name));
    }
    
    // Close the class
    output.push_str("}\n\n");
    
    // Generate JSON serialization functions outside the class (only if has_json is true)
    if class.has_json {
        output.push_str(&generate_from_json_function(&class.name, &all_fields[..]));
        output.push_str("\n");
        output.push_str(&generate_to_json_function(&class.name, &all_fields[..]));
        output.push_str("\n");
    }
    
    output
}

/// Generate mixin code with equality, hashCode, and toString methods for @freezed classes
/// 
/// # Arguments
/// * `code` - The Dart source code as a string
/// 
/// # Returns
/// * A string containing the generated mixin code and class implementation
/// 
/// # Example
/// ```
/// use faster_freezed::generate_mixin;
/// 
/// let dart_code = r#"
/// @freezed
/// abstract class Test with _$Test {
///   factory Test({required int i, @Default('hello') String data}) = _Test;
///   Test._();
///   factory Test.fromJson(Map<String, dynamic> json) => _$TestFromJson(json);
/// }
/// "#;
/// 
/// let mixin_code = generate_mixin(dart_code.to_string());
/// println!("{}", mixin_code);
/// ```
pub fn generate_mixin(code: String) -> String {
    let classes = parse_freezed_classes(code);
    let mut output = String::new();
    
    for class in &classes {
        output.push_str(&generate_single_mixin(class));
    }
    
    output
}