use faster_freezed::{parse_freezed_classes, generate_mixin, freezed_class::FreezedClass, parser};

#[test]
fn test_parse_and_generate_simple_class() {
    let code = r#"
@freezed
class SimpleClass with _$SimpleClass {
  const factory SimpleClass(
    String name,
    int age,
  ) = _SimpleClass;
}
"#;
    
    let classes = parse_freezed_classes(code.to_string());
    assert_eq!(classes.len(), 1);
    
    let class = &classes[0];
    assert_eq!(class.name, "SimpleClass");
    assert_eq!(class.positional_arguments.len(), 2);
    assert_eq!(class.named_arguments.len(), 0);
    
    let mixin_code = generate_mixin(code.to_string());
    assert!(mixin_code.contains("mixin _$SimpleClass"));
    assert!(mixin_code.contains("String get name"));
    assert!(mixin_code.contains("int get age"));
}

#[test]
fn test_parse_and_generate_with_nullable_types() {
    let code = r#"
@freezed
class NullableTest with _$NullableTest {
  const factory NullableTest({
    required String? nullableString,
    required int? nullableInt,
    required List<String>? nullableList,
  }) = _NullableTest;
}
"#;
    
    let classes = parse_freezed_classes(code.to_string());
    assert_eq!(classes.len(), 1);
    
    let class = &classes[0];
    assert_eq!(class.name, "NullableTest");
    assert_eq!(class.named_arguments.len(), 3);
    
    // Check that nullable types are preserved
    let string_arg = &class.named_arguments[0];
    assert_eq!(string_arg.r#type, "String?");
    
    let int_arg = &class.named_arguments[1];
    assert_eq!(int_arg.r#type, "int?");
    
    let list_arg = &class.named_arguments[2];
    assert_eq!(list_arg.r#type, "List<String>?");
    
    let mixin_code = generate_mixin(code.to_string());
    assert!(mixin_code.contains("String? get nullableString"));
    assert!(mixin_code.contains("int? get nullableInt"));
    assert!(mixin_code.contains("List<String>? get nullableList"));
}

#[test]
fn test_parse_and_generate_with_generic_types() {
    let code = r#"
@freezed
class GenericTest with _$GenericTest {
  const factory GenericTest({
    required List<int> nonNullableList,
    List<String>? nullableList,
    required Map<String, List<int>> nonNullableMap,
    Map<int, String>? nullableMap,
  }) = _GenericTest;
}
"#;
    
    let classes = parse_freezed_classes(code.to_string());
    assert_eq!(classes.len(), 1);
    
    let class = &classes[0];
    assert_eq!(class.name, "GenericTest");
    assert_eq!(class.named_arguments.len(), 4);
    
    // Check that generic types are preserved correctly
    let non_nullable_list = &class.named_arguments[0];
    assert_eq!(non_nullable_list.r#type, "List<int>");
    
    let nullable_list = &class.named_arguments[1];
    assert_eq!(nullable_list.r#type, "List<String>?");
    
    let non_nullable_map = &class.named_arguments[2];
    assert_eq!(non_nullable_map.r#type, "Map<String, List<int>>");
    
    let nullable_map = &class.named_arguments[3];
    assert_eq!(nullable_map.r#type, "Map<int, String>?");
    
    let mixin_code = generate_mixin(code.to_string());
    assert!(mixin_code.contains("List<int> get nonNullableList"));
    assert!(mixin_code.contains("List<String>? get nullableList"));
    assert!(mixin_code.contains("Map<String, List<int>> get nonNullableMap"));
    assert!(mixin_code.contains("Map<int, String>? get nullableMap"));
    
    // Check that copyWith method preserves nullability in type casts
    assert!(mixin_code.contains("nullableList: freezed == nullableList ? this.nullableList : nullableList as List<String>?"));
    assert!(mixin_code.contains("nullableMap: freezed == nullableMap ? this.nullableMap : nullableMap as Map<int, String>?"));
}

#[test]
fn test_parse_and_generate_with_const_constructor() {
    let code = r#"
@freezed
class ConstTest with _$ConstTest {
  const factory ConstTest({
    required String name,
    int age,
  }) = _ConstTest;
  
  const ConstTest._();
}
"#;
    
    let classes = parse_freezed_classes(code.to_string());
    assert_eq!(classes.len(), 1);
    
    let class = &classes[0];
    assert_eq!(class.name, "ConstTest");
    assert_eq!(class.has_const_constructor, true);
    
    let mixin_code = generate_mixin(code.to_string());
    // println!("Generated code:\n{}", mixin_code);
    assert!(mixin_code.contains("const  _ConstTest"));
}

#[test]
fn test_parse_and_generate_with_fromjson() {
    let code = r#"
@freezed
class JsonTest with _$JsonTest {
  const factory JsonTest({
    required String name,
    int age,
  }) = _JsonTest;

  factory JsonTest.fromJson(Map<String, Object?> json) => _$JsonTestFromJson(json);
}
"#;
    
    let classes = parse_freezed_classes(code.to_string());
    assert_eq!(classes.len(), 1);
    
    let class = &classes[0];
    assert_eq!(class.name, "JsonTest");
    assert_eq!(class.has_json, true);
    
    let mixin_code = generate_mixin(code.to_string());
    assert!(mixin_code.contains("mixin _$JsonTest"));
    assert!(mixin_code.contains("String get name"));
    assert!(mixin_code.contains("int get age"));
}

#[test]
fn test_parse_and_generate_multiple_classes() {
    let code = r#"
@freezed
class Person with _$Person {
  const factory Person({
    required String firstName,
    required String lastName,
    int? age,
  }) = _Person;
}

@freezed
class Alien with _$Alien {
  const factory Alien({
    required String firstName,
    required String lastName,
    required int age,
  }) = _Alien;
}
"#;
    
    let classes = parse_freezed_classes(code.to_string());
    assert_eq!(classes.len(), 2);
    
    let person = &classes[0];
    assert_eq!(person.name, "Person");
    assert_eq!(person.named_arguments.len(), 3);
    
    let alien = &classes[1];
    assert_eq!(alien.name, "Alien");
    assert_eq!(alien.named_arguments.len(), 3);
    
    let mixin_code = generate_mixin(code.to_string());
    assert!(mixin_code.contains("mixin _$Person"));
    assert!(mixin_code.contains("mixin _$Alien"));
    assert!(mixin_code.contains("class _Person"));
    assert!(mixin_code.contains("class _Alien"));
} 


  fn parse_dart_code(code: &str) -> Vec<FreezedClass> {
      parser::parse_dart_code(code)
  }

  #[test]
  fn test_parse_simple_class() {
      let code = r#"
@freezed
class SimpleClass with _$SimpleClass {
const factory SimpleClass(
  String name,
  int age,
) = _SimpleClass;
}
"#;
      
      let classes = parse_dart_code(code);
      assert_eq!(classes.len(), 1);
      
      let class = &classes[0];
      assert_eq!(class.name, "SimpleClass");
      assert_eq!(class.positional_arguments.len(), 2);
      assert_eq!(class.named_arguments.len(), 0);
      assert_eq!(class.has_json, false);
      assert_eq!(class.has_const_constructor, false);
      
      let name_arg = &class.positional_arguments[0];
      assert_eq!(name_arg.name, "name");
      assert_eq!(name_arg.r#type, "String");
      assert_eq!(name_arg.is_required, false);
      assert_eq!(name_arg.default_value, None);
      assert_eq!(name_arg.annotations.len(), 0);
      
      let age_arg = &class.positional_arguments[1];
      assert_eq!(age_arg.name, "age");
      assert_eq!(age_arg.r#type, "int");
      assert_eq!(age_arg.is_required, false);
      assert_eq!(age_arg.default_value, None);
      assert_eq!(age_arg.annotations.len(), 0);
  }

  #[test]
  fn test_parse_class_with_named_parameters() {
      let code = r#"
@freezed
class NamedClass with _$NamedClass {
const factory NamedClass({
  required String firstName,
  required String lastName,
  int? age,
}) = _NamedClass;
}
"#;
      
      let classes = parse_dart_code(code);
      assert_eq!(classes.len(), 1);
      
      let class = &classes[0];
      assert_eq!(class.name, "NamedClass");
      assert_eq!(class.positional_arguments.len(), 0);
      assert_eq!(class.named_arguments.len(), 3);
      assert_eq!(class.has_json, false);
      assert_eq!(class.has_const_constructor, false);
      
      let first_arg = &class.named_arguments[0];
      assert_eq!(first_arg.name, "firstName");
      assert_eq!(first_arg.r#type, "String");
      assert_eq!(first_arg.is_required, true);
      
      let last_arg = &class.named_arguments[1];
      assert_eq!(last_arg.name, "lastName");
      assert_eq!(last_arg.r#type, "String");
      assert_eq!(last_arg.is_required, true);
      
      let age_arg = &class.named_arguments[2];
      assert_eq!(age_arg.name, "age");
      assert_eq!(age_arg.r#type, "int?");
      assert_eq!(age_arg.is_required, false);
  }

  #[test]
  fn test_parse_class_with_annotations_and_defaults() {
      let code = r#"
@freezed
class AnnotatedClass with _$AnnotatedClass {
const factory AnnotatedClass(
  @Default("John") String name,
  @JsonKey(name: 'user_age') required int age,
) = _AnnotatedClass;
}
"#;
      
      let classes = parse_dart_code(code);
      assert_eq!(classes.len(), 1);
      
      let class = &classes[0];
      assert_eq!(class.name, "AnnotatedClass");
      assert_eq!(class.positional_arguments.len(), 2);
      assert_eq!(class.has_const_constructor, false);
      
      let name_arg = &class.positional_arguments[0];
      assert_eq!(name_arg.name, "name");
      assert_eq!(name_arg.r#type, "String");
      assert_eq!(name_arg.default_value, Some("John".to_string()));
      assert_eq!(name_arg.annotations.len(), 1);
      assert!(name_arg.annotations[0].contains("@Default"));
      
      let age_arg = &class.positional_arguments[1];
      assert_eq!(age_arg.name, "age");
      assert_eq!(age_arg.r#type, "int");
      assert_eq!(age_arg.is_required, true);
      assert_eq!(age_arg.annotations.len(), 1);
      assert!(age_arg.annotations[0].contains("@JsonKey"));
  }

  #[test]
  fn test_parse_class_with_fromjson() {
      let code = r#"
@freezed
class JsonClass with _$JsonClass {
const factory JsonClass({
  required String name,
  int age,
}) = _JsonClass;

factory JsonClass.fromJson(Map<String, Object?> json) => _$JsonClassFromJson(json);
}
"#;
      
      let classes = parse_dart_code(code);
      assert_eq!(classes.len(), 1);
      
      let class = &classes[0];
      assert_eq!(class.name, "JsonClass");
      assert_eq!(class.has_json, true);
      assert_eq!(class.named_arguments.len(), 2);
      assert_eq!(class.has_const_constructor, false);
  }

  #[test]
  fn test_parse_multiple_classes() {
      let code = r#"
@freezed
class Person with _$Person {
const factory Person(
  int hello, {
  @Default("hi") String firstName,
  @RandomAnnotation() required String lastName,
  required int age,
}) = _Person;

factory Person.fromJson(Map<String, Object?> json) => _$PersonFromJson(json);
}

@freezed
class Alien with _$Alien {
const factory Alien({
  required String firstName,
  required String lastName,
  required int age,
}) = _Alien;

factory Alien.fromJson(Map<String, Object?> json) => _$AlienFromJson(json);
}
"#;
      
      let classes = parse_dart_code(code);
      assert_eq!(classes.len(), 2);
      
      let person = &classes[0];
      assert_eq!(person.name, "Person");
      assert_eq!(person.has_json, true);
      assert_eq!(person.positional_arguments.len(), 1);
      assert_eq!(person.named_arguments.len(), 3);
      assert_eq!(person.has_const_constructor, false);
      
      let alien = &classes[1];
      assert_eq!(alien.name, "Alien");
      assert_eq!(alien.has_json, true);
      assert_eq!(alien.positional_arguments.len(), 0);
      assert_eq!(alien.named_arguments.len(), 3);
      assert_eq!(alien.has_const_constructor, false);
  }

  #[test]
  fn test_parse_parameter_function() {
      // Test the parse_parameter function directly
      let code = r#"
@freezed
class TestClass with _$TestClass {
const factory TestClass(
  @Default("test") String name,
) = _TestClass;
}
"#;
      
      let classes = parse_dart_code(code);
      assert_eq!(classes.len(), 1);
      
      let class = &classes[0];
      assert_eq!(class.name, "TestClass");
      assert_eq!(class.positional_arguments.len(), 1);
      assert_eq!(class.has_const_constructor, false);
      
      let name_arg = &class.positional_arguments[0];
      assert_eq!(name_arg.name, "name");
      assert_eq!(name_arg.r#type, "String");
      assert_eq!(name_arg.default_value, Some("test".to_string()));
      assert_eq!(name_arg.annotations.len(), 1);
      assert!(name_arg.annotations[0].contains("@Default"));
  }

  #[test]
  fn test_parse_class_with_const_constructor() {
      let code = r#"
@freezed
class ConstClass with _$ConstClass {
const factory ConstClass({
  required String name,
  int age,
}) = _ConstClass;

const ConstClass._();
}
"#;
      
      let classes = parse_dart_code(code);
      assert_eq!(classes.len(), 1);
      
      let class = &classes[0];
      assert_eq!(class.name, "ConstClass");
      assert_eq!(class.has_json, false);
      assert_eq!(class.named_arguments.len(), 2);
      assert_eq!(class.has_const_constructor, true);
      
      let name_arg = &class.named_arguments[0];
      assert_eq!(name_arg.name, "name");
      assert_eq!(name_arg.r#type, "String");
      assert_eq!(name_arg.is_required, true);
      
      let age_arg = &class.named_arguments[1];
      assert_eq!(age_arg.name, "age");
      assert_eq!(age_arg.r#type, "int");
      assert_eq!(age_arg.is_required, false);
  }

  #[test]
  fn test_generate_mixin() {
      let code = r#"
@freezed
abstract class Test with _$Test {
factory Test({required int i, @Default('hello') String data}) = _Test;
Test._();
factory Test.fromJson(Map<String, dynamic> json) => _$TestFromJson(json);
}
"#;
      
      let mixin_code = generate_mixin(code.to_string());
      println!("{}", mixin_code);
      // Check that the mixin contains the expected elements
      assert!(mixin_code.contains("mixin _$Test {"));
      assert!(mixin_code.contains("  int get i;"));
      assert!(mixin_code.contains("  String get data;"));
      assert!(mixin_code.contains("  bool operator =="));
      assert!(mixin_code.contains("other is Test"));
      assert!(mixin_code.contains("&& (identical(other.i, i) || other.i == i)"));
      assert!(mixin_code.contains("&& (identical(other.data, data) || other.data == data)"));
      assert!(mixin_code.contains("int get hashCode => Object.hash(runtimeType, i, data);"));
      assert!(mixin_code.contains("String toString()"));
      assert!(mixin_code.contains("return 'Test(i: $i, data: $data)';"));
      assert!(mixin_code.contains("Test copyWith({int? i, String? data});"));
      assert!(mixin_code.contains("}"));
      
      // Check that the class implementation contains the expected elements
      assert!(mixin_code.contains("class _Test extends Test {"));
      assert!(mixin_code.contains("_Test ({required this.i, this.data = 'hello'}) : super._();"));
      assert!(mixin_code.contains("@override\n  final int i;"));
      assert!(mixin_code.contains("@override\n  final String data;"));
  }

  #[test]
  fn test_generate_mixin_with_const_constructor() {
      let code = r#"
@freezed
abstract class ConstTest with _$ConstTest {
factory ConstTest({required int i, @Default('hello') String data}) = _ConstTest;
const ConstTest._();
factory ConstTest.fromJson(Map<String, dynamic> json) => _$ConstTestFromJson(json);
}
"#;
      
      let mixin_code = generate_mixin(code.to_string());
      
      // Check that the class implementation contains const keyword
      assert!(mixin_code.contains("class _ConstTest extends ConstTest {"));
      assert!(mixin_code.contains("_ConstTest ({required this.i, this.data = 'hello'}) : super._();"));
      assert!(mixin_code.contains("@override\n  final int i;"));
      assert!(mixin_code.contains("@override\n  final String data;"));
  }

  #[test]
  fn test_generate_mixin_with_copywith() {
      let code = r#"
@freezed
abstract class CopyWithTest with _$CopyWithTest {
factory CopyWithTest({required int i, String? data}) = _CopyWithTest;
CopyWithTest._();
}
"#;
      
      let mixin_code = generate_mixin(code.to_string());
      
      // Check that the copyWith method is generated
      assert!(mixin_code.contains("CopyWithTest copyWith({int? i, Object? data = freezed});"));
      assert!(mixin_code.contains("return _CopyWithTest("));
      assert!(mixin_code.contains("i: i == null ? this.i : i as int"));
      assert!(mixin_code.contains("data: freezed == data ? this.data : data as String?"));
      
      // Check that the mixin contains the copyWith declaration
      assert!(mixin_code.contains("mixin _$CopyWithTest {"));
      assert!(mixin_code.contains("CopyWithTest copyWith({int? i, Object? data = freezed});"));
  }

  #[test]
  fn test_generate_mixin_with_collections() {
      let code = r#"
@freezed
abstract class CollectionTest with _$CollectionTest {
factory CollectionTest({
  required List<String> items,
  required Map<String, int> scores,
  required Set<int> numbers,
  required String name,
}) = _CollectionTest;
CollectionTest._();
}
"#;
      
      let mixin_code = generate_mixin(code.to_string());
      
      // Check that deep collection equality is used for collections
      assert!(mixin_code.contains("const DeepCollectionEquality().equals(other.items, items)"));
      assert!(mixin_code.contains("const DeepCollectionEquality().equals(other.scores, scores)"));
      assert!(mixin_code.contains("const DeepCollectionEquality().equals(other.numbers, numbers)"));
      
      // Check that regular equality is used for non-collections
      assert!(mixin_code.contains("(identical(other.name, name) || other.name == name)"));
      
      // Check that the mixin contains the correct copyWith signature
      assert!(mixin_code.contains("CollectionTest copyWith({List<String>? items, Map<String, int>? scores, Set<int>? numbers, String? name});"));
  }

  #[test]
  fn test_generate_mixin_with_nullable_types() {
      let code = r#"
@freezed
abstract class NullableTest with _$NullableTest {
factory NullableTest({
  required String? nullableString,
  required int? nullableInt,
  required List<String>? nullableList,
}) = _NullableTest;
NullableTest._();
}
"#;
      
      let mixin_code = generate_mixin(code.to_string());
      
      // Check that the copyWith method preserves nullability in type casts
      assert!(mixin_code.contains("nullableString: freezed == nullableString ? this.nullableString : nullableString as String?"));
      assert!(mixin_code.contains("nullableInt: freezed == nullableInt ? this.nullableInt : nullableInt as int?"));
      assert!(mixin_code.contains("nullableList: freezed == nullableList ? this.nullableList : nullableList as List<String>?"));
      
      // Check that the mixin contains the correct copyWith signature with nullable types
      assert!(mixin_code.contains("NullableTest copyWith({Object? nullableString = freezed, Object? nullableInt = freezed, Object? nullableList = freezed});"));
  }

  #[test]
  fn test_generate_mixin_with_generics() {
      let code = r#"
@freezed
abstract class GenericsTest with _$GenericsTest {
factory GenericsTest({
  required List<int> ints,
  required Map<String, List<int>> complex,
}) = _GenericsTest;
GenericsTest._();
}
"#;
      let mixin_code = generate_mixin(code.to_string());
      assert!(mixin_code.contains("final List<int> ints;"));
      assert!(mixin_code.contains("final Map<String, List<int>> complex;"));
      assert!(mixin_code.contains("GenericsTest copyWith({List<int>? ints, Map<String, List<int>>? complex});"));
      assert!(mixin_code.contains("ints: ints == null ? this.ints : ints as List<int>"));
      assert!(mixin_code.contains("complex: complex == null ? this.complex : complex as Map<String, List<int>>"));
  }

  #[test]
  fn test_generate_mixin_with_json_converter() {
      let code = r#"
@freezed
abstract class JsonConverterTest with _$JsonConverterTest {
factory JsonConverterTest({
  required int id,
  @DurationJsonConverter() required Duration duration,
  @CustomJsonConverter() String? nullableField,
  @ListJsonConverter() required List<int> numbers,
}) = _JsonConverterTest;
JsonConverterTest._();
factory JsonConverterTest.fromJson(Map<String, dynamic> json) => _$JsonConverterTestFromJson(json);
}
"#;
      
      let mixin_code = generate_mixin(code.to_string());
      println!("{}", mixin_code);
      // Check that the mixin contains the expected elements
      assert!(mixin_code.contains("mixin _$JsonConverterTest {"));
      assert!(mixin_code.contains("  int get id;"));
      assert!(mixin_code.contains("  Duration get duration;"));
      assert!(mixin_code.contains("  String? get nullableField;"));
      assert!(mixin_code.contains("  List<int> get numbers;"));
      assert!(mixin_code.contains("  Map<String, dynamic> toJson();"));
      
      // Check that the class implementation contains the expected elements
      assert!(mixin_code.contains("class _JsonConverterTest extends JsonConverterTest {"));
      assert!(mixin_code.contains("  factory _JsonConverterTest.fromJson(Map<String, dynamic> json) => _$JsonConverterTestFromJson(json);"));
      assert!(mixin_code.contains("  @override\n  Map<String, dynamic> toJson() {\n    return _$JsonConverterTestToJson(this);\n  }"));
      
      // Check that the fromJson function uses JsonConverter for decorated fields
      assert!(mixin_code.contains("id: (json['id'] as num).toInt()"));
      assert!(mixin_code.contains("duration: const DurationJsonConverter().fromJson((json['duration'] as num).toInt())"));
      assert!(mixin_code.contains("nullableField: json['nullableField'] == null ? null : const CustomJsonConverter().fromJson((json['nullableField'] as num).toInt())"));
      assert!(mixin_code.contains("numbers: const ListJsonConverter().fromJson((json['numbers'] as num).toInt())"));
      
      // Check that the toJson function uses JsonConverter for decorated fields
      assert!(mixin_code.contains("'id': instance.id"));
      assert!(mixin_code.contains("'duration': const DurationJsonConverter().toJson(instance.duration)"));
      assert!(mixin_code.contains("'nullableField': const CustomJsonConverter().toJson(instance.nullableField)"));
      assert!(mixin_code.contains("'numbers': const ListJsonConverter().toJson(instance.numbers)"));
  }
