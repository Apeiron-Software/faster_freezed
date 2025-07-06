use std::fmt::Write;

use crate::dart_types::{DartType, PositionalParameter};

use super::{JsonMethod, generate_mixin_copywith_function, to_json_method_generator};

pub fn generate_mixin(
    output: &mut String,
    mixin_name: &str,
    class_name: &str,
    fields: &[PositionalParameter],
    class_to_json: &JsonMethod,
) {
    let _ = writeln!(output, "/// @nodoc");
    let _ = writeln!(output, "mixin {mixin_name} {{");
    generate_mixin_getters(output, fields);
    let _ = writeln!(output);
    generate_eq_operator(output, class_name, fields);
    let _ = writeln!(output);
    generate_hash_operator(output, fields);
    let _ = writeln!(output);
    generate_mixin_copywith_function(output, class_name);
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

pub fn generate_eq_operator(output: &mut String, class_name: &str, fields: &[PositionalParameter]) {
    let _ = writeln!(
        output,
        r#"  @override
  bool operator ==(Object other) {{
    return identical(this, other) ||
      (other.runtimeType == runtimeType &&
         other is {class_name}"#
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
    let _ = writeln!(
        output,
        r#"  @override
  int get hashCode => Object.hash(
    runtimeType,"#
    );

    for field in fields {
        let _ = write!(output, "    ");
        generate_hash_line(output, &field.name, &field.dart_type);
        let _ = writeln!(output, ",");
    }

    let _ = writeln!(output, r#"  );"#);
}

pub fn generate_hash_line(output: &mut String, field_name: &str, dart_type: &DartType) {
    if dart_type.is_collection() {
        let _ = write!(output, "const DeepCollectionEquality().hash({field_name})");
    } else {
        // Simple cmp
        let _ = write!(output, "{field_name}");
    }
}

pub fn generate_to_string(output: &mut String, class_name: &str, fields: &[PositionalParameter]) {
    let _ = write!(
        output,
        r#"  @override
  String toString() {{
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
