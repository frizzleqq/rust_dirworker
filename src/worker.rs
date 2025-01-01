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

    // sort directory entry  by path and action (so backup is always before clean)
    let mut sorted_entries: Vec<_> = config.directories.iter().collect();
    sorted_entries.sort_by(|a, b| a.path.cmp(&b.path).then(a.action.cmp(&b.action)));

    for entry in sorted_entries {
        entry
            .action
            .execute(entry, &timestamp, config.backup_root_path.as_deref());
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
    println!(
        "Listing directory (subdirs={}): '{}'",
        include_directories, path
    );
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
    println!();
}

fn clean_directory(path: &str, include_directories: bool) {
    println!(
        "Cleaning directory (subdirs={}): '{}'",
        include_directories, path
    );
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
    println!()
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
    let options = SimpleFileOptions::default().compression_method(zip::CompressionMethod::Stored);

    for entry in WalkDir::new(&src_dir).into_iter().filter_map(|e| e.ok()) {
        let path = entry.path();
        let name = path.strip_prefix(&src_dir).unwrap();
        // When using windows sep, not all tools can deal with the subdirs
        let path_as_string = name.to_str().unwrap().replace("\\", "/");
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::{self, File};
    use std::io::{Read, Write};
    use std::path::Path;
    use tempfile::tempdir;
    use zip::read::ZipArchive;

    fn create_test_files(test_dir: &Path) {
        let sub_dir = test_dir.join("subdir");
        let file1 = test_dir.join("file1.txt");
        let file2 = sub_dir.join("file2.txt");

        fs::create_dir_all(&sub_dir).unwrap();
        let mut f1 = File::create(file1).unwrap();
        let mut f2 = File::create(file2).unwrap();

        f1.write_all(b"Hello, world!").unwrap();
        f2.write_all(b"Hello, subdir!").unwrap();
    }

    #[test]
    fn test_backup_directory() {
        let temp_dir = tempdir().unwrap();
        let test_dir = temp_dir.path().join("dummy");
        let backup_root_path = temp_dir.path().join("backup");
        let timestamp = "20250101121500";

        create_test_files(&test_dir);

        backup_directory(
            test_dir.to_str().unwrap(),
            timestamp,
            backup_root_path.to_str().unwrap(),
        );

        let zip_file_name = backup_root_path.join(format!("dummy_{}.zip", timestamp));
        assert!(zip_file_name.exists(), "Backup zip file was not created");

        let file = File::open(&zip_file_name).expect("Failed to open zip file");
        let mut zip = ZipArchive::new(file).expect("Failed to read zip archive");

        let mut file1_content = String::new();
        zip.by_name("file1.txt")
            .expect("file1.txt not found in zip")
            .read_to_string(&mut file1_content)
            .expect("Failed to read file1.txt");
        assert_eq!(file1_content, "Hello, world!");

        let mut file2_content = String::new();
        zip.by_name("subdir/file2.txt")
            .expect("subdir/file2.txt not found in zip")
            .read_to_string(&mut file2_content)
            .expect("Failed to read file2.txt");
        assert_eq!(file2_content, "Hello, subdir!");
    }
}
