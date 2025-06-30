use crate::freezed_class::{Argument, FreezedClass};
use lazy_static::lazy_static;
use streaming_iterator::StreamingIterator;
use tree_sitter::{Parser, QueryCursor};

const FREEZED_CLASS: &str =
    "((marker_annotation) @freezed_marker . (class_definition) @class_definition)";
const CLASS_MEMBER_DEFINITION: &str = "(class_member_definition) @member";
const REDIRECTING_FACTORY_CONSTRUCTOR_SIGNATURE: &str =
    "(redirecting_factory_constructor_signature) @constructor";
const FORMAL_PARAMETER: &str = "(formal_parameter) @parameter";
const UNNAMED_CONSTRUCTOR: &str = "(constructor_signature) @unnamed_constructor";

lazy_static! {
    static ref DART_TS: tree_sitter::Language = tree_sitter_dart::language();
    static ref freezed_class_q: tree_sitter::Query =
        tree_sitter::Query::new(&DART_TS, FREEZED_CLASS).expect("hardcoded");
    static ref members_q: tree_sitter::Query =
        tree_sitter::Query::new(&DART_TS, CLASS_MEMBER_DEFINITION).expect("hardcoded");
    static ref redirecting_factory_q: tree_sitter::Query = tree_sitter::Query::new(
        &tree_sitter_dart::language(),
        REDIRECTING_FACTORY_CONSTRUCTOR_SIGNATURE,
    )
    .expect("hardcoded");
    static ref formal_parameter_q: tree_sitter::Query =
        tree_sitter::Query::new(&DART_TS, FORMAL_PARAMETER).expect("hardcoded");
    static ref unnamed_constructor_q: tree_sitter::Query =
        tree_sitter::Query::new(&DART_TS, UNNAMED_CONSTRUCTOR).expect("hardcoded");
}

/// Parse Dart code and extract all classes with @freezed annotation
pub fn parse_dart_code(code: &str) -> Vec<FreezedClass> {
    let mut parser = Parser::new();
    parser
        .set_language(&DART_TS)
        .expect("Error loading Dart grammar");
    let tree = parser.parse(code, None).unwrap();

    let mut query_cursor = QueryCursor::new();
    let mut freezed_classes = Vec::new();

    let mut test = query_cursor.matches(&freezed_class_q, tree.root_node(), code.as_bytes());

    while let Some(class_definition) = test.next() {
        let _freezed_annotation = class_definition.nodes_for_capture_index(0).next().unwrap();
        let class_body = class_definition.nodes_for_capture_index(1).next().unwrap();
        let class_name = class_body
            .child_by_field_name("name")
            .unwrap()
            .utf8_text(code.as_bytes())
            .unwrap();

        let mut positional_arguments = Vec::new();
        let mut named_arguments = Vec::new();
        let mut has_json = false;
        let mut has_const_constructor = false;

        let mut class_query = QueryCursor::new();
        let mut executed_query = class_query.matches(&members_q, class_body, code.as_bytes());

        while let Some(declaration) = executed_query.next() {
            let declaration_node = declaration.nodes_for_capture_index(0).next().unwrap();
            let declaration_text = declaration_node.utf8_text(code.as_bytes()).unwrap();

            // Check if this is a fromJson constructor
            if declaration_text.contains("fromJson") {
                has_json = true;
                continue;
            }

            // Check for unnamed constructor (._())
            let mut unnamed_constructor_cursor = QueryCursor::new();
            let mut unnamed_constructor_matches = unnamed_constructor_cursor.matches(
                &unnamed_constructor_q,
                declaration_node,
                code.as_bytes(),
            );

            if let Some(_unnamed_constructor) = unnamed_constructor_matches.next() {
                // Check if the constructor has the const keyword
                if declaration_text.contains("const") {
                    has_const_constructor = true;
                }
                continue;
            }

            // Alternative approach: look for the pattern "const ClassName._()"
            if declaration_text.contains("const") && declaration_text.contains("._()") {
                has_const_constructor = true;
                continue;
            }

            let mut redirecting_factory = QueryCursor::new();
            let mut query = redirecting_factory.matches(
                &redirecting_factory_q,
                declaration_node,
                code.as_bytes(),
            );

            if let Some(factory) = query.next() {
                // If the factory declaration is const, set has_const_constructor = true
                if declaration_text.contains("const factory") {
                    has_const_constructor = true;
                }
                let factory_node = factory.nodes_for_capture_index(0).next().unwrap();
                let mut parameter_cursor = QueryCursor::new();
                let mut parameter_matches =
                    parameter_cursor.matches(&formal_parameter_q, factory_node, code.as_bytes());

                while let Some(parameter_match) = parameter_matches.next() {
                    let parameter_node = parameter_match.nodes_for_capture_index(0).next().unwrap();

                    if let Some(argument) = parse_parameter(parameter_node, code.as_bytes()) {
                        println!("{:?}", argument);
                        let parameter_start = parameter_node.start_byte();
                        let factory_text_bytes =
                            factory_node.utf8_text(code.as_bytes()).unwrap_or("");

                        let brace_start = factory_text_bytes.find('{');
                        let brace_end = factory_text_bytes.rfind('}');

                        let is_named = if let (Some(start), Some(end)) = (brace_start, brace_end) {
                            let relative_start = parameter_start - factory_node.start_byte();
                            relative_start > start && relative_start < end
                        } else {
                            false
                        };

                        if is_named {
                            named_arguments.push(argument);
                        } else {
                            positional_arguments.push(argument);
                        }
                    }
                }
            }
        }

        let freezed_class = FreezedClass {
            name: class_name.to_string(),
            positional_arguments,
            optional_arguments: Vec::new(),
            named_arguments,
            has_json,
            has_const_constructor,
        };

        freezed_classes.push(freezed_class);
    }

    freezed_classes
}

fn extract_type_string(node: tree_sitter::Node, text: &[u8]) -> String {
    let kind = node.kind();
    let node_text = node.utf8_text(text).unwrap_or("");
    println!(
        "DEBUG extract_type_string: kind={} text={}",
        kind, node_text
    );
    match kind {
        "type" => {
            let mut result = String::new();
            for i in 0..node.child_count() {
                if let Some(child) = node.child(i) {
                    let child_str = extract_type_string(child, text);
                    result.push_str(&child_str);
                }
            }
            // Check if there's a '?' sibling after this type node
            if let Some(parent) = node.parent() {
                for i in 0..parent.child_count() {
                    if let Some(sibling) = parent.child(i) {
                        if sibling.kind() == "?" {
                            // Check if this '?' comes after our type node
                            if sibling.start_byte() > node.end_byte() {
                                result.push('?');
                                break;
                            }
                        }
                    }
                }
            }
            result
        }
        "type_identifier" | "identifier" => {
            let mut result = node_text.to_string();
            // Check if there's a '?' sibling after this identifier
            if let Some(parent) = node.parent() {
                for i in 0..parent.child_count() {
                    if let Some(sibling) = parent.child(i) {
                        if sibling.kind() == "?" {
                            // Check if this '?' comes after our identifier
                            if sibling.start_byte() > node.end_byte() {
                                result.push('?');
                                break;
                            }
                        }
                    }
                }
            }
            result
        }
        "type_arguments" => {
            let mut result = String::from("<");
            let mut first = true;
            for i in 0..node.child_count() {
                if let Some(child) = node.child(i) {
                    if !first {
                        result.push_str(", ");
                    }
                    result.push_str(&extract_type_string(child, text));
                    first = false;
                }
            }
            result.push('>');
            result
        }
        _ => {
            // Fallback: concatenate all children
            let mut result = String::new();
            for i in 0..node.child_count() {
                if let Some(child) = node.child(i) {
                    result.push_str(&extract_type_string(child, text));
                }
            }
            result
        }
    }
}

fn parse_parameter(parameter_node: tree_sitter::Node, text: &[u8]) -> Option<Argument> {
    let mut annotations = Vec::new();
    let mut name = String::new();
    let mut param_type = String::new();
    let mut default_value = None;
    let mut is_required = false;

    // Check if this parameter is marked as required by looking at siblings
    if let Some(parent) = parameter_node.parent() {
        for i in 0..parent.child_count() {
            if let Some(sibling) = parent.child(i) {
                if sibling.kind() == "required" {
                    // Check if this required keyword applies to our parameter
                    if let Some(next_sibling) = parent.child(i + 1) {
                        if next_sibling.id() == parameter_node.id() {
                            is_required = true;
                            break;
                        }
                    }
                }
            }
        }
    }

    // THIS IS WRONG
    // there's never a child by name "type"
    // Try to get the type from the type field first
    if let Some(type_field) = parameter_node.child_by_field_name("type") {
        let type_str = extract_type_string(type_field, text);
        if !type_str.is_empty() {
            param_type = type_str;
        }
    } else {
        // If no type field, try to reconstruct from type_identifier and type_arguments
        let mut type_str = String::new();
        for i in 0..parameter_node.child_count() {
            if let Some(child) = parameter_node.child(i) {
                if child.kind() == "type_identifier" {
                    let t = child.utf8_text(text).unwrap_or("");
                    if t != "required" && !t.starts_with('@') {
                        type_str.push_str(t);
                    }
                } else if child.kind() == "type_arguments" {
                    type_str.push_str(child.utf8_text(text).unwrap_or(""));
                }
            }
        }
        if !type_str.is_empty() {
            param_type = type_str;
        }
    }

    // Parse annotations and other elements
    for i in 0..parameter_node.child_count() {
        if let Some(child) = parameter_node.child(i) {
            match child.kind() {
                "annotation" => {
                    if let Ok(annotation_text) = child.utf8_text(text) {
                        // Parse the annotation content
                        if annotation_text.starts_with("@Default(") {
                            // Extract the default value from @Default("value")
                            if let Some(start) = annotation_text.find('(') {
                                if let Some(end) = annotation_text.rfind(')') {
                                    let default_val = &annotation_text[start + 1..end];
                                    // Remove quotes if present
                                    let default_val = default_val.trim_matches('"');
                                    default_value = Some(default_val.to_string());
                                }
                            }
                        }
                        annotations.push(annotation_text.to_string());
                    }
                }
                "type_identifier" => {
                    if param_type.is_empty() {
                        if let Ok(type_text) = child.utf8_text(text) {
                            // Skip "required" and annotations as type
                            if type_text != "required" && !type_text.starts_with('@') {
                                param_type = type_text.to_string();
                                // Check for nullable type (next sibling is '?')
                                if let Some(next_sibling) = parameter_node.child(i + 1) {
                                    if next_sibling.kind() == "?" {
                                        param_type.push('?');
                                    }
                                }
                            }
                        }
                    }
                }
                "generic_type" => {
                    // This is a generic type like List<String>, Map<String, int>, etc.
                    if let Ok(type_text) = child.utf8_text(text) {
                        param_type = type_text.to_string();
                        // Check for nullable type (next sibling is '?')
                        if let Some(next_sibling) = parameter_node.child(i + 1) {
                            if next_sibling.kind() == "?" {
                                param_type.push('?');
                            }
                        }
                    }
                }
                "type" => {
                    // This is a type node that might contain generic information
                    if param_type.is_empty() {
                        if let Ok(type_text) = child.utf8_text(text) {
                            param_type = type_text.to_string();
                            // Check for nullable type (next sibling is '?')
                            if let Some(next_sibling) = parameter_node.child(i + 1) {
                                if next_sibling.kind() == "?" {
                                    param_type.push('?');
                                }
                            }
                        }
                    }
                }
                "identifier" => {
                    if let Ok(name_text) = child.utf8_text(text) {
                        // Skip "required" and annotations as parameter name
                        if name_text != "required" && !name_text.starts_with('@') {
                            name = name_text.to_string();
                        }
                    }
                }
                "=" => {
                    // Default value follows
                    if let Some(next_sibling) = parameter_node.child(i + 1) {
                        if let Ok(default_text) = next_sibling.utf8_text(text) {
                            default_value = Some(default_text.to_string());
                        }
                    }
                }
                _ => {}
            }
        }
    }

    // If we found a "required" keyword in the parameter text itself, mark it as required
    let parameter_text = parameter_node.utf8_text(text).unwrap_or("");
    if parameter_text.contains("required") {
        is_required = true;
    }

    // If we still don't have a name or type, try to extract them from the parameter text
    if name.is_empty() || param_type.is_empty() {
        let parts: Vec<&str> = parameter_text.split_whitespace().collect();
        let mut skip_next = false;

        for part in parts.iter() {
            if skip_next {
                skip_next = false;
                continue;
            }

            if *part == "required" {
                is_required = true;
            } else if part.starts_with('@') {
                // Skip annotations and their content
                if part.contains('(') && !part.ends_with(')') {
                    skip_next = true;
                }
            } else if param_type.is_empty() && *part != "required" && !part.starts_with('@') {
                param_type = part.to_string();
            } else if name.is_empty()
                && *part != "required"
                && *part != param_type
                && !part.starts_with('@')
            {
                name = part.to_string();
            }
        }
    }

    // If we still don't have a complete type, try to extract the full type from the parameter text
    if param_type.is_empty() || param_type.len() < 3 {
        let parameter_text = parameter_node.utf8_text(text).unwrap_or("");
        // Look for patterns like "List<String>", "Map<String, int>", etc.
        let words: Vec<&str> = parameter_text.split_whitespace().collect();

        // First try specific collection types
        for word in &words {
            if (word.starts_with("List<") || word.starts_with("Map<") || word.starts_with("Set<"))
                && !word.starts_with("@")
            {
                param_type = word.to_string();
                break;
            }
        }

        // If still empty, try a more comprehensive approach
        if param_type.is_empty() {
            // Look for any word with angle brackets (generic types)
            for word in &words {
                if word.contains('<') && word.contains('>') && !word.starts_with('@') {
                    param_type = word.to_string();
                    break;
                }
            }
        }
    }

    if name.is_empty() || param_type.is_empty() {
        return None;
    }

    Some(Argument {
        annotations,
        name,
        r#type: param_type,
        default_value,
        is_required,
    })
}

