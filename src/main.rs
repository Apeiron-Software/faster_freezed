use faster_freezed::{parse_freezed_classes, generate_mixin};
use std::fs;

fn main() {
    let dart_code = fs::read_to_string("test_file.dart").expect("Failed to read test_file.dart");
    let mixin_code = generate_mixin(dart_code);
    println!("{}", mixin_code);
} 