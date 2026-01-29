#![windows_subsystem = "windows"]

mod commands;
mod services;
mod utils;

use commands::{index, search, config};
use services::tantivy_engine;
use tauri::Manager;

#[tauri::command]
async fn search(query: String, limit: u32) -> Result<Vec<commands::SearchResult>, String> {
    search::search_files(query, limit)
}

#[tauri::command]
async fn add_folder(path: String) -> Result<(), String> {
    index::add_indexed_folder(path)
}

#[tauri::command]
async fn remove_folder(path: String) -> Result<(), String> {
    index::remove_indexed_folder(path)
}

#[tauri::command]
fn get_folders() -> Result<Vec<String>, String> {
    index::get_indexed_folders()
}

#[tauri::command]
async fn reindex() -> Result<(), String> {
    index::rebuild_index()
}

#[tauri::command]
fn get_document_count() -> Result<u64, String> {
    index::get_document_count()
}

#[tauri::command]
fn get_index_stats() -> Result<services::tantivy_engine::IndexStats, String> {
    index::get_index_stats()
}


#[tauri::command]
fn get_config() -> Result<commands::Config, String> {
    config::get_config()
}

#[tauri::command]
async fn set_hotkey(modifiers: Vec<String>, key: String) -> Result<(), String> {
    config::set_hotkey(modifiers, key)
}

#[tauri::command]
async fn open_file(path: String) -> Result<(), String> {
    #[cfg(target_os = "windows")]
    {
        use std::process::Command;
        Command::new("explorer.exe")
            .args(["/select,", &path])
            .spawn()
            .map_err(|e| e.to_string())?;
    }
    Ok(())
}

fn main() {
    tantivy_engine::init().expect("Failed to initialize Tantivy");

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .setup(|app| {
            // Trigger initial scan and start watcher in background
            tauri::async_runtime::spawn(async move {
                if let Ok(cfg) = commands::config::get_config() {
                    if !cfg.indexed_folders.is_empty() {
                        // 1. Start watcher
                        if let Err(e) = services::file_watcher::start_watching(cfg.indexed_folders.clone()) {
                            eprintln!("Failed to start file watcher: {}", e);
                        }
                        
                        // 2. Initial scan to index missed files
                        println!("Starting initial index scan...");
                        for folder in cfg.indexed_folders {
                            if let Err(e) = services::tantivy_engine::index_folder(&folder) {
                                eprintln!("Failed to scan folder {}: {}", folder, e);
                            }
                        }
                        println!("Initial index scan complete.");
                    }
                }
            });
            Ok(())
        })

        .invoke_handler(tauri::generate_handler![
            search,
            add_folder,
            remove_folder,
            get_folders,
            reindex,
            get_document_count,
            get_index_stats,
            get_config,

            set_hotkey,
            open_file
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
