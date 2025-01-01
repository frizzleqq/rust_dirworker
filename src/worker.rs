use crate::config::{parse_config, DirectoryAction, DirectoryEntry};
use chrono::Utc;
use std::fs;
use std::fs::File;
use std::io::{BufWriter, Read, Write};
use std::path::Path;
use walkdir::WalkDir;
use zip::write::SimpleFileOptions;

impl DirectoryAction {
    fn execute(&self, entry: &DirectoryEntry, timestamp: &str, backup_root_path: Option<&str>) {
        match self {
            DirectoryAction::List => list_directory(&entry.path, entry.include_directories),
            DirectoryAction::Clean => clean_directory(&entry.path, entry.include_directories),
            DirectoryAction::Analyze => analyze_directory(&entry.path, entry.include_directories),
            DirectoryAction::Backup => {
                if let Some(backup_root_path) = backup_root_path {
                    backup_directory(&entry.path, timestamp, backup_root_path);
                } else {
                    panic!("Backup requires 'backup_root_path' in config.");
                }
            }
        }
    }
}

pub fn run_worker(config_path: &str) {
    let timestamp = Utc::now().format("%Y%m%d%H%M%S").to_string();
    let config_data = fs::read_to_string(config_path).expect("Failed to read config file");
    let config = parse_config(config_data);

    for entry in config.directories {
        entry
            .action
            .execute(&entry, &timestamp, config.backup_root_path.as_deref());
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

fn backup_directory(path: &str, timestamp: &str, backup_root_path: &str) {
    let src_dir = Path::new(path);
    let dir_name = src_dir.file_name().unwrap().to_str().unwrap();
    let zip_file_name = format!("{}/{}_{}.zip", backup_root_path, dir_name, timestamp);
    println!("Creating backup of '{}' -> '{}'", path, zip_file_name);

    fs::create_dir_all(backup_root_path).expect("Failed to create backup root directory");
    let file = File::create(&zip_file_name).expect("Failed to create zip file");

    let writer = BufWriter::new(file);
    let mut zip = zip::ZipWriter::new(writer);
    let options = SimpleFileOptions::default()
        .compression_method(zip::CompressionMethod::Stored)
        .unix_permissions(0o755);

    for entry in WalkDir::new(src_dir).into_iter().filter_map(|e| e.ok()) {
        let path = entry.path();
        let name = path.strip_prefix(src_dir).unwrap();
        let path_as_string = name.to_str().map(str::to_owned).unwrap();
        let mut buffer = Vec::new();

        if path.is_file() {
            println!("Adding file {}", path.display());
            zip.start_file(path_as_string, options).unwrap();
            let mut f = File::open(path).expect("Failed to open file");

            f.read_to_end(&mut buffer).expect("Failed to read file");
            zip.write_all(&buffer).unwrap();
            buffer.clear();
        } else if !name.as_os_str().is_empty() {
            println!("Adding dir {}", path.display());
            zip.add_directory(path_as_string, options).unwrap();
        }
    }

    zip.finish().unwrap();
    println!("Backup created: '{}'", zip_file_name);
    println!();
}
