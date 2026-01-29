use std::path::Path;

pub fn normalize_path(path: &str) -> String {
    let path = Path::new(path);
    path.to_string_lossy().to_string()
}

pub fn get_file_name(path: &str) -> String {
    let path = Path::new(path);
    path.file_name()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_default()
}

pub fn get_extension(path: &str) -> String {
    let path = Path::new(path);
    path.extension()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_default()
}
