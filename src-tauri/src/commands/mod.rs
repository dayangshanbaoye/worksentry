pub mod index;
pub mod search;
pub mod config;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone)]
pub struct SearchResult {
    pub path: String,
    pub file_name: String,
    pub score: f32,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Config {
    pub indexed_folders: Vec<String>,
    pub hotkey: HotkeyConfig,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct HotkeyConfig {
    pub modifiers: Vec<String>,
    pub key: String,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            indexed_folders: Vec::new(),
            hotkey: HotkeyConfig {
                modifiers: vec!["Alt".to_string()],
                key: "Space".to_string(),
            },
        }
    }
}
