# Tests

This directory contains test files for the `faster_freezed` project.

## Files

- `test_file.dart` - Sample Dart code with @freezed classes used for testing the parser and code generator
- `integration_tests.rs` - Integration tests that test the library's public API with various Dart code examples

## Test Types

### Unit Tests
Unit tests are located in `src/lib.rs` and test individual functions and modules.

### Integration Tests
Integration tests are located in `tests/integration_tests.rs` and test the library's public API end-to-end.

### Sample Data
`test_file.dart` contains sample Dart code with various @freezed class configurations:
- Simple classes with positional parameters
- Classes with named parameters
- Classes with nullable types
- Classes with generic types (List, Map, Set)
- Classes with const constructors
- Classes with fromJson constructors
- Classes with annotations and default values

## Running Tests

```bash
# Run all tests (unit + integration)
cargo test

# Run only unit tests
cargo test --lib

# Run only integration tests
cargo test --test integration_tests

# Run tests with output
cargo test -- --nocapture
``` 