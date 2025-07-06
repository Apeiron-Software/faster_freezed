use faster_freezed::json_serialization::generate_class;
use faster_freezed::parser::parse_dart_code;
use std::fs;

fn my_little_test() {
    let content = fs::read_to_string("test.dart").unwrap();
    //print_ts_tree_for_code(&content);
    let x = &parse_dart_code(&content)[0];
    dbg!(x);

    let mut output = String::new();

    generate_class(&mut output, x);

    // let constructor = x
    //     .redirecting_constructors
    //     .iter()
    //     .find(|e| e.class_name == x.name)
    //     .unwrap();
    // let mixin_name = &x.mixins.first().unwrap().as_raw();
    // generate_mixin(
    //     &mut output,
    //     mixin_name,
    //     &x.name,
    //     &constructor.parameters.get_all_params(),
    // );
    // let _ = writeln!(output);
    // generate_abstract_copywith_mixin(
    //     &mut output,
    //     //&x.mixins[0].as_raw(),
    //     //&x.name,
    //     &x.name,
    //     &constructor.parameters.get_all_params(),
    // );
    // let _ = writeln!(output);
    // generate_impl_mixin(
    //     &mut output,
    //     //&x.mixins[0].as_raw(),
    //     //&x.name,
    //     &x.name,
    //     &constructor.parameters,
    // );
    //
    // let _ = writeln!(output);
    // generate_solo_class(
    //     &mut output,
    //     &x.name,
    //     &format!("_{}", &x.name),
    //     &constructor.parameters,
    // );
    println!("{output}");
}

fn main() {
    my_little_test();
}
