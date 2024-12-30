mod config;

use config::{read_config, DirectoryAction, DirectoryEntry};
use std::path::Path;
use std::{env, fs};

impl DirectoryAction {
    fn execute(&self, entry: &DirectoryEntry) {
        match self {
            DirectoryAction::List => list_directory(&entry.path, entry.include_directories),
            DirectoryAction::Clean => clean_directory(&entry.path, entry.include_directories),
            DirectoryAction::Analyze => analyze_directory(&entry.path, entry.include_directories),
        }
    }
}

fn analyze_directory(path: &str, include_directories: bool) {
    println!(
        "Analyzing directory (subdirs={}): '{}'",
        include_directories, path
    );
    let (file_count, total_size) =
        analyze_directory_recursive(Path::new(path), Some(include_directories));

    println!("Number of files: {}", file_count);
    println!("Total size: {} bytes", total_size);
    println!();
}

fn analyze_directory_recursive(path: &Path, include_directories: Option<bool>) -> (u64, u64) {
    let include_directories = include_directories.unwrap_or(false);
    let dir_entries = fs::read_dir(path).expect("Failed to read directory");

    let mut file_count = 0;
    let mut total_size = 0;

    for entry in dir_entries {
        let entry = entry.expect("Failed to read entry");
        let path = entry.path();
        if path.is_dir() {
            if include_directories {
                let dir_stats = analyze_directory_recursive(&path, Some(include_directories));
                file_count += dir_stats.0;
                total_size += dir_stats.1;
            }
        } else {
            file_count += 1;
            total_size += fs::metadata(&path).expect("Failed to read metadata").len();
        }
    }

    (file_count, total_size)
}

fn list_directory(path: &str, include_directories: bool) {
    let dir_entries = fs::read_dir(path).expect("Failed to read directory");

    for entry in dir_entries {
        let entry = entry.expect("Failed to read entry");
        let path = entry.path();
        if path.is_dir() {
            if include_directories {
                println!("Directory: '{}'", path.display());
            }
        } else {
            println!("File: '{}'", path.display());
        }
    }
}

fn clean_directory(path: &str, include_directories: bool) {
    let dir_entries = fs::read_dir(path).expect("Failed to read directory");

    for entry in dir_entries {
        let entry = entry.expect("Failed to read entry");
        let path = entry.path();
        if path.is_dir() {
            if include_directories {
                println!("Removing directory: '{}'", path.display());
                fs::remove_dir_all(path).expect("Failed to remove directory");
            } else {
                println!("Skipping directory: '{}'", path.display());
            }
        } else {
            println!("Removing file: '{}'", path.display());
            fs::remove_file(path).expect("Failed to remove file");
        }
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: {} <config.json>", args[0]);
        std::process::exit(1);
    }

    let config_data = fs::read_to_string(&args[1]).expect("Failed to read config file");
    let config = read_config(config_data);

    for entry in config.directories {
        entry.action.execute(&entry);
    }
}
