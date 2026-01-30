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

// Update signature to accept flags
// Helper to find all profile directories
fn get_profile_dirs(user_data_dir: &Path) -> Vec<PathBuf> {
    let mut profiles = Vec::new();
    if let Ok(entries) = fs::read_dir(user_data_dir) {
        for entry in entries.filter_map(|e| e.ok()) {
            let path = entry.path();
            if path.is_dir() {
                if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                    // Check for Default or Profile X
                    if name == "Default" || name.starts_with("Profile ") {
                        profiles.push(path);
                    }
                }
            }
        }
    }
    profiles
}

pub fn extract_all_browser_data(enable_history: bool, enable_bookmarks: bool) -> Vec<BrowserData> {
    let mut data = Vec::new();
    let local_app_data = std::env::var("LOCALAPPDATA").unwrap_or_default();
    let base_path = Path::new(&local_app_data);

    // Chrome
    let chrome_user_data = base_path.join(r"Google\Chrome\User Data");
    if chrome_user_data.exists() {
        for profile_dir in get_profile_dirs(&chrome_user_data) {
            let profile_name = profile_dir.file_name().unwrap_or_default().to_string_lossy();
            let source_name = format!("Chrome ({})", profile_name);

            if enable_history {
                match extract_history(&profile_dir.join("History"), &source_name) {
                    Ok(mut history) => data.append(&mut history),
                    Err(e) => eprintln!("Error extracting Chrome history from {:?}: {}", profile_dir, e),
                }
            }
            if enable_bookmarks {
                 match extract_bookmarks(&profile_dir.join("Bookmarks"), &source_name) {
                    Ok(mut bookmarks) => data.append(&mut bookmarks),
                    Err(e) => eprintln!("Error extracting Chrome bookmarks from {:?}: {}", profile_dir, e),
                }
            }
        }
    }

    // Edge
    let edge_user_data = base_path.join(r"Microsoft\Edge\User Data");
    if edge_user_data.exists() {
        for profile_dir in get_profile_dirs(&edge_user_data) {
             let profile_name = profile_dir.file_name().unwrap_or_default().to_string_lossy();
             let source_name = format!("Edge ({})", profile_name);

            if enable_history {
                 match extract_history(&profile_dir.join("History"), &source_name) {
                    Ok(mut history) => data.append(&mut history),
                    Err(e) => eprintln!("Error extracting Edge history from {:?}: {}", profile_dir, e),
                }
            }
            if enable_bookmarks {
                 match extract_bookmarks(&profile_dir.join("Bookmarks"), &source_name) {
                    Ok(mut bookmarks) => data.append(&mut bookmarks),
                    Err(e) => eprintln!("Error extracting Edge bookmarks from {:?}: {}", profile_dir, e),
                }
            }
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
    // Unique temp file name to avoid collisions between profiles
    let random_suffix = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH)?.as_nanos();
    let temp_db_path = temp_dir.join(format!("worksentry_hist_{}_{}.db", source.replace(" ", "_").replace("(", "").replace(")", ""), random_suffix));
    
    // Attempt copy
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

    // Clean up - ignore deletion errors
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
    if let Some(type_str) = node.get("type").and_then(|t| t.as_str()) {
        if type_str == "url" {
            if let Some(url) = node.get("url").and_then(|v| v.as_str()) {
                let name = node.get("name").and_then(|v| v.as_str()).unwrap_or("");
                if !name.is_empty() {
                    results.push(BrowserData {
                        url: url.to_string(),
                        title: name.to_string(),
                        source: source.to_string(),
                        data_type: "Bookmark".to_string(),
                    });
                }
            }
        }
    }

    // Process children logic remains same but improved check above
    if let Some(children) = node.get("children").and_then(|v| v.as_array()) {
        for child in children {
            process_bookmark_node(child, source, results);
        }
    }
    
    // Process roots object
    if node.is_object() {
        for (key, value) in node.as_object().unwrap() {
            // We want to traverse into objects (like "bookmark_bar") regardless of whether they have children
            // The "children" key itself is an Array, so is_object() is false, which prevents re-processing the array as an object.
            if value.is_object() {
                 process_bookmark_node(value, source, results);
            }
        }
    }
}


