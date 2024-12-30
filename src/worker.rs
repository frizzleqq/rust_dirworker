use crate::config::{Config, DirectoryAction, DirectoryEntry};
use std::fs;
use std::path::Path;

impl DirectoryAction {
    fn execute(&self, entry: &DirectoryEntry) {
        match self {
            DirectoryAction::List => list_directory(&entry.path, entry.include_directories),
            DirectoryAction::Clean => clean_directory(&entry.path, entry.include_directories),
            DirectoryAction::Analyze => analyze_directory(&entry.path, entry.include_directories),
        }
    }
}

pub fn parse_config(config_data: String) -> Config {
    serde_json::from_str(&config_data).expect("Failed to parse config file")
}

pub fn run_worker(config_path: &str) {
    let config_data = fs::read_to_string(config_path).expect("Failed to read config file");
    let config = parse_config(config_data);

    for entry in config.directories {
        entry.action.execute(&entry);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_config() {
        let json_data = r#"
        {
            "directories": [
                {
                    "path": "/path/to/dir1",
                    "include_directories": true,
                    "action": "list"
                },
                {
                    "path": "/path/to/dir2",
                    "include_directories": false,
                    "action": "clean"
                }
            ]
        }
        "#;

        let config = parse_config(json_data.to_string());

        assert_eq!(config.directories.len(), 2);

        let dir1 = &config.directories[0];
        assert_eq!(dir1.path, "/path/to/dir1");
        assert!(dir1.include_directories);
        assert_eq!(dir1.action, DirectoryAction::List);

        let dir2 = &config.directories[1];
        assert_eq!(dir2.path, "/path/to/dir2");
        assert!(!dir2.include_directories);
        assert_eq!(dir2.action, DirectoryAction::Clean);
    }

    #[test]
    fn test_parse_config_with_default_include_directories() {
        let json_data = r#"
        {
            "directories": [
                {
                    "path": "/path/to/dir",
                    "action": "analyze"
                }
            ]
        }
        "#;

        let config = parse_config(json_data.to_string());

        assert_eq!(config.directories.len(), 1);

        let dir = &config.directories[0];
        assert_eq!(dir.path, "/path/to/dir");
        assert!(!dir.include_directories);
        assert_eq!(dir.action, DirectoryAction::Analyze);
    }

    #[test]
    #[should_panic(expected = "unknown variant `wrong_action`")]
    fn test_parse_config_wrong_action_error() {
        let json_data = r#"
        {
            "directories": [
                {
                    "path": "/path/to/dir",
                    "action": "wrong_action"
                }
            ]
        }
        "#;

        parse_config(json_data.to_string());
    }

    #[test]
    #[should_panic(expected = "missing field `path`")]
    fn test_parse_config_missing_path_error() {
        let json_data = r#"
        {
            "directories": [
                {
                    "action": "list"
                }
            ]
        }
        "#;

        parse_config(json_data.to_string());
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
