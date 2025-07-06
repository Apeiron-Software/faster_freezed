use std::env;
use std::fmt::write;
use std::fs;
use std::fs::File;
use std::fs::OpenOptions;
use std::fs::read_to_string;
use std::io::BufReader;
use std::io::ErrorKind;
use std::io::Read;
use std::path::Path;
use std::path::PathBuf;
use std::process::ExitCode;
use std::time::Instant;

use faster_freezed::json_serialization::generate_class;
use faster_freezed::parser::parse_dart_code;
use memmap2::Mmap;
use memmap2::MmapOptions;

// fn my_little_test() {
//     let content = fs::read_to_string("test.dart").unwrap();
//     let x = &parse_dart_code(&content)[0];
//     dbg!(x);
//
//     let mut output = String::new();
//
//     generate_class(&mut output, x);
//
//     println!("{output}");
// }

fn traverse_directory(path: &Path) -> Vec<PathBuf> {
    let Ok(directory) = fs::read_dir(path) else {
        eprintln!("[E] Error reading directory {path:?}");
        return Vec::new();
    };

    let mut files = Vec::new();

    for entry in directory {
        let Ok(entry) = entry else {
            eprintln!("[E] Error reading directory {path:?}");
            continue;
        };

        if entry.path().is_file() {
            let path = entry.path();
            let name = path.file_name().unwrap().to_str().unwrap();
            if name.ends_with(".dart")
                && !name.ends_with(".g.dart")
                && !name.ends_with(".freezed.dart")
            {
                files.push(entry.path());
            }
        }

        if entry.path().is_dir() {
            files.extend(traverse_directory(&entry.path()));
        }
    }

    files
}

fn process_file(data: &str, path: &Path) {
    let classes = parse_dart_code(data);
    if classes.is_empty() {
        eprintln!(
            "Found '@freezed' string in {:?} but couldn't parse it",
            path
        );
        return;
    }

    let part_of = format!(
        "part of '../{}';",
        path.file_name().unwrap().to_str().unwrap()
    );

    let identity = "\nT _$identity<T>(T value) => value;\n";

    let mut freezed_file = part_of.clone();
    freezed_file.push_str(identity);

    let mut g_file = part_of.clone();

    let init_g_len = g_file.len();
    let init_freezed_len = freezed_file.len();

    for class in classes {
        generate_class(&mut freezed_file, &mut g_file, &class);
    }

    let mut parent_dir = path.parent().unwrap().to_owned();
    let file_name = path.file_stem().unwrap().to_str().unwrap();
    let freezed_file_name = format!("{}.freezed.dart", file_name);
    let json_file_name = format!("{}.g.dart", file_name);

    parent_dir.push("generated");
    let r = std::fs::create_dir(&parent_dir);
    assert!(r.is_ok() || r.is_err_and(|e| e.kind() == ErrorKind::AlreadyExists));

    let freezed_file_path = parent_dir.join(freezed_file_name);
    let g_file_path = parent_dir.join(json_file_name);

    if freezed_file.len() > init_freezed_len {
        std::fs::write(freezed_file_path, freezed_file).unwrap();
    } else {
        panic!("Tried to write empty file, while there's a parsed class.");
    }

    if g_file.len() > init_g_len {
        std::fs::write(g_file_path, g_file).unwrap();
    }
}

fn main() -> ExitCode {
    let start = Instant::now();
    let args: Vec<String> = env::args().collect();
    assert!(!args.is_empty());

    if args.len() != 2 {
        println!("Invalid usage.");
        println!("    Usage: faster_freezed <TARGET_DIRECTORY>");
        return ExitCode::SUCCESS;
    }

    let path = Path::new(&args[1]);
    let dart_files = traverse_directory(path);
    let dart_files_count = dart_files.len();
    let traversing_timer = start.elapsed();

    let mut files_to_process = Vec::new();

    for file in dart_files {
        let x = read_to_string(&file).unwrap();
        if x.contains("@freezed") {
            files_to_process.push((file, x));
        }
    }
    let force_search_timing = start.elapsed();

    for file in &files_to_process {
        //println!("Processing {:?}", file.0);
        process_file(&file.1, &file.0);
    }

    let parsing_and_generating = start.elapsed();

    println!("Took {:?} to discover file tree", traversing_timer);
    println!(
        "Found {} freezed files from {} dart files in {:?}",
        files_to_process.len(),
        dart_files_count,
        force_search_timing - traversing_timer
    );

    println!(
        "Took {:?} to parse and generate.",
        parsing_and_generating - force_search_timing,
    );
    println!("Total: {:?}", start.elapsed());

    ExitCode::SUCCESS
}
