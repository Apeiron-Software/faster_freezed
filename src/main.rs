use faster_freezed::parse_freezed_classes;
use std::env;
use std::fs;
use std::path::Path;
use std::path::PathBuf;
use pathdiff;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        eprintln!("Usage: {} <directory>", args[0]);
        std::process::exit(1);
    }

    let directory = &args[1];
    if !Path::new(directory).exists() {
        eprintln!("Directory '{}' does not exist", directory);
        std::process::exit(1);
    }

    // No global generated dir creation here

    // Scan for Dart files and process them
    if let Err(e) = process_directory(directory) {
        eprintln!("Error processing directory: {}", e);
        std::process::exit(1);
    }
}

fn process_directory(dir_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let entries = fs::read_dir(dir_path)?;
    
    for entry in entries {
        let entry = entry?;
        let path = entry.path();
        
        if path.is_dir() {
            // Recursively process subdirectories
            if let Some(dir_name) = path.file_name() {
                if dir_name != "generated" { // Skip generated directory
                    process_directory(path.to_str().unwrap())?;
                }
            }
        } else if path.is_file() {
            // Check if it's a Dart file
            if let Some(extension) = path.extension() {
                if extension == "dart" {
                    process_dart_file(&path)?;
                }
            }
        }
    }
    
    Ok(())
}

fn process_dart_file(file_path: &Path) -> Result<(), Box<dyn std::error::Error>> {
    // Read file content
    let content = fs::read_to_string(file_path)?;
    
    // Check if file contains @freezed
    if !content.contains("@freezed") {
        return Ok(());
    }
    
    // Parse freezed classes
    let classes = parse_freezed_classes(content);
    if classes.is_empty() {
        return Ok(());
    }
    
    // Check if any class has JSON support
    let has_json = classes.iter().any(|class| class.has_json);
    if !has_json {
        return Ok(());
    }
    
    // Get filename without extension
    let file_stem = file_path.file_stem()
        .and_then(|s| s.to_str())
        .ok_or("Invalid filename")?;

    // Determine output directory: <parent>/generated
    let parent_dir = file_path.parent().ok_or("No parent directory")?;
    let generated_dir = parent_dir.join("generated");
    if !generated_dir.exists() {
        fs::create_dir(&generated_dir)?;
    }
    
    // Generate output files in the local generated dir
    generate_output_files(&generated_dir, file_stem, &classes, file_path)?;
    
    println!("Processed: {}", file_path.display());
    Ok(())
}

fn generate_output_files(
    generated_dir: &Path,
    file_stem: &str,
    classes: &[faster_freezed::freezed_class::FreezedClass],
    original_file_path: &Path,
) -> Result<(), Box<dyn std::error::Error>> {
    use std::path::PathBuf;
    // Compute relative path from generated_dir to original_file_path
    let generated_dir_abs = generated_dir.canonicalize()?;
    let original_file_abs = original_file_path.canonicalize()?;
    let relative_path = pathdiff::diff_paths(&original_file_abs, &generated_dir_abs)
        .unwrap_or_else(|| PathBuf::from(original_file_path.file_name().unwrap()));
    let relative_path_str = relative_path.to_string_lossy().replace('\\', "/");

    let part_of_line = format!("part of '{}';\n\n", relative_path_str);

    // Generate the full mixin code
    let mixin_code = generate_mixin_for_classes(classes);
    
    // Generate .freezed.dart file (contains everything)
    let freezed_path = generated_dir.join(format!("{}.freezed.dart", file_stem));
    let freezed_content = format!(
        "// GENERATED CODE - DO NOT MODIFY BY HAND\n\n{}{}",
        part_of_line,
        mixin_code
    );
    fs::write(&freezed_path, freezed_content)?;
    
    // Generate .g.dart file (contains only JSON part)
    let g_path = generated_dir.join(format!("{}.g.dart", file_stem));
    let g_content = format!(
        "{}// GENERATED CODE - DO NOT MODIFY BY HAND\n\n{}",
        part_of_line,
        generate_json_only_content(classes)
    );
    fs::write(&g_path, g_content)?;
    
    Ok(())
}

fn generate_mixin_for_classes(classes: &[faster_freezed::freezed_class::FreezedClass]) -> String {
    let mut output = String::new();
    
    for class in classes {
        if class.has_json {
            // Generate the mixin and class implementation
            let class_mixin = generate_single_class_mixin(class);
            output.push_str(&class_mixin);
        }
    }
    
    output
}

fn generate_single_class_mixin(class: &faster_freezed::freezed_class::FreezedClass) -> String {
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
        output.push_str("  Map<String, dynamic> toJson();\n");
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
    
    // Add toJson method implementation if has_json is true
    if class.has_json {
        output.push_str(&format!(
            "  @override\n  Map<String, dynamic> toJson() {{\n    return _${}ToJson(this);\n  }}\n",
            class.name
        ));
    }
    
    // Do NOT generate fromJson factory or toJson method implementation here
    // Do NOT generate top-level fromJson/toJson functions here
    
    // Close the class
    output.push_str("}\n\n");
    
    output
}

fn generate_json_only_content(classes: &[faster_freezed::freezed_class::FreezedClass]) -> String {
    let mut output = String::new();
    output.push_str("// GENERATED CODE - DO NOT MODIFY BY HAND\n\n");
    
    for class in classes {
        if class.has_json {
            let mut all_fields = Vec::new();
            all_fields.extend(&class.positional_arguments);
            all_fields.extend(&class.named_arguments);
            
            output.push_str(&generate_from_json_function(&class.name, &all_fields[..]));
            output.push_str("\n");
            output.push_str(&generate_to_json_function(&class.name, &all_fields[..]));
            output.push_str("\n");
        }
    }
    
    output
}

// Helper functions (copied from lib.rs)
use faster_freezed::freezed_class::Argument;

fn generate_getters(all_fields: &[&Argument]) -> String {
    let getters: Vec<String> = all_fields
        .iter()
        .map(|arg| format!("  {} get {};", arg.r#type.replace("required ", "").trim(), arg.name))
        .collect();
    getters.join("\n")
}

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

fn generate_constructor(class: &faster_freezed::freezed_class::FreezedClass) -> String {
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

fn generate_field_declarations(all_fields: &[&Argument]) -> String {
    let mut output = String::new();
    for field in all_fields {
        output.push_str(&format!("  @override\n  final {} {};\n", field.r#type, field.name));
    }
    output
}

fn generate_copy_with_implementation(class_name: &str, all_fields: &[&Argument]) -> String {
    let mut output = String::new();
    output.push_str(&format!("  @override\n  {} copyWith({{", class_name));
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

fn generate_from_json_function(class_name: &str, all_fields: &[&Argument]) -> String {
    use faster_freezed::json_serialization::generate_field_from_json;
    
    let mut output = String::new();
    output.push_str(&format!("_{} _${}FromJson(Map<String, dynamic> json) => _{}(", 
        class_name, class_name, class_name));
    
    let from_json_args: Vec<String> = all_fields
        .iter()
        .map(|arg| {
            let field_name = &arg.name;
            let field_type = &arg.r#type;
            generate_field_from_json(field_name, field_type, arg)
        })
        .collect();
    
    output.push_str(&from_json_args.join(",\n  "));
    output.push_str(");\n");
    output
}

fn generate_to_json_function(class_name: &str, all_fields: &[&Argument]) -> String {
    use faster_freezed::json_serialization::generate_field_to_json;
    
    let mut output = String::new();
    output.push_str(&format!("Map<String, dynamic> _${}ToJson(_{} instance) => <String, dynamic>{{", 
        class_name, class_name));
    
    let to_json_fields: Vec<String> = all_fields
        .iter()
        .map(|arg| {
            let field_name = &arg.name;
            generate_field_to_json(field_name, arg)
        })
        .collect();
    
    output.push_str(&format!("\n{}", to_json_fields.join(",\n")));
    output.push_str(",\n};\n");
    output
} 