use crate::dart_types::{
    Annotation, ClassDefinition, DartType, NamedParameter, ParameterList, PositionalParameter,
    RedirectedConstructor,
};
use lazy_static::lazy_static;
use streaming_iterator::StreamingIterator;
use tree_sitter::{Parser, QueryCursor};

const FREEZED_CLASS: &str = "(class_definition (annotation) @freezed_marker) @class_definition";
const CLASS_MEMBER_DEFINITION: &str =
    "((declaration) @member)\n((factory_constructor_signature) @member)";

unsafe extern "C" {
    pub fn tree_sitter_dart() -> tree_sitter::Language;
}

lazy_static! {
    pub static ref DART_TS: tree_sitter::Language = unsafe { tree_sitter_dart() };
    static ref freezed_class_q: tree_sitter::Query =
        tree_sitter::Query::new(&DART_TS, FREEZED_CLASS).expect("hardcoded");
    static ref members_q: tree_sitter::Query =
        tree_sitter::Query::new(&DART_TS, CLASS_MEMBER_DEFINITION).expect("hardcoded");
}

pub fn get_text(node: tree_sitter::Node, code: &str) -> String {
    node.utf8_text(code.as_bytes()).unwrap().to_owned()
}

/// Parse Dart code and extract all classes with @freezed annotation
pub fn parse_dart_code(code: &str) -> Vec<ClassDefinition> {
    let mut parser = Parser::new();
    parser
        .set_language(&DART_TS)
        .expect("Error loading Dart grammar");
    let tree = parser.parse(code, None).unwrap();

    let mut query_cursor = QueryCursor::new();
    let mut output_freezed_classes = Vec::new();

    let mut class_with_annotation_matches =
        query_cursor.matches(&freezed_class_q, tree.root_node(), code.as_bytes());

    while let Some(class_with_annotation) = class_with_annotation_matches.next() {
        let _freezed_annotation = class_with_annotation
            .nodes_for_capture_index(0)
            .next()
            .unwrap();

        let mut gen_form = false;
        if let Some(element) = _freezed_annotation.next_sibling()
            && element
                .utf8_text(code.as_bytes())
                .unwrap()
                .contains("@qform")
        {
            gen_form = true;
        }

        if get_text(_freezed_annotation, code) != "@freezed" {
            continue;
        }

        // We can get multiple body matches, but there's only one body
        let class_declaration = class_with_annotation
            .nodes_for_capture_index(1)
            .next()
            .unwrap();

        let class_name = class_declaration
            .child_by_field_name("name")
            .unwrap()
            .utf8_text(code.as_bytes())
            .unwrap();

        // FEAT: support multiple mixins
        let mixin = 'mixin: {
            let superclass = class_declaration.child_by_field_name("superclass").unwrap();
            let mut superclass_cursor = superclass.walk();
            for child in superclass.children(&mut superclass_cursor) {
                if child.kind() == "mixins" {
                    let (dart_type, _skip) = parse_type(child.named_child(0).unwrap(), code);
                    break 'mixin Some(dart_type);
                }
            }
            None
        };

        let mut redirecting_constructors = Vec::new();
        let mut json_constructor = None;
        let mut unnamed_constructor = None;
        let mut class_query = QueryCursor::new();

        let mut executed_query =
            class_query.matches(&members_q, class_declaration, code.as_bytes());

        while let Some(inner_declaration_group) = executed_query.next() {
            assert!(inner_declaration_group.captures.len() == 1);
            let capture = inner_declaration_group.captures.first().unwrap();
            let x = parse_class_declaration(capture.node, code);
            if let Some(y) = x {
                match y {
                    DeclarationParseResult::Redirected(redirected_constructor) => {
                        redirecting_constructors.push(redirected_constructor);
                    }
                    DeclarationParseResult::Json(redirected_constructor) => {
                        json_constructor = Some(redirected_constructor);
                    }
                    DeclarationParseResult::Unnamed(redirected_constructor) => {
                        unnamed_constructor = Some(redirected_constructor);
                    }
                }
            }
        }


        let freezed_class = ClassDefinition {
            name: class_name.to_string(),
            gen_form,
            mixins: mixin.into_iter().collect(),
            redirecting_constructors,
            json_constructor,
            unnamed_constructor,
        };

        output_freezed_classes.push(freezed_class);
    }

    output_freezed_classes
}

enum RedirectingFactoryItems {
    Const,
    FactoryKeyword,
    ClassName,
    ConstructorName,
    FormalParameterList,
    AssignedConstructor,
}

enum DeclarationParseResult {
    Redirected(RedirectedConstructor),
    Json(RedirectedConstructor),
    Unnamed(RedirectedConstructor),
}

fn parse_class_declaration(node: tree_sitter::Node, code: &str) -> Option<DeclarationParseResult> {
    let first_child = node.child(0).unwrap();

    if first_child.kind() == "redirecting_factory_constructor_signature" {
        assert!(node.child_count() == 1);
        let redirecting_constructor = first_child;

        // redirecting_factory_constructor_signature: $ => seq(
        //     optional($.const_builtin),
        //     $._factory,
        //     sep1($.identifier, '.'),
        //     $.formal_parameter_list,
        //     '=',
        //     $._type_not_void,
        //     optional(seq('.', $.identifier)),
        // ),
        let mut is_const = false;
        let mut class_name = String::new();
        let mut constructor_name: Option<String> = None;
        let mut assigned_type: Option<DartType> = None;

        let mut arguments = ParameterList::default();
        let mut stage = RedirectingFactoryItems::Const;
        let mut cursor = redirecting_constructor.walk();

        let mut was_processed = 0;
        for child in redirecting_constructor.children(&mut cursor) {
            while was_processed == 0 {
                match stage {
                    RedirectingFactoryItems::Const => {
                        if child.kind() == "const_builtin" {
                            is_const = true;
                            was_processed = 1;
                        }
                        stage = RedirectingFactoryItems::FactoryKeyword;
                    }
                    RedirectingFactoryItems::FactoryKeyword => {
                        assert!(child.kind() == "factory");
                        was_processed = 1;
                        stage = RedirectingFactoryItems::ClassName;
                    }
                    RedirectingFactoryItems::ClassName => {
                        assert!(child.kind() == "identifier");
                        class_name = child.utf8_text(code.as_bytes()).unwrap().to_owned();
                        was_processed = 1;
                        stage = RedirectingFactoryItems::ConstructorName;
                    }
                    RedirectingFactoryItems::ConstructorName => {
                        if child.kind() == "." {
                            was_processed = 1;
                        } else if child.kind() == "identifier" {
                            constructor_name =
                                Some(child.utf8_text(code.as_bytes()).unwrap().to_owned());
                            was_processed = 1;
                            stage = RedirectingFactoryItems::FormalParameterList;
                        } else if child.kind() == "formal_parameter_list" {
                            stage = RedirectingFactoryItems::FormalParameterList;
                        } else {
                            panic!("Wrong syntax");
                        }
                    }
                    RedirectingFactoryItems::FormalParameterList => {
                        arguments = parse_formal_parameter_list(child, code);
                        was_processed = 1;
                        stage = RedirectingFactoryItems::AssignedConstructor;
                    }
                    RedirectingFactoryItems::AssignedConstructor => {
                        if child.kind() == "=" {
                            was_processed = 1;
                        } else {
                            let (dart_type, skipped) = parse_type(child, code);
                            assigned_type = Some(dart_type);
                            was_processed = skipped;
                        }
                    }
                }
            }
            was_processed -= 1;
        }
        let constructor = RedirectedConstructor {
            class_name,
            is_const,
            constructor_name,
            parameters: arguments,
            assigned_type: assigned_type.unwrap(),
        };
        return Some(DeclarationParseResult::Redirected(constructor));
    } else if node.kind() == "factory_constructor_signature" {
        if let Some(child) = node.named_child(1)
            && get_text(child, code) == "fromJson"
        {}
        let constructor = RedirectedConstructor {
            class_name: "fromJson".to_string(),
            ..Default::default()
        };

        return Some(DeclarationParseResult::Json(constructor));
    } else if ["constructor_signature", "constant_constructor_signature"]
        .contains(&first_child.kind())
    {
        let is_const = first_child.kind() == "constant_constructor_signature";
        if is_const {
            assert_eq!(get_text(first_child.named_child(2).unwrap(), code), "_");
        } else {
            assert_eq!(get_text(first_child.named_child(1).unwrap(), code), "_");
        }

        let constructor = RedirectedConstructor {
            class_name: "_".to_string(),
            is_const,
            ..Default::default()
        };

        return Some(DeclarationParseResult::Unnamed(constructor));
    } else if node.kind() == "declaration" {
        return None;
    }

    todo!(
        "Unimplemented: first: {}, node: {}",
        first_child.kind(),
        node.kind()
    );
}

// FORMAL PARAMETER GRAMMAR
// seq(
//     optional(
//         $._metadata
//     ),
//     optional(
//         $._required
//     ),
//     $.formal_parameter,
//     optional(
//         seq(
//             '=',
//             $._expression
//         )
//     )
// ),
//
// _normal_formal_parameter: $ => seq(
//     optional(
//         $._metadata
//     ), // Can't have metadata as argument here
//     choice(
//         $._function_formal_parameter,
//         $._simple_formal_parameter,
//         $.constructor_param,
//         $.super_formal_parameter
//     )
// ),

#[derive(Debug, Clone, Copy)]
enum FormalParameterSteps {
    OpenBracket,
    Annotations,
    Required,
    FormalParameter,
    DefaultValue,
    CloseBracket,
    Finish,
}

fn parse_formal_parameter_list(node: tree_sitter::Node, code: &str) -> ParameterList {
    assert!(node.kind() == "formal_parameter_list");
    let mut positional_parameters: Vec<PositionalParameter> = Vec::new();
    let mut named_parameters: Vec<NamedParameter> = Vec::new();

    // Parsing regular formal parameters,
    // It's just a list of `$.formal_parameter`
    let mut current_node = node.named_child(0);

    while let Some(child) = current_node {
        if child.kind() == "formal_parameter" {
            positional_parameters.push(parse_formal_parameter(child, code));
            current_node = child.next_named_sibling();
        } else {
            assert!(child.kind() == "optional_formal_parameters");
            break;
        }
    }

    if let Some(optional_node) = current_node {
        assert_eq!(optional_node.kind(), "optional_formal_parameters");

        let mut cursor = optional_node.walk();
        let mut current_state = FormalParameterSteps::OpenBracket;

        let mut processing_argument = false;

        let mut current_argument: Option<NamedParameter> = None;

        for child in optional_node.children(&mut cursor) {
            let mut was_processed = false;
            while !was_processed {
                if child.kind() == "comment" {
                    break;
                }

                match current_state {
                    FormalParameterSteps::OpenBracket => {
                        assert_eq!(child.kind(), "{");
                        processing_argument = true;
                        was_processed = true;
                        current_state = FormalParameterSteps::Annotations;
                    }
                    FormalParameterSteps::Annotations => {
                        assert!(processing_argument);

                        if child.kind() == "}" {
                            was_processed = true;
                            current_state = FormalParameterSteps::CloseBracket;
                            continue;
                        }

                        if current_argument.is_none() {
                            current_argument = Some(NamedParameter::default());
                        }

                        if child.kind() == "annotation" {
                            was_processed = true;
                            let name_node = child.child_by_field_name("name").unwrap();
                            let mut argument_node = name_node
                                .next_named_sibling()
                                .and_then(|e| e.named_child(0));

                            let name = get_text(name_node, code);
                            let mut arguments = Vec::new();
                            while let Some(arg) = argument_node {
                                arguments.push(get_text(arg, code));
                                argument_node = arg.next_named_sibling();
                            }

                            current_argument
                                .as_mut()
                                .unwrap()
                                .annotations
                                .push(Annotation { name, arguments });
                        } else {
                            was_processed = false;
                            current_state = FormalParameterSteps::Required;
                        }
                    }
                    FormalParameterSteps::Required => {
                        if child.kind() == "required" {
                            current_argument.as_mut().unwrap().is_required = true;
                            was_processed = true;
                        }
                        current_state = FormalParameterSteps::FormalParameter;
                    }
                    FormalParameterSteps::FormalParameter => {
                        assert_eq!(child.kind(), "formal_parameter");
                        let x = parse_formal_parameter(child, code);

                        current_argument.as_mut().unwrap().name = x.name;
                        current_argument.as_mut().unwrap().dart_type = x.dart_type;
                        current_argument
                            .as_mut()
                            .unwrap()
                            .annotations
                            .extend(x.annotations);

                        was_processed = true;
                        current_state = FormalParameterSteps::DefaultValue;
                    }
                    FormalParameterSteps::DefaultValue => {
                        if child.kind() == "=" {
                            was_processed = true;
                        } else if child.kind() == "," {
                            was_processed = true;
                            current_state = FormalParameterSteps::Annotations;
                            named_parameters.push(current_argument.unwrap());
                            current_argument = None;
                        } else if child.kind() == "}" {
                            was_processed = false;
                            current_state = FormalParameterSteps::CloseBracket;
                        } else {
                            // TODO, default value
                            unreachable!("What the fuck are you: {}", child.kind());
                        }
                    }
                    FormalParameterSteps::CloseBracket => {
                        assert_eq!(child.kind(), "}");
                        was_processed = true;
                        current_state = FormalParameterSteps::Finish;
                    }
                    FormalParameterSteps::Finish => unreachable!(),
                }
            }
        }

        if let Some(argument) = current_argument {
            named_parameters.push(argument);
        }
        /* current_argument = None; */
    }

    ParameterList {
        positional_parameters,
        named_parameters,
    }
}

fn parse_formal_parameter(node: tree_sitter::Node, code: &str) -> PositionalParameter {
    assert_eq!(node.kind(), "formal_parameter");

    let current_node = node.named_child(0).unwrap();
    let annotations = parse_annotations(current_node, code);
    let mut total_skip = annotations.len();

    let (argument_type, skipped) = parse_type(node.named_child(total_skip).unwrap(), code);
    total_skip += skipped;
    let identifier = node.named_child(total_skip).unwrap();

    assert_eq!(identifier.kind(), "identifier");
    let argument_name = get_text(identifier, code);

    PositionalParameter {
        name: argument_name,
        dart_type: argument_type,
        annotations,
    }
}

//  - Function type ???
//  - _typeName, arguments, nullable
//  - Record

fn parse_annotations(node: tree_sitter::Node, code: &str) -> Vec<Annotation> {
    if node.kind() != "annotation" {
        return Vec::new();
    }

    assert_eq!(node.kind(), "annotation");
    let mut annotations = Vec::new();
    let mut cursor_node = Some(node);

    while let Some(node) = cursor_node {
        if node.kind() == "annotation" {
            let name_node = node.child_by_field_name("name").unwrap();
            let mut argument_node = name_node
                .next_named_sibling()
                .and_then(|e| e.named_child(0));

            let name = get_text(name_node, code);
            let mut arguments = Vec::new();
            while let Some(arg) = argument_node {
                arguments.push(get_text(arg, code));
                argument_node = arg.next_named_sibling();
            }
            annotations.push(Annotation { name, arguments });
            cursor_node = node.next_named_sibling();
        } else {
            break;
        }
    }

    annotations
}

fn parse_type(node: tree_sitter::Node, code: &str) -> (DartType, usize) {
    let mut processed = 0;
    let mut name: String = String::new();
    let mut type_arguments = Vec::new();
    let mut current_node = Some(node);
    let mut nullable = false;

    if let Some(node) = current_node
        && node.kind() == "record_type"
    {
        // TODO
        name = get_text(node, code);

        processed += 1;
        current_node = node.next_named_sibling();
    } else if let Some(node) = current_node
        && node.kind() == "type_identifier"
    {
        name = get_text(node, code);

        processed += 1;
        current_node = node.next_named_sibling();
    }

    if let Some(node) = current_node
        && node.kind() == "type_arguments"
    {
        //let arguments_node = current_node.named_child(0).unwrap();
        let mut current_child = 0;

        while current_child < node.named_child_count() {
            let (dart_type, skipped) = parse_type(node.named_child(current_child).unwrap(), code);
            type_arguments.push(dart_type);
            current_child += skipped;
        }
        processed += 1;
        current_node = node.next_named_sibling();
    }

    if let Some(node) = current_node
        && node.kind() == "nullable_type"
    {
        processed += 1;
        nullable = true;
        //current_node = node.next_named_sibling().unwrap();
    }

    (
        DartType {
            name,
            nullable,
            type_arguments,
        },
        processed,
    )
}

#[allow(unused)]
fn print_ts_element(node: tree_sitter::Node, code: &str, depth: usize) {
    let indent = "  ".repeat(depth);
    let node_text = node.utf8_text(code.as_bytes()).unwrap_or("");
    println!(
        "{}[TS] kind: {}, text: '{}'",
        indent,
        node.kind(),
        node_text,
    );
}

#[allow(unused)]
fn print_ts_tree(node: tree_sitter::Node, code: &str, depth: usize) {
    let indent = "  ".repeat(depth);
    let node_text = node.utf8_text(code.as_bytes()).unwrap_or("");
    println!(
        "{}[TS] kind: {}, text: '{}'",
        indent,
        node.kind(),
        node_text,
    );

    for i in 0..node.named_child_count() {
        if let Some(child) = node.named_child(i) {
            print_ts_tree(child, code, depth + 1);
        }
    }
}

// fn print_ts_tree_unnamed(node: tree_sitter::Node, text: &str, depth: usize) {
//     let indent = "  ".repeat(depth);
//     let node_text = node.utf8_text(text.as_bytes()).unwrap_or("");
//     println!(
//         "{}[TS] kind: {}, text: '{}'",
//         indent,
//         node.kind(),
//         node_text,
//     );
//
//     for i in 0..node.child_count() {
//         if let Some(child) = node.child(i) {
//             print_ts_tree_unnamed(child, text, depth + 1);
//         }
//     }
// }

/// Print the tree-sitter parse tree for the given Dart code (for debugging)
pub fn print_ts_tree_for_code(code: &str) {
    let mut parser = Parser::new();
    parser
        .set_language(&DART_TS)
        .expect("Error loading Dart grammar");
    let tree = parser.parse(code, None).unwrap();
    let root = tree.root_node();
    print_ts_tree(root, code, 0);
}
