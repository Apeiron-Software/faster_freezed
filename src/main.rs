use faster_freezed::{parse_freezed_classes, generate_mixin};

fn main() {
    let dart_code = r#"
@freezed
abstract class Test with _$Test {
  factory Test({
    required int? test,
    required String asdf,
    @Default('asdfasdf') String asdf2,
    @Default(Duration.zero) Duration dur,
  }) = _Test;
  const Test._();

}

"#;

    let mixin_code = generate_mixin(dart_code.to_string());
    println!("{}", mixin_code);
} 