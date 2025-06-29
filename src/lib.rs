pub mod freezed_class;
pub mod parser;

use freezed_class::FreezedClass;

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
        // Generate getter declarations
        let mut all_fields = Vec::new();
        all_fields.extend(&class.positional_arguments);
        all_fields.extend(&class.named_arguments);
        
        if !all_fields.is_empty() {
            // Generate mixin name and opening brace
            output.push_str(&format!("mixin _${} {{\n", class.name));
            
            // Generate getter declarations
            let getters: Vec<String> = all_fields
                .iter()
                .map(|arg| format!("  {} get {};", arg.r#type, arg.name))
                .collect();
            output.push_str(&getters.join("\n"));
            output.push('\n');
            
            // Generate equality operator
            output.push_str("  @override\n");
            output.push_str("  bool operator ==(Object other) {\n");
            output.push_str("    return identical(this, other) ||\n");
            output.push_str("        (other.runtimeType == runtimeType &&\n");
            output.push_str(&format!("            other is {}\n", class.name));
            
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
            output.push_str("  }\n"); // DO NOT TOUCH THIS, THIS IS GOOD
            
            // Generate hashCode
            output.push_str("\n  @override\n");
            output.push_str("  int get hashCode => Object.hash(runtimeType, ");
            let hash_fields: Vec<String> = all_fields
                .iter()
                .map(|arg| arg.name.clone())
                .collect();
            output.push_str(&hash_fields.join(", "));
            output.push_str(");\n");
            
            // Generate toString
            output.push_str("\n  @override\n");
            output.push_str("  String toString() {\n");
            output.push_str(&format!("    return '{}({})';\n", 
                class.name, 
                all_fields.iter().map(|arg| format!("{}: ${}", arg.name, arg.name)).collect::<Vec<_>>().join(", ")
            ));
            output.push_str("  }\n");
            
            // Generate copyWith method declaration in mixin
            output.push_str(&format!("  {} copyWith({{", class.name));
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
            
            // Close the mixin
            output.push_str("}\n\n");
            
            // Generate the class implementation
            let const_keyword = if class.has_const_constructor { "const " } else { "" };
            output.push_str(&format!("class _{} extends {} {{\n", class.name, class.name));
            
            // Generate constructor
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
            
            output.push_str("\n");
            
            // Generate final field declarations
            for field in &all_fields {
                // println!("DEBUG FIELD: {}: {}", field.name, field.r#type); // Debug print
                output.push_str(&format!("  @override\n  final {} {};\n", field.r#type, field.name));
            }
            
            // Generate copyWith method in the class
            output.push_str("\n  @override\n");
            output.push_str(&format!("  {} copyWith({{", class.name));
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
            output.push_str(&format!("    return _{}(", class.name));
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
            
            // Close the class
            output.push_str("}\n\n");
        }
    }
    
    output
}