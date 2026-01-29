//! File watcher service for real-time index updates
//!
//! Uses the `notify` crate to watch for filesystem changes and automatically
//! updates the search index when files are created, modified, or deleted.

use crate::services::tantivy_engine;
use notify::{Config, Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{channel, Receiver};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

/// Whether the file watcher is currently running
static WATCHER_RUNNING: AtomicBool = AtomicBool::new(false);

/// Debounce delay in milliseconds
const DEBOUNCE_DELAY_MS: u64 = 500;

/// File event types we handle
#[derive(Debug, Clone, PartialEq)]
pub enum FileEventType {
    Created,
    Modified,
    Deleted,
}

/// A file event with debouncing support
#[derive(Debug, Clone)]
struct PendingEvent {
    path: PathBuf,
    event_type: FileEventType,
    timestamp: Instant,
}

/// Manages the file watcher and processes events
pub struct FileWatcherManager {
    watcher: Option<RecommendedWatcher>,
    watched_folders: Vec<String>,
    pending_events: Arc<Mutex<HashMap<PathBuf, PendingEvent>>>,
}

impl FileWatcherManager {
    /// Creates a new file watcher manager
    pub fn new() -> Self {
        Self {
            watcher: None,
            watched_folders: Vec::new(),
            pending_events: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Starts watching the given folders
    pub fn start(&mut self, folders: Vec<String>) -> Result<(), String> {
        // Create a channel for events
        let (tx, rx) = channel();

        // Create the watcher
        let watcher = RecommendedWatcher::new(
            move |res: Result<Event, notify::Error>| {
                if let Ok(event) = res {
                    let _ = tx.send(event);
                }
            },
            Config::default().with_poll_interval(Duration::from_secs(2)),
        )
        .map_err(|e| format!("Failed to create watcher: {}", e))?;

        self.watcher = Some(watcher);
        self.watched_folders = folders.clone();

        // Add all folders to watch
        for folder in &folders {
            if let Some(ref mut w) = self.watcher {
                let path = std::path::Path::new(folder);
                if path.exists() && path.is_dir() {
                    if let Err(e) = w.watch(path, RecursiveMode::Recursive) {
                        eprintln!("Failed to watch folder {}: {}", folder, e);
                    } else {
                        println!("Watching folder: {}", folder);
                    }
                }
            }
        }

        // Start the event processing thread
        let pending_events = self.pending_events.clone();
        thread::spawn(move || {
            Self::process_events(rx, pending_events);
        });

        Ok(())
    }

    /// Stops watching all folders
    pub fn stop(&mut self) {
        self.watcher = None;
        self.watched_folders.clear();
    }

    /// Adds a folder to watch
    pub fn add_folder(&mut self, folder: &str) -> Result<(), String> {
        if let Some(ref mut w) = self.watcher {
            let path = std::path::Path::new(folder);
            if path.exists() && path.is_dir() {
                w.watch(path, RecursiveMode::Recursive)
                    .map_err(|e| format!("Failed to watch folder: {}", e))?;
                self.watched_folders.push(folder.to_string());
                println!("Added folder to watch: {}", folder);
            }
        }
        Ok(())
    }

    /// Removes a folder from watch
    pub fn remove_folder(&mut self, folder: &str) -> Result<(), String> {
        if let Some(ref mut w) = self.watcher {
            let path = std::path::Path::new(folder);
            let _ = w.unwatch(path); // Ignore errors if not watched
            self.watched_folders.retain(|f| f != folder);
            println!("Removed folder from watch: {}", folder);
        }
        Ok(())
    }

    /// Processes events from the receiver with debouncing
    fn process_events(rx: Receiver<Event>, pending_events: Arc<Mutex<HashMap<PathBuf, PendingEvent>>>) {
        // Event collection thread
        let pending_clone = pending_events.clone();
        thread::spawn(move || {
            for event in rx {
                Self::handle_notify_event(event, &pending_clone);
            }
        });

        // Debounce processing loop
        loop {
            thread::sleep(Duration::from_millis(100));

            let now = Instant::now();
            let mut events_to_process = Vec::new();

            // Collect events that have been debounced long enough
            {
                let mut pending = pending_events.lock().unwrap();
                let debounce_duration = Duration::from_millis(DEBOUNCE_DELAY_MS);
                
                pending.retain(|path, event| {
                    if now.duration_since(event.timestamp) >= debounce_duration {
                        events_to_process.push((path.clone(), event.clone()));
                        false // Remove from pending
                    } else {
                        true // Keep in pending
                    }
                });
            }

            // Process the debounced events
            for (path, event) in events_to_process {
                Self::process_file_event(&path, &event.event_type);
            }
        }
    }

    /// Handles a raw notify event and adds it to pending events
    fn handle_notify_event(event: Event, pending: &Arc<Mutex<HashMap<PathBuf, PendingEvent>>>) {
        let event_type = match event.kind {
            EventKind::Create(_) => Some(FileEventType::Created),
            EventKind::Modify(_) => Some(FileEventType::Modified),
            EventKind::Remove(_) => Some(FileEventType::Deleted),
            _ => None,
        };

        if let Some(etype) = event_type {
            for path in event.paths {
                // Only process indexable files
                if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
                    if is_indexable_ext(ext) {
                        let mut pending_guard = pending.lock().unwrap();
                        pending_guard.insert(
                            path.clone(),
                            PendingEvent {
                                path,
                                event_type: etype.clone(),
                                timestamp: Instant::now(),
                            },
                        );
                    }
                }
            }
        }
    }

    /// Processes a single file event (after debouncing)
    fn process_file_event(path: &PathBuf, event_type: &FileEventType) {
        let path_str = path.to_string_lossy().to_string();
        
        match event_type {
            FileEventType::Created | FileEventType::Modified => {
                if path.exists() && path.is_file() {
                    match tantivy_engine::index_single_file(&path_str) {
                        Ok(_) => println!("Indexed file: {}", path_str),
                        Err(e) => eprintln!("Failed to index file {}: {}", path_str, e),
                    }
                }
            }
            FileEventType::Deleted => {
                match tantivy_engine::delete_file(&path_str) {
                    Ok(_) => println!("Removed from index: {}", path_str),
                    Err(e) => eprintln!("Failed to remove from index {}: {}", path_str, e),
                }
            }
        }
    }
}

/// Check if a file extension is indexable
fn is_indexable_ext(ext: &str) -> bool {
    let ext_lower = ext.to_lowercase();
    let ext_str = ext_lower.as_str();
    
    // Text files (content + filename)
    matches!(ext_str,
        "txt" | "md" | "json" | "rs" | "py" | "js" | "ts" | "tsx" | "jsx" |
        "html" | "css" | "xml" | "yaml" | "yml" | "toml" | "ini" | "conf" |
        "log" | "csv" | "sh" | "bat" | "ps1" | "c" | "cpp" | "h" | "hpp" |
        "java" | "go" | "rb" | "php" | "vue" | "svelte" | "sql" | "r" |
        "scala" | "kt" | "swift" | "dart" | "lua" | "pl" | "pm"
    ) ||
    // Binary files (filename only)
    matches!(ext_str,
        "pdf" | "doc" | "docx" | "xls" | "xlsx" | "ppt" | "pptx" | "odt" | "ods" | "odp" |
        "epub" | "mobi" | "azw" | "azw3" | "fb2" | "djvu" |
        "jpg" | "jpeg" | "png" | "gif" | "bmp" | "svg" | "webp" | "ico" | "tiff" |
        "mp3" | "wav" | "flac" | "ogg" | "mp4" | "mkv" | "avi" | "mov" | "wmv" |
        "zip" | "rar" | "7z" | "tar" | "gz" | "bz2" |
        "exe" | "msi" | "dmg" | "app" | "apk" |
        "iso" | "torrent"
    )
}

// ============================================================================
// Global File Watcher Instance
// ============================================================================

use once_cell::sync::Lazy;

static FILE_WATCHER: Lazy<Mutex<FileWatcherManager>> = Lazy::new(|| {
    Mutex::new(FileWatcherManager::new())
});

/// Starts the global file watcher with the given folders
pub fn start_watching(folders: Vec<String>) -> Result<(), String> {
    if WATCHER_RUNNING.compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst).is_err() {
        return Ok(()); // Already running
    }

    let mut watcher = FILE_WATCHER.lock().map_err(|e| e.to_string())?;
    watcher.start(folders)?;
    
    println!("File watcher started");
    Ok(())
}

/// Stops the global file watcher
pub fn stop_watching() -> Result<(), String> {
    WATCHER_RUNNING.store(false, Ordering::SeqCst);
    
    let mut watcher = FILE_WATCHER.lock().map_err(|e| e.to_string())?;
    watcher.stop();
    
    println!("File watcher stopped");
    Ok(())
}

/// Adds a folder to the global file watcher
pub fn add_watch_folder(folder: &str) -> Result<(), String> {
    if !WATCHER_RUNNING.load(Ordering::SeqCst) {
        return Ok(()); // Watcher not running
    }
    
    let mut watcher = FILE_WATCHER.lock().map_err(|e| e.to_string())?;
    watcher.add_folder(folder)
}

/// Removes a folder from the global file watcher
pub fn remove_watch_folder(folder: &str) -> Result<(), String> {
    let mut watcher = FILE_WATCHER.lock().map_err(|e| e.to_string())?;
    watcher.remove_folder(folder)
}

/// Checks if the file watcher is running
pub fn is_running() -> bool {
    WATCHER_RUNNING.load(Ordering::SeqCst)
}
