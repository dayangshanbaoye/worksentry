use std::path::{Path, PathBuf};
use std::fs;
use serde::{Serialize, Deserialize};
use rusqlite::Connection;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct BrowserData {
    pub url: String,
    pub title: String,
    pub source: String, // "Chrome", "Edge"
    pub data_type: String, // "History", "Bookmark"
}

#[derive(Debug, Serialize, Clone)]
pub struct BrowserStatus {
    pub installed_browsers: Vec<String>,
}

const CHROME_HISTORY_PATH: &str = r"Google\Chrome\User Data\Default\History";
const CHROME_BOOKMARKS_PATH: &str = r"Google\Chrome\User Data\Default\Bookmarks";
const EDGE_HISTORY_PATH: &str = r"Microsoft\Edge\User Data\Default\History";
const EDGE_BOOKMARKS_PATH: &str = r"Microsoft\Edge\User Data\Default\Bookmarks";

pub fn get_installed_browsers() -> Vec<String> {
    let mut browsers = Vec::new();
    let local_app_data = std::env::var("LOCALAPPDATA").unwrap_or_default();
    let base_path = Path::new(&local_app_data);

    if base_path.join(r"Google\Chrome\User Data").exists() {
        browsers.push("Google Chrome".to_string());
    }
    if base_path.join(r"Microsoft\Edge\User Data").exists() {
        browsers.push("Microsoft Edge".to_string());
    }
    browsers
}

pub fn extract_all_browser_data() -> Vec<BrowserData> {
    let mut data = Vec::new();
    let local_app_data = std::env::var("LOCALAPPDATA").unwrap_or_default();
    let base_path = Path::new(&local_app_data);

    if base_path.join(r"Google\Chrome\User Data").exists() {
        if let Ok(mut history) = extract_history(&base_path.join(CHROME_HISTORY_PATH), "Google Chrome") {
            data.append(&mut history);
        }
        if let Ok(mut bookmarks) = extract_bookmarks(&base_path.join(CHROME_BOOKMARKS_PATH), "Google Chrome") {
            data.append(&mut bookmarks);
        }
    }

    if base_path.join(r"Microsoft\Edge\User Data").exists() {
        if let Ok(mut history) = extract_history(&base_path.join(EDGE_HISTORY_PATH), "Microsoft Edge") {
            data.append(&mut history);
        }
        if let Ok(mut bookmarks) = extract_bookmarks(&base_path.join(EDGE_BOOKMARKS_PATH), "Microsoft Edge") {
            data.append(&mut bookmarks);
        }
    }

    data
}

fn extract_history(path: &Path, source: &str) ->  Result<Vec<BrowserData>, Box<dyn std::error::Error>> {
    if !path.exists() {
        return Ok(Vec::new());
    }

    // Copy to temp file to avoid lock issues
    let temp_dir = std::env::temp_dir();
    let temp_db_path = temp_dir.join(format!("worksentry_hist_{}.db", source.replace(" ", "_")));
    fs::copy(path, &temp_db_path)?;

    let conn = Connection::open(&temp_db_path)?;
    let mut stmt = conn.prepare("SELECT url, title, visit_count, last_visit_time FROM urls ORDER BY visit_count DESC LIMIT 2000")?;
    
    let rows = stmt.query_map([], |row| {
        Ok(BrowserData {
            url: row.get(0)?,
            title: row.get(1)?,
            source: source.to_string(),
            data_type: "History".to_string(),
        })
    })?;

    let mut results = Vec::new();
    for row in rows {
        if let Ok(data) = row {
            if !data.title.is_empty() {
                results.push(data);
            }
        }
    }

    // Clean up
    let _ = fs::remove_file(temp_db_path);

    Ok(results)
}

fn extract_bookmarks(path: &Path, source: &str) -> Result<Vec<BrowserData>, Box<dyn std::error::Error>> {
    if !path.exists() {
        return Ok(Vec::new());
    }

    let content = fs::read_to_string(path)?;
    let json: serde_json::Value = serde_json::from_str(&content)?;
    
    let mut results = Vec::new();
    if let Some(roots) = json.get("roots") {
        process_bookmark_node(roots, source, &mut results);
    }

    Ok(results)
}

fn process_bookmark_node(node: &serde_json::Value, source: &str, results: &mut Vec<BrowserData>) {
    if let Some(url) = node.get("url").and_then(|v| v.as_str()) {
        if let Some(name) = node.get("name").and_then(|v| v.as_str()) {
             results.push(BrowserData {
                url: url.to_string(),
                title: name.to_string(),
                source: source.to_string(),
                data_type: "Bookmark".to_string(),
            });
        }
    }

    // Process children
    if let Some(children) = node.get("children").and_then(|v| v.as_array()) {
        for child in children {
            process_bookmark_node(child, source, results);
        }
    }
    
    // Process roots object (bookmark_bar, other, etc)
    if node.is_object() {
        for (_key, value) in node.as_object().unwrap() {
            if value.is_object() || value.is_array() {
                 process_bookmark_node(value, source, results);
            }
        }
    }
}
