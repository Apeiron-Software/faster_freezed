use crate::freezed_class::{
    Annotation, DartType, FreezedClass2, NamedArgument, RedirectedConstructor,
};
use lazy_static::lazy_static;
use streaming_iterator::StreamingIterator;
use tree_sitter::{Parser, QueryCursor};

const FREEZED_CLASS: &str = "(class_definition (annotation) @freezed_marker) @class_definition";
const CLASS_MEMBER_DEFINITION: &str =
    "((declaration) @member)\n((factory_constructor_signature) @member)";
const REDIRECTING_FACTORY_CONSTRUCTOR_SIGNATURE: &str =
    "(redirecting_factory_constructor_signature) @constructor";
const FORMAL_PARAMETER: &str = "(formal_parameter) @parameter";
const UNNAMED_CONSTRUCTOR: &str = "(constructor_signature) @unnamed_constructor";

unsafe extern "C" {
    pub fn tree_sitter_dart() -> tree_sitter::Language;
}

lazy_static! {
    pub static ref DART_TS: tree_sitter::Language = unsafe { tree_sitter_dart() };
    static ref freezed_class_q: tree_sitter::Query =
        tree_sitter::Query::new(&DART_TS, FREEZED_CLASS).expect("hardcoded");
    static ref members_q: tree_sitter::Query =
        tree_sitter::Query::new(&DART_TS, CLASS_MEMBER_DEFINITION).expect("hardcoded");
    static ref redirecting_factory_q: tree_sitter::Query =
        tree_sitter::Query::new(&DART_TS, REDIRECTING_FACTORY_CONSTRUCTOR_SIGNATURE,)
            .expect("hardcoded");
    static ref formal_parameter_q: tree_sitter::Query =
        tree_sitter::Query::new(&DART_TS, FORMAL_PARAMETER).expect("hardcoded");
    static ref unnamed_constructor_q: tree_sitter::Query =
        tree_sitter::Query::new(&DART_TS, UNNAMED_CONSTRUCTOR).expect("hardcoded");
}

pub fn get_text(node: tree_sitter::Node, code: &str) -> String {
    node.utf8_text(code.as_bytes()).unwrap().to_owned()
}

/// Parse Dart code and extract all classes with @freezed annotation
pub fn parse_dart_code(code: &str) -> Vec<FreezedClass2> {
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
        // We can get multiple matches, but we only get one freezed annotation
        let _freezed_annotation = class_with_annotation
            .nodes_for_capture_index(0)
            .next()
            .unwrap();
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

        let mut redirecting_constructors = Vec::new();
        let mut class_query = QueryCursor::new();

        let mut executed_query =
            class_query.matches(&members_q, class_declaration, code.as_bytes());

        while let Some(inner_declaration_group) = executed_query.next() {
            assert!(inner_declaration_group.captures.len() == 1);
            let capture = inner_declaration_group.captures.first().unwrap();
            let x = parse_class_declaration(capture.node, code);
            if let Some(y) = x {
                redirecting_constructors.push(y);
            }
        }

        let freezed_class = FreezedClass2 {
            name: class_name.to_string(),
            redirecting_constructors,
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

fn parse_class_declaration(node: tree_sitter::Node, code: &str) -> Option<RedirectedConstructor> {
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

        let mut arguments = Vec::<NamedArgument>::new();
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
        return Some(RedirectedConstructor {
            class_name,
            is_const,
            constructor_name,
            named_arguments: arguments,
            assigned_type: assigned_type.unwrap(),
        });
    } else if node.kind() == "factory_constructor_signature" {
        if let Some(child) = node.named_child(1)
            && get_text(child, code) == "fromJson"
        {}
        return Some(RedirectedConstructor {
            class_name: "fromJson".to_string(),
            is_const: false,
            constructor_name: Some("".to_string()),
            named_arguments: Vec::new(),
            assigned_type: DartType::default(),
        });
        // print_ts_tree(node, code, 0);
        // return RedirectedConstructor {
        //     class_name: "".to_string(),
        //     is_const: false,
        //     constructor_name: Some("".to_string()),
        //     named_arguments: Vec::new(),
        //     assigned_type: DartType::default(),
        // };
    } else if first_child.kind() == "constructor_signature" {
        assert_eq!(get_text(first_child.named_child(1).unwrap(), code), "_");
        // TODO, maybe fix??
        return Some(RedirectedConstructor {
            class_name: "_".to_string(),
            is_const: false,
            constructor_name: Some("".to_string()),
            named_arguments: Vec::new(),
            assigned_type: DartType::default(),
        });
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

fn parse_formal_parameter_list(node: tree_sitter::Node, code: &str) -> Vec<NamedArgument> {
    assert!(node.kind() == "formal_parameter_list");

    // Only optional parameters for now
    let optional_node = node.named_child(0).unwrap();

    let mut cursor = optional_node.walk();
    let mut current_state = FormalParameterSteps::OpenBracket;

    let mut arguments: Vec<NamedArgument> = Vec::new();
    let mut processing_argument = false;

    let mut current_argument: Option<NamedArgument> = None;

    for child in optional_node.children(&mut cursor) {
        let mut was_processed = false;
        while !was_processed {
            match current_state {
                FormalParameterSteps::OpenBracket => {
                    assert_eq!(child.kind(), "{");
                    processing_argument = true;
                    was_processed = true;
                    current_state = FormalParameterSteps::Annotations;
                }
                FormalParameterSteps::Annotations => {
                    if current_argument.is_none() {
                        current_argument = Some(NamedArgument::default());
                    }

                    assert!(processing_argument);
                    if child.kind() == "}" {
                        was_processed = true;
                        current_state = FormalParameterSteps::CloseBracket;
                    } else if child.kind() == "annotation" {
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
                    current_argument.as_mut().unwrap().argument_type = x.argument_type;

                    was_processed = true;
                    current_state = FormalParameterSteps::DefaultValue;
                }
                FormalParameterSteps::DefaultValue => {
                    if child.kind() == "=" {
                        was_processed = true;
                    } else if child.kind() == "," {
                        was_processed = true;
                        current_state = FormalParameterSteps::Annotations;
                        arguments.push(current_argument.unwrap());
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
    arguments
}

fn parse_formal_parameter(node: tree_sitter::Node, code: &str) -> NamedArgument {
    assert_eq!(node.kind(), "formal_parameter");
    // DEBUG
    {
        println!("PARAMETER:");
        print_ts_element(node, code, 0);
        println!("TREE:");
        print_ts_tree(node, code, 0);
    }

    let (argument_type, skipped) = parse_type(node.named_child(0).unwrap(), code);
    let identifier = node.named_child(skipped).unwrap();

    assert_eq!(identifier.kind(), "identifier");
    let argument_name = get_text(identifier, code);

    NamedArgument {
        name: argument_name,
        argument_type,
        ..Default::default()
    }
}

//  - Function type ???
//  - _typeName, arguments, nullable
//  - Record

fn parse_type(node: tree_sitter::Node, code: &str) -> (DartType, usize) {
    let mut processed = 0;
    let mut name: String = String::new();
    let mut type_arguments = Vec::new();
    let mut current_node = Some(node);
    let mut nullable = false;

    if node.kind() == "record_type" {
        todo!();
    }

    if let Some(node) = current_node
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
