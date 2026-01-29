pub mod path_utils;

use std::path::Path;

pub fn is_text_file(path: &Path) -> bool {
    if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
        let text_exts = ["txt", "md", "json", "rs", "py", "js", "ts", "html", "css", "xml", "yaml", "yml", "toml", "ini", "conf", "log", "csv"];
        text_exts.contains(&ext.to_lowercase().as_str())
    } else {
        false
    }
}
