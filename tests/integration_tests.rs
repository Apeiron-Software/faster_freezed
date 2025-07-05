use faster_freezed::{freezed_class::FreezedClass, generate_mixin, parse_freezed_classes, parser};

#[test]
fn test_pasing_types() {
    let code = r#"
@freezed
abstract class Test with _$Test {
  factory Test({
    required List<Map<ASDF, ASDF>> test,
  }) = _Test;
  Test._();
}
    "#;
}
