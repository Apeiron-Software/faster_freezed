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
                .map(|arg| format!("            && (identical(other.{0}, {0}) || other.{0} == {0})", arg.name))
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
                output.push_str(&format!("  {}const _{} ({{{}}}) : super._();\n", 
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
                output.push_str(&format!("  {} _{} ({}) : super._();\n", 
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
                output.push_str(&format!("  {}const _{} ({}, {{{}}}) : super._();\n", 
                    const_keyword, class.name, pos_params.join(", "), named_params.join(", ")));
            }
            
            output.push_str("\n");
            
            // Generate final field declarations
            for field in &all_fields {
                output.push_str(&format!("  @override\n  final {} {};\n", field.r#type, field.name));
            }
            
            // Generate copyWith method
            output.push_str("\n  @override\n");
            output.push_str(&format!("  {} copyWith({{", class.name));
            
            // Generate copyWith parameters
            let copy_with_params: Vec<String> = all_fields
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
            output.push_str(&copy_with_params.join(", "));
            output.push_str("}) {\n");
            
            // Generate copyWith body
            output.push_str(&format!("    return _{}(", class.name));
            
            let copy_with_args: Vec<String> = all_fields
                .iter()
                .map(|arg| {
                    let is_nullable = arg.r#type.contains('?');
                    if is_nullable {
                        format!("{}: freezed == {} ? this.{} : {} as {}", 
                            arg.name, arg.name, arg.name, arg.name, arg.r#type.trim_end_matches('?'))
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

#[cfg(test)]
mod tests {
    use super::*;
    use freezed_class::FreezedClass;

    fn parse_dart_code(code: &str) -> Vec<FreezedClass> {
        parser::parse_dart_code(code)
    }

    #[test]
    fn test_parse_simple_class() {
        let code = r#"
@freezed
class SimpleClass with _$SimpleClass {
  const factory SimpleClass(
    String name,
    int age,
  ) = _SimpleClass;
}
"#;
        
        let classes = parse_dart_code(code);
        assert_eq!(classes.len(), 1);
        
        let class = &classes[0];
        assert_eq!(class.name, "SimpleClass");
        assert_eq!(class.positional_arguments.len(), 2);
        assert_eq!(class.named_arguments.len(), 0);
        assert_eq!(class.has_json, false);
        assert_eq!(class.has_const_constructor, false);
        
        let name_arg = &class.positional_arguments[0];
        assert_eq!(name_arg.name, "name");
        assert_eq!(name_arg.r#type, "String");
        assert_eq!(name_arg.is_required, false);
        assert_eq!(name_arg.default_value, None);
        assert_eq!(name_arg.annotations.len(), 0);
        
        let age_arg = &class.positional_arguments[1];
        assert_eq!(age_arg.name, "age");
        assert_eq!(age_arg.r#type, "int");
        assert_eq!(age_arg.is_required, false);
        assert_eq!(age_arg.default_value, None);
        assert_eq!(age_arg.annotations.len(), 0);
    }

    #[test]
    fn test_parse_class_with_named_parameters() {
        let code = r#"
@freezed
class NamedClass with _$NamedClass {
  const factory NamedClass({
    required String firstName,
    required String lastName,
    int? age,
  }) = _NamedClass;
}
"#;
        
        let classes = parse_dart_code(code);
        assert_eq!(classes.len(), 1);
        
        let class = &classes[0];
        assert_eq!(class.name, "NamedClass");
        assert_eq!(class.positional_arguments.len(), 0);
        assert_eq!(class.named_arguments.len(), 3);
        assert_eq!(class.has_json, false);
        assert_eq!(class.has_const_constructor, false);
        
        let first_arg = &class.named_arguments[0];
        assert_eq!(first_arg.name, "firstName");
        assert_eq!(first_arg.r#type, "String");
        assert_eq!(first_arg.is_required, true);
        
        let last_arg = &class.named_arguments[1];
        assert_eq!(last_arg.name, "lastName");
        assert_eq!(last_arg.r#type, "String");
        assert_eq!(last_arg.is_required, true);
        
        let age_arg = &class.named_arguments[2];
        assert_eq!(age_arg.name, "age");
        assert_eq!(age_arg.r#type, "int?");
        assert_eq!(age_arg.is_required, false);
    }

    #[test]
    fn test_parse_class_with_annotations_and_defaults() {
        let code = r#"
@freezed
class AnnotatedClass with _$AnnotatedClass {
  const factory AnnotatedClass(
    @Default("John") String name,
    @JsonKey(name: 'user_age') required int age,
  ) = _AnnotatedClass;
}
"#;
        
        let classes = parse_dart_code(code);
        assert_eq!(classes.len(), 1);
        
        let class = &classes[0];
        assert_eq!(class.name, "AnnotatedClass");
        assert_eq!(class.positional_arguments.len(), 2);
        assert_eq!(class.has_const_constructor, false);
        
        let name_arg = &class.positional_arguments[0];
        assert_eq!(name_arg.name, "name");
        assert_eq!(name_arg.r#type, "String");
        assert_eq!(name_arg.default_value, Some("John".to_string()));
        assert_eq!(name_arg.annotations.len(), 1);
        assert!(name_arg.annotations[0].contains("@Default"));
        
        let age_arg = &class.positional_arguments[1];
        assert_eq!(age_arg.name, "age");
        assert_eq!(age_arg.r#type, "int");
        assert_eq!(age_arg.is_required, true);
        assert_eq!(age_arg.annotations.len(), 1);
        assert!(age_arg.annotations[0].contains("@JsonKey"));
    }

    #[test]
    fn test_parse_class_with_fromjson() {
        let code = r#"
@freezed
class JsonClass with _$JsonClass {
  const factory JsonClass({
    required String name,
    int age,
  }) = _JsonClass;

  factory JsonClass.fromJson(Map<String, Object?> json) => _$JsonClassFromJson(json);
}
"#;
        
        let classes = parse_dart_code(code);
        assert_eq!(classes.len(), 1);
        
        let class = &classes[0];
        assert_eq!(class.name, "JsonClass");
        assert_eq!(class.has_json, true);
        assert_eq!(class.named_arguments.len(), 2);
        assert_eq!(class.has_const_constructor, false);
    }

    #[test]
    fn test_parse_multiple_classes() {
        let code = r#"
@freezed
class Person with _$Person {
  const factory Person(
    int hello, {
    @Default("hi") String firstName,
    @RandomAnnotation() required String lastName,
    required int age,
  }) = _Person;

  factory Person.fromJson(Map<String, Object?> json) => _$PersonFromJson(json);
}

@freezed
class Alien with _$Alien {
  const factory Alien({
    required String firstName,
    required String lastName,
    required int age,
  }) = _Alien;

  factory Alien.fromJson(Map<String, Object?> json) => _$AlienFromJson(json);
}
"#;
        
        let classes = parse_dart_code(code);
        assert_eq!(classes.len(), 2);
        
        let person = &classes[0];
        assert_eq!(person.name, "Person");
        assert_eq!(person.has_json, true);
        assert_eq!(person.positional_arguments.len(), 1);
        assert_eq!(person.named_arguments.len(), 3);
        assert_eq!(person.has_const_constructor, false);
        
        let alien = &classes[1];
        assert_eq!(alien.name, "Alien");
        assert_eq!(alien.has_json, true);
        assert_eq!(alien.positional_arguments.len(), 0);
        assert_eq!(alien.named_arguments.len(), 3);
        assert_eq!(alien.has_const_constructor, false);
    }

    #[test]
    fn test_parse_parameter_function() {
        // Test the parse_parameter function directly
        let code = r#"
@freezed
class TestClass with _$TestClass {
  const factory TestClass(
    @Default("test") String name,
  ) = _TestClass;
}
"#;
        
        let classes = parse_dart_code(code);
        assert_eq!(classes.len(), 1);
        
        let class = &classes[0];
        assert_eq!(class.name, "TestClass");
        assert_eq!(class.positional_arguments.len(), 1);
        assert_eq!(class.has_const_constructor, false);
        
        let name_arg = &class.positional_arguments[0];
        assert_eq!(name_arg.name, "name");
        assert_eq!(name_arg.r#type, "String");
        assert_eq!(name_arg.default_value, Some("test".to_string()));
        assert_eq!(name_arg.annotations.len(), 1);
        assert!(name_arg.annotations[0].contains("@Default"));
    }

    #[test]
    fn test_parse_class_with_const_constructor() {
        let code = r#"
@freezed
class ConstClass with _$ConstClass {
  const factory ConstClass({
    required String name,
    int age,
  }) = _ConstClass;
  
  const ConstClass._();
}
"#;
        
        let classes = parse_dart_code(code);
        assert_eq!(classes.len(), 1);
        
        let class = &classes[0];
        assert_eq!(class.name, "ConstClass");
        assert_eq!(class.has_json, false);
        assert_eq!(class.named_arguments.len(), 2);
        assert_eq!(class.has_const_constructor, true);
        
        let name_arg = &class.named_arguments[0];
        assert_eq!(name_arg.name, "name");
        assert_eq!(name_arg.r#type, "String");
        assert_eq!(name_arg.is_required, true);
        
        let age_arg = &class.named_arguments[1];
        assert_eq!(age_arg.name, "age");
        assert_eq!(age_arg.r#type, "int");
        assert_eq!(age_arg.is_required, false);
    }

    #[test]
    fn test_generate_mixin() {
        let code = r#"
@freezed
abstract class Test with _$Test {
  factory Test({required int i, @Default('hello') String data}) = _Test;
  Test._();
  factory Test.fromJson(Map<String, dynamic> json) => _$TestFromJson(json);
}
"#;
        
        let mixin_code = generate_mixin(code.to_string());
        
        // Check that the mixin contains the expected elements
        assert!(mixin_code.contains("mixin _$Test {"));
        assert!(mixin_code.contains("  int get i;"));
        assert!(mixin_code.contains("  String get data;"));
        assert!(mixin_code.contains("  bool operator =="));
        assert!(mixin_code.contains("other is Test"));
        assert!(mixin_code.contains("&& (identical(other.i, i) || other.i == i)"));
        assert!(mixin_code.contains("&& (identical(other.data, data) || other.data == data)"));
        assert!(mixin_code.contains("int get hashCode => Object.hash(runtimeType, i, data);"));
        assert!(mixin_code.contains("String toString()"));
        assert!(mixin_code.contains("return 'Test(i: $i, data: $data)';"));
        assert!(mixin_code.contains("}"));
        
        // Check that the class implementation contains the expected elements
        assert!(mixin_code.contains("class _Test extends Test {"));
        assert!(mixin_code.contains("const _Test ({required this.i, this.data = 'hello'}) : super._();"));
        assert!(mixin_code.contains("@override\n  final int i;"));
        assert!(mixin_code.contains("@override\n  final String data;"));
    }

    #[test]
    fn test_generate_mixin_with_const_constructor() {
        let code = r#"
@freezed
abstract class ConstTest with _$ConstTest {
  factory ConstTest({required int i, @Default('hello') String data}) = _ConstTest;
  const ConstTest._();
  factory ConstTest.fromJson(Map<String, dynamic> json) => _$ConstTestFromJson(json);
}
"#;
        
        let mixin_code = generate_mixin(code.to_string());
        
        // Check that the class implementation contains const keyword
        assert!(mixin_code.contains("class _ConstTest extends ConstTest {"));
        assert!(mixin_code.contains("const _ConstTest ({required this.i, this.data = 'hello'}) : super._();"));
        assert!(mixin_code.contains("@override\n  final int i;"));
        assert!(mixin_code.contains("@override\n  final String data;"));
    }

    #[test]
    fn test_generate_mixin_with_copywith() {
        let code = r#"
@freezed
abstract class CopyWithTest with _$CopyWithTest {
  factory CopyWithTest({required int i, String? data}) = _CopyWithTest;
  CopyWithTest._();
}
"#;
        
        let mixin_code = generate_mixin(code.to_string());
        
        // Check that the copyWith method is generated
        assert!(mixin_code.contains("CopyWithTest copyWith({"));
        assert!(mixin_code.contains("Object? i"));
        assert!(mixin_code.contains("Object? data = freezed"));
        assert!(mixin_code.contains("return _CopyWithTest("));
        assert!(mixin_code.contains("i: i == null ? this.i : i as int"));
        assert!(mixin_code.contains("data: freezed == data ? this.data : data as String"));
    }
} 