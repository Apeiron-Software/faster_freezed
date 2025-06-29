@freezed
class Person with _$Person {
  const factory Person(
    int hello, {
    @Default("hi") String firstName,
    @RandomAnnotation() required String lastName,
    required int age,
  }) = _Person;

  factory Person.fromJson(Map<String, Object?> json)
      => _$PersonFromJson(json);
}


@freezed
class Alien with _$Alien {
  const factory Alien({
    required String firstName,
    required String lastName,
    required int age,
  }) = _Alien;

  factory Alien.fromJson(Map<String, Object?> json)
      => _$AlienFromJson(json);
}

@freezed
class GenericTest with _$GenericTest {
  const factory GenericTest({
    required List<int> nonNullableList,
    List<String>? nullableList,
    required Map<String, List<int>> nonNullableMap,
    Map<int, String>? nullableMap,
  }) = _GenericTest;
}

@freezed
class SimpleNullable with _$SimpleNullable {
  const factory SimpleNullable({
    List<String>? nullableList,
  }) = _SimpleNullable;
}
