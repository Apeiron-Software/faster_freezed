fn main() {
    let mut build = cc::Build::new();
    build.file("tree-sitter-dart/parser.c");
    // Only include scanner.c if it exists
    if std::path::Path::new("tree-sitter-dart/scanner.c").exists() {
        build.file("tree-sitter-dart/scanner.c");
    }
    build.flag("-w"); // Suppress all warnings
    build.compile("tree-sitter-dart");
} 