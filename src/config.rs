use serde::Deserialize;

#[derive(Deserialize)]
pub struct Config {
    pub directories: Vec<DirectoryEntry>,
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
}
