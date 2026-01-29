use crate::commands::SearchResult;
use crate::services::tantivy_engine;

/// Primary search function - uses launcher-style fuzzy matching
/// 
/// This matches characters in sequence, so "7r" finds "7 Rules of Power"
pub fn search_files(query: String, limit: u32) -> Result<Vec<SearchResult>, String> {
    // Use launcher-style search for best UX (same as app launchers like Alfred/Raycast)
    let results = tantivy_engine::search_files_launcher(&query, limit as usize)
        .map_err(|e| e.to_string())?;
    Ok(results)
}

/// Search with specific options (fuzzy/prefix matching)
#[allow(dead_code)]
pub fn search_files_with_options(
    query: String,
    limit: u32,
    fuzzy: bool,
    prefix: bool,
) -> Result<Vec<SearchResult>, String> {
    let results = tantivy_engine::search_files_enhanced(&query, limit as usize, fuzzy, prefix)
        .map_err(|e| e.to_string())?;
    Ok(results)
}

/// Simple exact token-based search (no fuzzy, no launcher style)
#[allow(dead_code)]
pub fn search_files_exact(query: String, limit: u32) -> Result<Vec<SearchResult>, String> {
    let results = tantivy_engine::search_files(&query, limit as usize)
        .map_err(|e| e.to_string())?;
    Ok(results)
}
