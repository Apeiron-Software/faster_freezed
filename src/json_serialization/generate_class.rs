use crate::dart_types::{
    ClassDefinition, DartType, ParameterList, PositionalParameter, get_generic_string,
};
use std::fmt::Write;

use super::{
    JsonMethod, from_json_function_generator, generate_abstract_copywith_mixin,
    generate_copywith_impl_mixin, generate_mixin, generate_solo_class, to_json_function_generator,
};

pub fn intersection_positional_parameters(
    vec1: &mut Vec<PositionalParameter>,
    vec2: &[PositionalParameter],
) {
    let mut i = 0;

    while i < vec1.len() {
        let item = &vec1[i];
        let iniside = vec2
            .iter()
            .any(|e| e.name == item.name && e.dart_type == item.dart_type);

        if !iniside {
            vec1.remove(i);
        } else {
            i += 1;
        }
    }
}

pub fn generate_class(output: &mut String, json_output: &mut String, class: &ClassDefinition) {
    let mixin_type = class.mixins.first().unwrap().as_raw();

    let class_generics = class.mixins.first().unwrap().type_arguments.to_owned();
    let mut copywith_generics = class_generics.clone();
    let class_name = class.name.clone();
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

    let just_generics = get_generic_string(&class_generics);
    let copywith_generics = get_generic_string(&copywith_generics);

    let mut intersecting_fields = class
        .redirecting_constructors
        .first()
        .unwrap()
        .parameters
        .get_all_params();

    for constructor in &class.redirecting_constructors {
        intersection_positional_parameters(
            &mut intersecting_fields,
            &constructor.parameters.get_all_params(),
        );
    }

    let mixin_virtual_parameters = ParameterList {
        positional_parameters: intersecting_fields.clone(),
        ..Default::default()
    };

    let class_to_json = if let Some(_json_constructor) = &class.json_constructor {
        JsonMethod::Signature
    } else {
        JsonMethod::None
    };

    generate_mixin(
        output,
        &mixin_type,
        &class.name,
        &class.mixins.first().unwrap().type_arguments,
        &intersecting_fields,
        &class_to_json,
    );

    generate_abstract_copywith_mixin(
        output,
        &class.name,
        &class_generics,
        None,
        &mixin_virtual_parameters,
    );
    if !intersecting_fields.is_empty() {
        generate_copywith_impl_mixin(
            output,
            &class.name,
            &class_generics,
            &mixin_virtual_parameters,
            false,
        );
    }

    for constructor in &class.redirecting_constructors {
        let inner_class = constructor.assigned_type.name.clone();

        let class_to_json = if let Some(_json_constructor) = &class.json_constructor {
            JsonMethod::Impl(class.name.clone()) // Change it to assigned json constructor? O_o
        } else {
            JsonMethod::None
        };

        generate_solo_class(
            output,
            &class.name,
            &class_generics,
            class_to_json,
            &inner_class,
            &constructor.parameters,
            constructor.is_const,
            &class.unnamed_constructor,
        );

        if !constructor.parameters.is_empty() {
            generate_abstract_copywith_mixin(
                output,
                &inner_class,
                &class_generics,
                Some(&class.name),
                &constructor.parameters,
            );

            generate_copywith_impl_mixin(
                output,
                &inner_class,
                &class_generics,
                &constructor.parameters,
                true,
            );
        }
    }

    if class.json_constructor.is_some() {
        let main_constructor = &class.redirecting_constructors.first().unwrap();
        let parameters = &main_constructor.parameters;

        to_json_function_generator(
            json_output,
            &class.name,
            &class
                .redirecting_constructors
                .first()
                .unwrap()
                .assigned_type
                .as_raw(),
            &parameters.get_all_params(),
        );

        from_json_function_generator(
            json_output,
            &main_constructor.assigned_type.as_raw(),
            &class.name,
            parameters,
        );
    }
}
