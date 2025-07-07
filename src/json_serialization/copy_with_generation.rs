use std::fmt::Write;

use crate::dart_types::{DartType, ParameterList, get_generic_string};

pub fn generate_mixin_copywith_function(
    output: &mut String,
    class_name: &str,
    copywith_use_generics: &str,
    class_generics: &str,
) {
    let _ = writeln!(
        output,
        "  @pragma('vm:prefer-inline')
  ${class_name}CopyWith{copywith_use_generics} get copyWith =>
    _${class_name}CopyWithImpl{copywith_use_generics}(this as {class_name}{class_generics}, _$identity);"
    );
}

pub fn generate_abstract_copywith_mixin(
    output: &mut String,
    class_name: &str,
    class_generics: &[DartType],
    implements: Option<&str>,
    fields: &ParameterList,
) {
    let mut copywith_generics = class_generics.to_owned();
    copywith_generics.push(DartType {
        name: "$Res".to_string(),
        ..Default::default()
    });

    let just_generics = get_generic_string(class_generics);
    let copywith_generics = get_generic_string(&copywith_generics);

    if fields.get_all_params().is_empty() {
        let _ = writeln!(
            output,
            "class ${class_name}CopyWith{copywith_generics} {}{{",
            if let Some(class) = implements {
                format!("implements ${class}CopyWith{copywith_generics} ")
            } else {
                "".to_string()
            }
        );

        let _ = writeln!(
            output,
            "  ${class_name}CopyWith({class_name}{just_generics} value, $Res Function({class_name}) _then);
  }}"
        );
        return;
    }

    let _ = writeln!(
        output,
        "abstract mixin class ${class_name}CopyWith{copywith_generics} {}{{",
        if let Some(class) = implements {
            format!("implements ${class}CopyWith{copywith_generics} ")
        } else {
            "".to_string()
        }
    );

    let _ = writeln!(
        output,
        "  factory ${class_name}CopyWith({class_name}{just_generics} value, $Res Function({class_name}) _then) =
      _${class_name}CopyWithImpl;
  $Res call({{"
    );

    for field in fields.get_all_params() {
        let _ = writeln!(output, "    {} {},", field.dart_type.as_raw(), field.name);
    }

    let _ = writeln!(output, "  }});");
    let _ = writeln!(output, "}}");
}

pub fn generate_copywith_impl_mixin(
    output: &mut String,
    class_name: &str,
    class_generics: &[DartType],
    fields: &ParameterList,
    has_constructor: bool,
) {
    let mut copywith_generics = class_generics.to_owned();
    copywith_generics.push(DartType {
        name: "$Res".to_string(),
        ..Default::default()
    });

    let just_generics = get_generic_string(class_generics);
    let copywith_generics = get_generic_string(&copywith_generics);

    let _ = writeln!(
        output,
        "class _${class_name}CopyWithImpl{copywith_generics} implements ${class_name}CopyWith{copywith_generics} {{
  _${class_name}CopyWithImpl(this._self, this._then);
  
  final {class_name}{just_generics} _self;
  final $Res Function({class_name}{just_generics}) _then;
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
