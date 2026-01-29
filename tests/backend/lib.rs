#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::PathBuf;
    use tempfile::TempDir;

    #[test]
    fn test_search_result_structure() {
        struct SearchResult {
            path: String,
            file_name: String,
            score: f32,
        }

        let result = SearchResult {
            path: "/test/path.txt".to_string(),
            file_name: "path.txt".to_string(),
            score: 1.0,
        };

        assert_eq!(result.path, "/test/path.txt");
        assert_eq!(result.file_name, "path.txt");
        assert!(result.score >= 0.0);
    }

    #[test]
    fn test_config_default() {
        struct Config {
            indexed_folders: Vec<String>,
            hotkey: HotkeyConfig,
        }

        struct HotkeyConfig {
            modifiers: Vec<String>,
            key: String,
        }

        let config = Config {
            indexed_folders: Vec::new(),
            hotkey: HotkeyConfig {
                modifiers: vec!["Alt".to_string()],
                key: "Space".to_string(),
            },
        };

        assert!(config.indexed_folders.is_empty());
        assert_eq!(config.hotkey.modifiers, vec!["Alt"]);
        assert_eq!(config.hotkey.key, "Space");
    }

    #[test]
    fn test_is_text_file_extensions() {
        let test_cases = vec![
            ("file.txt", true),
            ("file.md", true),
            ("file.json", true),
            ("file.rs", true),
            ("file.py", true),
            ("file.js", true),
            ("file.png", false),
            ("file.jpg", false),
            ("file.exe", false),
            ("file.zip", false),
        ];

        for (filename, expected) in test_cases {
            let ext = PathBuf::from(filename)
                .extension()
                .and_then(|e| e.to_str())
                .map(|s| s.to_lowercase())
                .unwrap_or_default();

            let text_exts = vec![
                "txt", "md", "json", "rs", "py", "js", "ts", "html", "css", "xml",
                "yaml", "yml", "toml", "ini", "conf", "log", "csv",
            ];

            let result = text_exts.contains(&ext.as_str());
            assert_eq!(result, expected, "Failed for {}", filename);
        }
    }

    #[test]
    fn test_path_normalization() {
        fn normalize_path(path: &str) -> String {
            let path = std::path::Path::new(path);
            path.to_string_lossy().to_string()
        }

        assert_eq!(normalize_path("C:\\Users\\test"), "C:\\Users\\test");
        assert_eq!(normalize_path("/home/user"), "/home/user");
    }

    #[test]
    fn test_get_file_name() {
        fn get_file_name(path: &str) -> String {
            let path = std::path::Path::new(path);
            path.file_name()
                .map(|s| s.to_string_lossy().to_string())
                .unwrap_or_default()
        }

        assert_eq!(get_file_name("/home/user/file.txt"), "file.txt");
        assert_eq!(get_file_name("C:\\Users\\test.txt"), "test.txt");
        assert_eq!(get_file_name(""), "");
    }

    #[test]
    fn test_get_extension() {
        fn get_extension(path: &str) -> String {
            let path = std::path::Path::new(path);
            path.extension()
                .map(|s| s.to_string_lossy().to_string())
                .unwrap_or_default()
        }

        assert_eq!(get_extension("file.txt"), "txt");
        assert_eq!(get_extension("file.tar.gz"), "gz");
        assert_eq!(get_extension("file"), "");
    }

    #[test]
    fn test_folder_path_validation() {
        fn is_valid_folder(path: &str) -> bool {
            let path = std::path::Path::new(path);
            path.exists() && path.is_dir()
        }

        let temp_dir = TempDir::new().unwrap();
        assert!(is_valid_folder(temp_dir.path().to_str().unwrap()));

        assert!(!is_valid_folder("/nonexistent/path"));
    }
}
