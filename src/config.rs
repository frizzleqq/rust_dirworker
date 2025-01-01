use serde::Deserialize;

#[derive(Deserialize)]
pub struct Config {
    pub directories: Vec<DirectoryEntry>,
    pub backup_root_path: Option<String>,
}

#[derive(Deserialize)]
pub struct DirectoryEntry {
    pub path: String,
    #[serde(default)]
    pub include_directories: bool,
    pub action: DirectoryAction,
}

#[derive(Debug, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum DirectoryAction {
    Clean,
    List,
    Analyze,
    Backup,
}

pub fn parse_config(config_data: String) -> Config {
    serde_json::from_str(&config_data).expect("Failed to parse config file")
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
            ],
            "backup_root_path": "/backup"
        }
        "#;

        let config = parse_config(json_data.to_string());

        assert_eq!(config.directories.len(), 2);
        assert_eq!(config.backup_root_path, Some("/backup".to_string()));

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
