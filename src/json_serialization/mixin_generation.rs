use std::fmt::Write;

use crate::dart_types::{DartType, PositionalParameter, get_generic_string};

use super::{JsonMethod, generate_mixin_copywith_function, to_json_method_generator};

pub fn generate_mixin(
    output: &mut String,
    mixin_type: &str,
    class_name: &str,
    class_generics: &[DartType],
    fields: &[PositionalParameter],
    class_to_json: &JsonMethod,
) {
    let _ = writeln!(output, "/// @nodoc");
    let _ = writeln!(output, "mixin {mixin_type} {{");
    generate_mixin_getters(output, fields);
    let _ = writeln!(output);
    generate_eq_operator(output, mixin_type, fields);
    let _ = writeln!(output);
    generate_hash_operator(output, fields);
    let _ = writeln!(output);

    let mut copywith_generics = class_generics.to_owned();
    // That's a warcrime, check if can be done without nesting, just plain
    copywith_generics.push(DartType {
        name: DartType {
            name: class_name.to_string(),
            type_arguments: class_generics.to_owned(),
            nullable: false,
        }
        .as_raw(), // I could be worse, but like the fuck?
        ..Default::default()
    });

    let just_generics = get_generic_string(class_generics);
    let copywith_generics = get_generic_string(&copywith_generics);

    if !fields.is_empty() {
        generate_mixin_copywith_function(output, class_name, &copywith_generics, &just_generics);
        let _ = writeln!(output);
    }
    generate_to_string(output, class_name, fields, false);
    let _ = writeln!(output);

    match class_to_json {
        JsonMethod::None => {}
        JsonMethod::Signature => {
            to_json_method_generator(output, None);
            let _ = writeln!(output);
        }
        JsonMethod::Impl(name) => {
            to_json_method_generator(output, Some(name));
            let _ = writeln!(output);
        }
    }

    let _ = writeln!(output, "}}");
}

pub fn generate_mixin_getters(output: &mut String, fields: &[PositionalParameter]) {
    for field in fields {
        let _ = writeln!(output, "  {} get {};", field.dart_type.as_raw(), field.name);
    }
}

pub fn generate_introspection_class(
    output: &mut String,
    class_name: &str,
    fields: &[PositionalParameter],
) {
    let _ = writeln!(output, "class {class_name}Fields {{",);
    for field in fields {
        let _ = writeln!(
            output,
            "  static const {name} = ($get${class_name}${name}, $set${class_name}${name});",
            name = field.name
        );
    }
    let _ = writeln!(output, "static const $all = {{");
    for field in fields {
        let _ = writeln!(output, "  #{name}: {name},", name = field.name);
    }
    let _ = writeln!(output, "  }};");
    let _ = writeln!(output, "}}",);
    generate_mixin_getset_functions(class_name, output, fields);
}

pub fn generate_mixin_getset_functions(
    class_name: &str,
    output: &mut String,
    fields: &[PositionalParameter],
) {
    for field in fields {
        let _ = writeln!(
            output,
            "{field_type} $get${class_name}${name}({class_name} value) => value.{name};
  {class_name} $set${class_name}${name}({class_name} data, {field_type} value) => data.copyWith({name}: value);",
            field_type = field.dart_type.as_raw(),
            name = field.name
        );
    }
}

pub fn generate_eq_operator(output: &mut String, mixin_type: &str, fields: &[PositionalParameter]) {
    let _ = writeln!(
        output,
        r#"  @override
  bool operator ==(Object other) {{
    return identical(this, other) ||
      (other.runtimeType == runtimeType &&
         other is {mixin_type}"#
    );

    for field in fields {
        let _ = write!(output, "         && ");
        generate_comparator(output, &field.name, &field.dart_type);
        let _ = writeln!(output);
    }

    let _ = writeln!(
        output,
        r#"       );
  }}"#
    );
}

pub fn generate_comparator(output: &mut String, field_name: &str, dart_type: &DartType) {
    if dart_type.is_collection() {
        let _ = write!(
            output,
            "const DeepCollectionEquality().equals(other.{field_name}, {field_name})"
        );
    } else {
        // Simple cmp
        let _ = write!(
            output,
            "(identical(other.{field_name}, {field_name}) 
             || other.{field_name} == {field_name})"
        );
    }
}

pub fn generate_hash_operator(output: &mut String, fields: &[PositionalParameter]) {
    if fields.is_empty() {
        let _ = writeln!(
            output,
            r#"  @override
  int get hashCode => runtimeType.hashCode;"#
        );
        return;
    }

    if fields.len() > 19 {
        let _ = writeln!(
            output,
            r#"  @override
  int get hashCode => Object.hashAll([
    runtimeType,"#
        );
    } else {
        let _ = writeln!(
            output,
            r#"  @override
  int get hashCode => Object.hash(
    runtimeType,"#
        );
    }

    for field in fields {
        let _ = write!(output, "    ");
        generate_hash_line(output, &field.name, &field.dart_type);
        let _ = writeln!(output, ",");
    }

    if fields.len() > 19 {
        let _ = writeln!(output, r#"  ]);"#);
    } else {
        let _ = writeln!(output, r#"  );"#);
    }
}

pub fn generate_hash_line(output: &mut String, field_name: &str, dart_type: &DartType) {
    if dart_type.is_collection() {
        let _ = write!(output, "const DeepCollectionEquality().hash({field_name})");
    } else {
        // Simple cmp
        let _ = write!(output, "{field_name}");
    }
}

// TODO, toString for generics
pub fn generate_to_string(
    output: &mut String,
    class_name: &str,
    fields: &[PositionalParameter],
    override_f: bool,
) {
    if override_f {
        let _ = write!(output, "  @override");
    }
    let _ = write!(
        output,
        r#"  String toString() {{
      return '{class_name}("#
    );

    for field in fields {
        let _ = write!(output, "{0}: ${0}, ", field.name);
    }

    let _ = writeln!(
        output,
        r#")';
  }}"#
    );
}
