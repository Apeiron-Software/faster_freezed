use std::fmt::Write;

use crate::dart_types::{DartType, ParameterList};

pub fn generate_mixin_copywith_function(output: &mut String, class_name: &str) {
    let _ = writeln!(
        output,
        "  @pragma('vm:prefer-inline')
  ${class_name}CopyWith<{class_name}> get copyWith =>
    _${class_name}CopyWithImpl<{class_name}>(this as {class_name}, _$identity);"
    );
}

pub fn generate_abstract_copywith_mixin(
    output: &mut String,
    class_name: &str,
    implements: Option<&str>,
    fields: &ParameterList,
) {
    if fields.get_all_params().is_empty() {
        let _ = writeln!(
            output,
            "class ${class_name}CopyWith<$Res> {}{{",
            if let Some(class) = implements {
                format!("implements ${class}CopyWith<$Res> ")
            } else {
                "".to_string()
            }
        );

        let _ = writeln!(
            output,
            "  ${class_name}CopyWith({class_name} value, $Res Function({class_name}) _then);
  }}"
        );
        return;
    }

    let _ = writeln!(
        output,
        "abstract mixin class ${class_name}CopyWith<$Res> {}{{",
        if let Some(class) = implements {
            format!("implements ${class}CopyWith<$Res> ")
        } else {
            "".to_string()
        }
    );

    let _ = writeln!(
        output,
        "  factory ${class_name}CopyWith({class_name} value, $Res Function({class_name}) _then) =
      _${class_name}CopyWithImpl;
  $Res call({{"
    );

    for field in fields.get_all_params() {
        let _ = writeln!(output, "    {} {},", field.dart_type.as_raw(), field.name);
    }

    let _ = writeln!(output, "// SOSAAAAAAT");

    let _ = writeln!(output, "  }});");
    let _ = writeln!(output, "}}");
}

pub fn generate_copywith_impl_mixin(
    output: &mut String,
    class_name: &str,
    fields: &ParameterList,
    has_constructor: bool,
) {
    let _ = writeln!(
        output,
        "class _${class_name}CopyWithImpl<$Res> implements ${class_name}CopyWith<$Res> {{
  _${class_name}CopyWithImpl(this._self, this._then);
  
  final {class_name} _self;
  final $Res Function({class_name}) _then;
"
    );
    generate_impl_function(output, class_name, fields, has_constructor);

    let _ = writeln!(output, "}}");
}

pub fn generate_impl_function(
    output: &mut String,
    class_name: &str,
    fields: &ParameterList,
    has_constructor: bool,
) {
    let _ = writeln!(
        output,
        "  @override
  @pragma('vm:prefer-inline')
  $Res call({{"
    );
    for field in fields.get_all_params() {
        if field.dart_type.nullable || field.dart_type.name.is_empty() {
            let _ = writeln!(output, "    Object? {} = freezed,", field.name);
        } else {
            let _ = writeln!(output, "    Object? {} = null,", field.name);
        }
    }

    if has_constructor {
        let _ = writeln!(
            output,
            "  }}) {{
  return _then({class_name}("
        );

        for pos_field in &fields.positional_parameters {
            generate_copywith_element(output, &pos_field.name, &pos_field.dart_type);
            let _ = write!(output, ",");
        }

        if !fields.named_parameters.is_empty() {
            for field in &fields.named_parameters {
                let _ = write!(output, "{}: ", field.name);
                generate_copywith_element(output, &field.name, &field.dart_type);
                let _ = write!(output, ",");
            }
        }
    } else {
        let _ = writeln!(
            output,
            "  }}) {{
  return _then(_self.copyWith("
        );

        let all_params = fields.get_all_params();
        if !all_params.is_empty() {
            for field in all_params {
                let _ = write!(output, "{}: ", field.name);
                generate_copywith_element(output, &field.name, &field.dart_type);
                let _ = write!(output, ",");
            }
        }
    }

    let _ = writeln!(output, "));");
    let _ = writeln!(output, "  }}");
}

pub fn generate_copywith_element(output: &mut String, name: &str, dart_type: &DartType) {
    if dart_type.nullable || dart_type.name.is_empty() {
        let _ = writeln!(
            output,
            "freezed == {name} ? _self.{name} : {name} as {}",
            dart_type.as_raw()
        );
    } else {
        let _ = writeln!(
            output,
            "null == {name} ? _self.{name} : {name} as {}",
            dart_type.as_raw()
        );
    }
}
