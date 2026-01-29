use crate::services::tantivy_engine;
use crate::services::file_watcher;
use std::path::Path;

pub fn add_indexed_folder(path: String) -> Result<(), String> {
    let path = Path::new(&path);
    if !path.exists() || !path.is_dir() {
        return Err("Invalid folder path".to_string());
    }

    let path_str = path.to_string_lossy().to_string();

    let mut config = crate::commands::config::CONFIG.lock().map_err(|e| e.to_string())?;
    if !config.indexed_folders.contains(&path_str) {
        config.indexed_folders.push(path_str.clone());
        crate::commands::config::save_config(&config)?;
    }
    drop(config);

    // Index the folder
    tantivy_engine::index_folder(&path_str).map_err(|e| e.to_string())?;
    
    // Add to file watcher for real-time updates
    let _ = file_watcher::add_watch_folder(&path_str);
    
    Ok(())
}

pub fn remove_indexed_folder(path: String) -> Result<(), String> {
    // Remove from config
    let mut config = crate::commands::config::CONFIG.lock().map_err(|e| e.to_string())?;
    config.indexed_folders.retain(|p| p != &path);
    crate::commands::config::save_config(&config)?;
    drop(config);
    
    // Remove from file watcher
    let _ = file_watcher::remove_watch_folder(&path);
    
    // Also remove the folder's files from the index
    tantivy_engine::delete_folder(&path).map_err(|e| e.to_string())?;
    Ok(())
}

pub fn get_indexed_folders() -> Result<Vec<String>, String> {
    let config = crate::commands::config::CONFIG.lock().map_err(|e| e.to_string())?;
    Ok(config.indexed_folders.clone())
}

pub fn rebuild_index() -> Result<(), String> {
    let config = crate::commands::config::CONFIG.lock().map_err(|e| e.to_string())?;
    let folders = config.indexed_folders.clone();
    drop(config);

    tantivy_engine::rebuild_index(&folders).map_err(|e| e.to_string())?;
    Ok(())
}

pub fn get_document_count() -> Result<u64, String> {
    tantivy_engine::get_document_count().map_err(|e| e.to_string())
}

pub fn get_index_stats() -> Result<tantivy_engine::IndexStats, String> {
    tantivy_engine::get_index_stats().map_err(|e| e.to_string())
}
