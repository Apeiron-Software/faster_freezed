pub mod dart_types;
pub mod json_serialization;
pub mod parser;

use dart_types::ClassDefinition;

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
pub fn parse_freezed_classes(code: String) -> Vec<ClassDefinition> {
    parser::parse_dart_code(&code)
}
