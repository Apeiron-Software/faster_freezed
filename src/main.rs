use faster_freezed::{parse_freezed_classes, generate_mixin};

fn main() {
    let dart_code = r#"
@freezed
abstract class Test with _$Test {
  factory Test({
    required int x,
    @Default('hello') String a,
    @Default(42) int count,
  }) = _Test;
  Test._();
}
"#;

    let classes = parse_freezed_classes(dart_code.to_string());
    
    println!("=== FREEZED CLASSES SUMMARY ===");
    for class in &classes {
        println!("{:?}", class);
        println!("Class: {} (has_json: {})", class.name, class.has_json);
        println!("  Positional arguments: {}", class.positional_arguments.len());
        println!("  Named arguments: {}", class.named_arguments.len());
        println!("  Optional arguments: {}", class.optional_arguments.len());
    }

    println!("\n=== GENERATED MIXIN CODE ===");
    let mixin_code = generate_mixin(dart_code.to_string());
    println!("{}", mixin_code);

    println!("Finished");
} 