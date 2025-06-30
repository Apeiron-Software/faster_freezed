@freezed
abstract class Test with _$Test {
  factory Test({
    required int? i,
    required String? id,
    required List<Another>? data,
    required Another? dataObject,
  }) = _Test;
  Test._();
  factory Test.fromJson(Map<String, dynamic> json) => _$TestFromJson(json);
}
