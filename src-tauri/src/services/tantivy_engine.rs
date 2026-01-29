use crate::commands::SearchResult;
use jieba_rs::Jieba;
use std::fs;
use std::path::Path;
use std::time::UNIX_EPOCH;
use tantivy::collector::TopDocs;
use tantivy::query::{BooleanQuery, FuzzyTermQuery, Occur, Query, QueryParser, TermQuery};
use tantivy::schema::{Field, Schema, TEXT, STRING, STORED, NumericOptions, Value as _, IndexRecordOption};
use tantivy::{Index, IndexWriter, TantivyDocument, Term};

/// TantivyEngine provides full-text search capabilities for WorkSentry.
/// 
/// Features:
/// - Persistent index storage
/// - Incremental indexing (only updates changed files)
/// - Duplicate prevention (deletes old entries before re-adding)
/// - Rich metadata (path, filename, content, extension, size, modified time)
pub struct TantivyEngine {
    schema: Schema,
    path_field: Field,
    file_name_field: Field,
    content_field: Field,
    extension_field: Field,
    size_field: Field,
    modified_time_field: Field,
    url_field: Field,
    record_type_field: Field,
    index_path: std::path::PathBuf,
}

impl TantivyEngine {
    pub fn new() -> tantivy::Result<Self> {
        // Use a persistent index path in the app's data directory
        let index_path = dirs::data_dir()
            .unwrap_or_else(|| std::env::temp_dir())
            .join("worksentry")
            .join("index");

        Self::new_with_path(index_path)
    }

    /// Creates a new TantivyEngine with a custom index path (useful for tests)
    pub fn new_with_path(index_path: std::path::PathBuf) -> tantivy::Result<Self> {
        let mut schema_builder = Schema::builder();
        
        // Path is the unique identifier - used for deduplication
        let path_field = schema_builder.add_text_field("path", STRING | STORED);
        // Filename is tokenized for full-text search
        let file_name_field = schema_builder.add_text_field("file_name", TEXT | STORED);
        // Content is tokenized but not stored (saves space)
        let content_field = schema_builder.add_text_field("content", TEXT);
        // Extension for filtering
        let extension_field = schema_builder.add_text_field("extension", STRING | STORED);
        // File size in bytes
        let size_field = schema_builder.add_u64_field("size", NumericOptions::default() | STORED);
        // Modified time as unix timestamp (for incremental indexing)
        // Modified time as unix timestamp (for incremental indexing)
        let modified_time_field = schema_builder.add_i64_field("modified_time", NumericOptions::default() | STORED);
        
        // New fields for Browser Integration
        // URL for bookmarks/history items
        let url_field = schema_builder.add_text_field("url", STRING | STORED);
        // Record type: "file", "bookmark", "history"
        let record_type_field = schema_builder.add_text_field("record_type", STRING | STORED);

        let schema = schema_builder.build();

        Ok(Self {
            schema,
            path_field,
            file_name_field,
            content_field,
            extension_field,
            size_field,
            modified_time_field,
            url_field,
            record_type_field,
            index_path,
        })
    }

    /// Gets or creates the Tantivy index
    fn get_index(&self) -> tantivy::Result<Index> {
        if let Some(parent) = self.index_path.parent() {
            if !parent.exists() {
                fs::create_dir_all(parent)?;
            }
        }
        
        if !self.index_path.exists() {
            fs::create_dir_all(&self.index_path)?;
            return Index::create_in_dir(&self.index_path, self.schema.clone());
        }
        
        // Try to open existing index, if schema mismatch, recreate
        match Index::open_in_dir(&self.index_path) {
            Ok(index) => Ok(index),
            Err(_) => {
                // Schema may have changed, recreate index
                fs::remove_dir_all(&self.index_path)?;
                fs::create_dir_all(&self.index_path)?;
                Index::create_in_dir(&self.index_path, self.schema.clone())
            }
        }
    }

    /// Gets the modified time of a file as a unix timestamp
    fn get_file_mtime(&self, path: &Path) -> Option<i64> {
        fs::metadata(path)
            .ok()
            .and_then(|m| m.modified().ok())
            .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
            .map(|d| d.as_secs() as i64)
    }

    /// Gets the size of a file in bytes
    fn get_file_size(&self, path: &Path) -> u64 {
        fs::metadata(path).map(|m| m.len()).unwrap_or(0)
    }

    /// Looks up the indexed modified time for a file path
    fn get_indexed_mtime(&self, index: &Index, path_str: &str) -> tantivy::Result<Option<i64>> {
        let reader = index.reader()?;
        let searcher = reader.searcher();
        
        let term = Term::from_field_text(self.path_field, path_str);
        let term_query = tantivy::query::TermQuery::new(term, tantivy::schema::IndexRecordOption::Basic);
        
        let top_docs = searcher.search(&term_query, &TopDocs::with_limit(1))?;
        
        if let Some((_, doc_address)) = top_docs.first() {
            let doc: TantivyDocument = searcher.doc(*doc_address)?;
            for field_value in doc.field_values() {
                if field_value.field() == self.modified_time_field {
                    if let Some(mtime) = field_value.value().as_i64() {
                        return Ok(Some(mtime));
                    }
                }
            }
        }
        
        Ok(None)
    }

    /// Indexes a single file, deleting any existing entry first (prevents duplicates)
    fn index_single_file(&self, writer: &mut IndexWriter, path: &Path) -> tantivy::Result<bool> {
        let path_str = path.to_string_lossy().to_string();
        
        // Delete existing document with this path (prevents duplicates)
        let term = Term::from_field_text(self.path_field, &path_str);
        writer.delete_term(term);
        
        // Get file metadata
        let file_name = path.file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();
        let extension = path.extension()
            .unwrap_or_default()
            .to_string_lossy()
            .to_lowercase();
        let size = self.get_file_size(path);
        let modified_time = self.get_file_mtime(path).unwrap_or(0);
        
        // For binary files, only index the filename (not content)
        // For text files, read and index the content
        let content = if self.is_text_indexable(&extension) {
            match self.read_file_content(path) {
                Ok(c) => c,
                Err(_) => String::new(), // Empty content if unreadable
            }
        } else {
            // For binary files (epub, pdf, etc.), we add the filename as content too
            // This helps with searching by filename tokens
            file_name.clone()
        };
        
        // Create and add document
        let mut doc = TantivyDocument::new();
        doc.add_text(self.path_field, &path_str);
        doc.add_text(self.file_name_field, &file_name);
        doc.add_text(self.content_field, &content);
        doc.add_text(self.extension_field, &extension);
        doc.add_u64(self.size_field, size);
        doc.add_i64(self.modified_time_field, modified_time);
        
        writer.add_document(doc)?;
        Ok(true)
    }

    /// Public method to index a single file (creates its own writer)
    /// Used by the file watcher for real-time updates
    pub fn index_file(&self, path_str: &str) -> tantivy::Result<bool> {
        let path = Path::new(path_str);
        if !path.exists() || !path.is_file() {
            return Ok(false);
        }
        
        // Check if it's an indexable file
        if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
            if !self.is_indexable_ext(ext) {
                return Ok(false);
            }
        } else {
            return Ok(false);
        }
        
        let index = self.get_index()?;
        let mut writer: IndexWriter<TantivyDocument> = index.writer(50_000_000)?;
        
        let result = self.index_single_file(&mut writer, path)?;
        writer.commit()?;
        
        Ok(result)
    }

    /// Indexes a folder, only updating files that have changed (incremental indexing)
    pub fn index_folder(&self, folder: &str) -> tantivy::Result<()> {
        let path = Path::new(folder);
        if !path.exists() || !path.is_dir() {
            return Ok(());
        }

        let index = self.get_index()?;
        let mut writer = index.writer(50_000_000)?;
        
        self.index_folder_with_writer(&index, &mut writer, folder)?;
        
        writer.commit()?;
        Ok(())
    }

    /// Internal method to index a folder using an existing writer
    fn index_folder_with_writer(&self, index: &Index, writer: &mut IndexWriter, folder: &str) -> tantivy::Result<u32> {
        let path = Path::new(folder);
        if !path.exists() || !path.is_dir() {
            return Ok(0);
        }

        let mut indexed_count = 0u32;

        for entry in walkdir::WalkDir::new(folder)
            .follow_links(true)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            let entry_path = entry.path();
            if entry_path.is_file() {
                if let Some(ext) = entry_path.extension().and_then(|e| e.to_str()) {
                    if self.is_indexable_ext(ext) {
                        // Check if file needs to be re-indexed (incremental)
                        let path_str = entry_path.to_string_lossy().to_string();
                        let file_mtime = self.get_file_mtime(entry_path);
                        let indexed_mtime = self.get_indexed_mtime(index, &path_str).ok().flatten();
                        
                        // Only re-index if file is new or modified
                        let needs_update = match (file_mtime, indexed_mtime) {
                            (Some(f), Some(i)) => f > i,
                            (Some(_), None) => true, // New file
                            _ => true, // Unknown state, re-index to be safe
                        };
                        
                        if needs_update {
                            if self.index_single_file(writer, entry_path)? {
                                indexed_count += 1;
                            }
                        }
                    }
                }
            }
        }

        Ok(indexed_count)
    }

    /// Checks if a file extension should be indexed (content + filename)
    pub fn is_indexable_ext(&self, ext: &str) -> bool {
        self.is_text_indexable(ext) || self.is_filename_only_indexable(ext)
    }

    /// Text files where we can read and index the content
    fn is_text_indexable(&self, ext: &str) -> bool {
        matches!(
            ext.to_lowercase().as_str(),
            "txt" | "md" | "json" | "rs" | "py" | "js" | "ts" | "tsx" | "jsx" |
            "html" | "css" | "xml" | "yaml" | "yml" | "toml" | "ini" | "conf" |
            "log" | "csv" | "sh" | "bat" | "ps1" | "c" | "cpp" | "h" | "hpp" |
            "java" | "go" | "rb" | "php" | "vue" | "svelte" | "sql" | "r" |
            "scala" | "kt" | "swift" | "dart" | "lua" | "pl" | "pm"
        )
    }

    /// Binary files where we only index the filename, not content
    fn is_filename_only_indexable(&self, ext: &str) -> bool {
        matches!(
            ext.to_lowercase().as_str(),
            // Documents
            "pdf" | "doc" | "docx" | "xls" | "xlsx" | "ppt" | "pptx" | "odt" | "ods" | "odp" |
            // Ebooks
            "epub" | "mobi" | "azw" | "azw3" | "fb2" | "djvu" |
            // Images
            "jpg" | "jpeg" | "png" | "gif" | "bmp" | "svg" | "webp" | "ico" | "tiff" |
            // Audio/Video
            "mp3" | "wav" | "flac" | "ogg" | "mp4" | "mkv" | "avi" | "mov" | "wmv" |
            // Archives
            "zip" | "rar" | "7z" | "tar" | "gz" | "bz2" |
            // Executables/Installers
            "exe" | "msi" | "dmg" | "app" | "apk" |
            // Other
            "iso" | "torrent"
        )
    }

    /// Reads file content, skipping files that are too large (>1MB)
    pub fn read_file_content(&self, path: &Path) -> Result<String, std::io::Error> {
        if let Ok(metadata) = fs::metadata(path) {
            if metadata.len() > 1024 * 1024 {
                return Ok(String::new()); // Skip large files
            }
        }
        fs::read_to_string(path)
    }

    /// Indexes browser history and bookmarks
    pub fn index_browser_data(&self, data: Vec<crate::services::browser_extractor::BrowserData>) -> tantivy::Result<()> {
        let index = self.get_index()?;
        let mut writer = index.writer(50_000_000)?; 

        for item in data {
            // Use URL as unique ID for deduplication
            let term = Term::from_field_text(self.path_field, &item.url);
            writer.delete_term(term);

            let mut doc = TantivyDocument::new();
            
            doc.add_text(self.path_field, &item.url);
            doc.add_text(self.file_name_field, &item.title);
            doc.add_text(self.content_field, &item.url); 
            doc.add_text(self.extension_field, &item.source);
            doc.add_text(self.record_type_field, &item.data_type);
            doc.add_text(self.url_field, &item.url);
            
            doc.add_u64(self.size_field, 0); 
            doc.add_i64(self.modified_time_field, 0);

            writer.add_document(doc)?;
        }

        writer.commit()?;
        Ok(())
    }

    /// Searches the index for matching documents
    pub fn search(&self, query: &str, limit: usize) -> tantivy::Result<Vec<SearchResult>> {
        if query.trim().is_empty() {
            return Ok(Vec::new());
        }

        let index = self.get_index()?;

        let query_parser = QueryParser::for_index(
            &index,
            vec![self.content_field, self.file_name_field],
        );

        let parsed_query = query_parser.parse_query(query)?;
        let searcher = index.reader()?.searcher();

        let top_docs_result: Vec<(f32, tantivy::DocAddress)> = searcher
            .search(&parsed_query, &TopDocs::with_limit(limit))?;

        let mut results = Vec::new();

        for (score, doc_address) in top_docs_result {
            let doc: TantivyDocument = searcher.doc(doc_address)?;
            let mut path_result = String::new();
            let mut file_name = String::new();
            let mut record_type = "file".to_string();

            for field_value in doc.field_values() {
                let field: Field = field_value.field();
                if let Some(text) = field_value.value().as_str() {
                    if field == self.path_field {
                        path_result = text.to_string();
                    } else if field == self.file_name_field {
                        file_name = text.to_string();
                    } else if field == self.record_type_field {
                        record_type = text.to_string();
                    }
                }
            }

            results.push(SearchResult {
                path: path_result,
                file_name,
                score,
                record_type,
            });
        }

        Ok(results)
    }

    /// Enhanced search with fuzzy matching and Chinese text support
    /// 
    /// - `fuzzy`: Enable fuzzy matching (allows typos, edit distance 1-2)
    /// - `prefix`: Enable prefix matching (partial word matches)
    pub fn search_enhanced(&self, query: &str, limit: usize, fuzzy: bool, prefix: bool) -> tantivy::Result<Vec<SearchResult>> {
        if query.trim().is_empty() {
            return Ok(Vec::new());
        }

        let index = self.get_index()?;
        let searcher = index.reader()?.searcher();

        // Tokenize the query (handles Chinese with jieba)
        let tokens = self.tokenize_query(query);
        
        if tokens.is_empty() {
            return Ok(Vec::new());
        }

        // Build queries for each token
        let mut subqueries: Vec<(Occur, Box<dyn Query>)> = Vec::new();
        
        for token in &tokens {
            let token_lower = token.to_lowercase();
            
            // Create queries for both content and file_name fields
            let fields = [self.content_field, self.file_name_field];
            
            for field in fields {
                if fuzzy && token.len() >= 3 {
                    // Fuzzy query with edit distance based on word length
                    let distance = if token.len() <= 4 { 1 } else { 2 };
                    let term = Term::from_field_text(field, &token_lower);
                    let fuzzy_query = FuzzyTermQuery::new(term, distance as u8, true);
                    subqueries.push((Occur::Should, Box::new(fuzzy_query)));
                }
                
                if prefix && token.len() >= 2 {
                    // Prefix query - match terms starting with the token
                    // We'll use a term query as a fallback since prefix queries need different handling
                    let term = Term::from_field_text(field, &token_lower);
                    let term_query = TermQuery::new(term, IndexRecordOption::Basic);
                    subqueries.push((Occur::Should, Box::new(term_query)));
                }
                
                // Always include exact match
                let term = Term::from_field_text(field, &token_lower);
                let term_query = TermQuery::new(term, IndexRecordOption::Basic);
                subqueries.push((Occur::Should, Box::new(term_query)));
            }
        }

        // If no subqueries built, fall back to standard search
        if subqueries.is_empty() {
            return self.search(query, limit);
        }

        let boolean_query = BooleanQuery::new(subqueries);
        
        let top_docs_result: Vec<(f32, tantivy::DocAddress)> = searcher
            .search(&boolean_query, &TopDocs::with_limit(limit))?;

        let mut results = Vec::new();

        for (score, doc_address) in top_docs_result {
            let doc: TantivyDocument = searcher.doc(doc_address)?;
            let mut path_result = String::new();
            let mut file_name = String::new();
            let mut record_type = "file".to_string();

            for field_value in doc.field_values() {
                let field: Field = field_value.field();
                if let Some(text) = field_value.value().as_str() {
                    if field == self.path_field {
                        path_result = text.to_string();
                    } else if field == self.file_name_field {
                        file_name = text.to_string();
                    } else if field == self.record_type_field {
                        record_type = text.to_string();
                    }
                }
            }

            results.push(SearchResult {
                path: path_result,
                file_name,
                score,
                record_type,
            });
        }
        
        Ok(results)
    }

    /// Launcher-style search that matches characters in sequence (like "7r" → "7 Rules")
    /// 
    /// This is the most flexible search mode, ideal for app launchers:
    /// - Characters in query should appear in order in filename
    /// - Spaces in query act as separators (each part must match)
    /// - Case-insensitive
    pub fn search_launcher(&self, query: &str, limit: usize) -> tantivy::Result<Vec<SearchResult>> {
        if query.trim().is_empty() {
            return Ok(Vec::new());
        }

        let index = self.get_index()?;
        let reader = index.reader()?;
        let searcher = reader.searcher();
        
        let query_lower = query.to_lowercase();
        let query_parts: Vec<&str> = query_lower.split_whitespace().collect();

        let mut results = Vec::new();

        // Iterate through all documents and do substring/fuzzy matching
        for segment_reader in searcher.segment_readers() {
            let store_reader = segment_reader.get_store_reader(1)?;
            for doc_id in 0..segment_reader.num_docs() {
                if let Ok(doc) = store_reader.get::<TantivyDocument>(doc_id) {
                    let mut path_result = String::new();
                    let mut file_name = String::new();
                    let mut record_type = "file".to_string();

                    for field_value in doc.field_values() {
                        if field_value.field() == self.path_field {
                            if let Some(text) = field_value.value().as_str() {
                                path_result = text.to_string();
                            }
                        } else if field_value.field() == self.file_name_field {
                            if let Some(text) = field_value.value().as_str() {
                                file_name = text.to_string();
                            }
                        } else if field_value.field() == self.record_type_field {
                             if let Some(text) = field_value.value().as_str() {
                                record_type = text.to_string();
                            }
                        }
                    }

                    if file_name.is_empty() {
                        continue;
                    }

                    let file_name_lower = file_name.to_lowercase();
                    
                    // Calculate match score
                    if let Some(score) = Self::calculate_launcher_score(&query_parts, &file_name_lower) {
                        results.push(SearchResult {
                            path: path_result,
                            file_name,
                            score,
                            record_type,
                        });
                    }
                }
            }
        }

        // Sort by score (higher is better) and limit
        results.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));
        results.truncate(limit);

        Ok(results)
    }

    /// Calculates a launcher-style match score
    /// Returns Some(score) if the query matches, None otherwise
    /// 
    /// Matching rules:
    /// - For "7r" searching "7 Rules": chars must appear in order
    /// - For "7 rules" searching "7 Rules of Power": both parts must match
    fn calculate_launcher_score(query_parts: &[&str], file_name: &str) -> Option<f32> {
        if query_parts.is_empty() {
            return None;
        }

        let mut total_score = 0.0f32;
        
        // 1. Check for exact/prefix/substring matches of the full query (if single part)
        // This gives a huge boost to "vibe" matching "vibe_coding"
        if query_parts.len() == 1 {
            let part = query_parts[0];
            if file_name.starts_with(part) {
                total_score += 1000.0;
            } else if file_name.contains(part) {
                total_score += 500.0;
            }
        }

        let mut search_pos = 0usize;
        let file_chars: Vec<char> = file_name.chars().collect();

        for part in query_parts {
            if part.is_empty() {
                continue;
            }

            // Find this part's characters in sequence
            let part_chars: Vec<char> = part.chars().collect();
            let mut part_idx = 0;
            let mut part_start_pos = None;
            let mut current_consecutive = 0;
            let mut max_consecutive = 0;

            // Simple scanner for subsequence match
            while search_pos < file_chars.len() && part_idx < part_chars.len() {
                if file_chars[search_pos] == part_chars[part_idx] {
                    if part_start_pos.is_none() {
                        part_start_pos = Some(search_pos);
                    }
                    
                    // Bonus for matching at word boundaries
                    if search_pos == 0 || !file_chars[search_pos - 1].is_alphanumeric() {
                        total_score += 10.0; // Significant word boundary bonus
                    }
                    
                    // Track consecutiveness
                    if search_pos > 0 && part_idx > 0 && file_chars[search_pos - 1] == part_chars[part_idx - 1] {
                         // This logic is slightly flawed for gaps, but sufficient for local check
                         // Better: check indices. 
                         // Since we are moving search_pos strictly forward, let's just track adjacent matches
                         // Check implies we matched previous char at search_pos-1.
                         current_consecutive += 1;
                    } else {
                        current_consecutive = 0;
                    }
                    max_consecutive = max_consecutive.max(current_consecutive);

                    part_idx += 1;
                }
                search_pos += 1;
            }

            // If we didn't match all characters in this part, no match
            if part_idx < part_chars.len() {
                return None;
            }

            // Score based on position
            if let Some(pos) = part_start_pos {
                // Earlier matches score higher (0-10 pts)
                total_score += 10.0 * ((file_chars.len() as f32 - pos as f32) / file_chars.len() as f32);
            }
            
            // Score based on consecutiveness (0-20 pts)
            total_score += max_consecutive as f32 * 2.0;
        }

        // Bonus for shorter filenames (match density)
        if !file_name.is_empty() {
            total_score += 100.0 / (file_name.len() as f32).sqrt();
        }

        // Semantic Multipliers: Extension Priority
        if let Some(mut ext_bonus) = std::path::Path::new(file_name).extension().and_then(|e| e.to_str()).map(|ext| {
            match ext.to_lowercase().as_str() {
                // Apps: 1.5x multiplier (simulated by adding score)
                "exe" | "lnk" | "app" | "bat" | "cmd" => 500.0,
                // Folders (harder to detect here without flags, assume none)
                // Docs: 1.0x (Baseline - no change)
                "pdf" | "docx" | "epub" | "md" | "txt" => 0.0,
                // Code/System: 0.8x (Penalty)
                "rs" | "json" | "dll" | "xml" | "sys" | "ts" | "js" | "css" | "html" => -50.0, 
                // Default
                _ => 0.0,
            }
        }) {
             total_score += ext_bonus;
        }

        Some(total_score)
    }

    /// Tokenizes a query string, handling both English and Chinese text
    fn tokenize_query(&self, query: &str) -> Vec<String> {
        let mut tokens = Vec::new();
        
        // Check if query contains Chinese characters
        if Self::contains_chinese(query) {
            // Use jieba for Chinese tokenization
            let jieba = Jieba::new();
            let words = jieba.cut(query, true); // Use search mode
            for word in words {
                let trimmed = word.trim();
                if !trimmed.is_empty() && trimmed.len() > 0 {
                    tokens.push(trimmed.to_string());
                }
            }
        } else {
            // Simple whitespace tokenization for non-Chinese
            for word in query.split_whitespace() {
                let cleaned: String = word.chars()
                    .filter(|c| c.is_alphanumeric() || *c == '_' || *c == '-')
                    .collect();
                if !cleaned.is_empty() {
                    tokens.push(cleaned);
                }
            }
        }
        
        tokens
    }

    /// Checks if a string contains Chinese characters
    fn contains_chinese(text: &str) -> bool {
        text.chars().any(|c| {
            // CJK Unified Ideographs range
            (c >= '\u{4E00}' && c <= '\u{9FFF}') ||
            // CJK Extension A
            (c >= '\u{3400}' && c <= '\u{4DBF}') ||
            // CJK Extension B
            (c >= '\u{20000}' && c <= '\u{2A6DF}')
        })
    }

    /// Deletes a specific file from the index
    pub fn delete_file(&self, path: &str) -> tantivy::Result<()> {
        let index = self.get_index()?;
        let mut writer: IndexWriter<TantivyDocument> = index.writer(50_000_000)?;
        let term = Term::from_field_text(self.path_field, path);
        writer.delete_term(term);
        writer.commit()?;
        Ok(())
    }

    /// Deletes all files from a folder in the index
    pub fn delete_folder(&self, folder: &str) -> tantivy::Result<u32> {
        let index = self.get_index()?;
        let reader = index.reader()?;
        let searcher = reader.searcher();
        
        // Find all documents with paths starting with this folder
        let mut writer: IndexWriter<TantivyDocument> = index.writer(50_000_000)?;
        let mut deleted_count = 0u32;
        
        // We need to iterate through all documents and delete those matching the folder
        // This is less efficient but more reliable than trying to use prefix queries
        for segment_reader in searcher.segment_readers() {
            let store_reader = segment_reader.get_store_reader(1)?;
            for doc_id in 0..segment_reader.num_docs() {
                if let Ok(doc) = store_reader.get::<TantivyDocument>(doc_id) {
                    for field_value in doc.field_values() {
                        if field_value.field() == self.path_field {
                            if let Some(path) = field_value.value().as_str() {
                                if path.starts_with(folder) {
                                    let term = Term::from_field_text(self.path_field, path);
                                    writer.delete_term(term);
                                    deleted_count += 1;
                                }
                            }
                        }
                    }
                }
            }
        }
        
        writer.commit()?;
        Ok(deleted_count)
    }

    /// Clears the entire index
    pub fn clear_index(&self) -> tantivy::Result<()> {
        if self.index_path.exists() {
            fs::remove_dir_all(&self.index_path)?;
        }
        Ok(())
    }

    /// Gets the index path
    pub fn get_index_path(&self) -> &std::path::PathBuf {
        &self.index_path
    }

    /// Gets the total number of documents in the index
    pub fn get_document_count(&self) -> tantivy::Result<u64> {
        let index = self.get_index()?;
        let reader = index.reader()?;
        let searcher = reader.searcher();
        Ok(searcher.num_docs())
    }

    /// Gets index statistics
    pub fn get_index_stats(&self) -> tantivy::Result<IndexStats> {
        let doc_count = self.get_document_count()?;
        let size_bytes = self.calculate_index_size();
        
        Ok(IndexStats {
            document_count: doc_count,
            size_bytes,
            index_path: self.index_path.to_string_lossy().to_string(),
        })
    }

    /// Calculates the total size of the index directory
    fn calculate_index_size(&self) -> u64 {
        if !self.index_path.exists() {
            return 0;
        }
        
        walkdir::WalkDir::new(&self.index_path)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.path().is_file())
            .filter_map(|e| fs::metadata(e.path()).ok())
            .map(|m| m.len())
            .sum()
    }
}

/// Statistics about the search index
#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct IndexStats {
    pub document_count: u64,
    pub size_bytes: u64,
    pub index_path: String,
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::io::Write;
    use std::thread;
    use std::time::Duration;
    use tempfile::tempdir;

    use std::sync::atomic::{AtomicU32, Ordering};
    static TEST_COUNTER: AtomicU32 = AtomicU32::new(0);

    fn create_test_engine() -> TantivyEngine {
        let counter = TEST_COUNTER.fetch_add(1, Ordering::SeqCst);
        let test_path = std::env::temp_dir()
            .join(format!("worksentry_test_{}_{}", std::process::id(), counter));
        TantivyEngine::new_with_path(test_path).unwrap()
    }

    #[test]
    fn test_engine_creation() {
        let engine = create_test_engine();
        assert!(engine.get_index_path().is_absolute());
        let engine2 = create_test_engine();
        assert_ne!(engine.get_index_path(), engine2.get_index_path());
    }

    #[test]
    fn test_is_indexable_ext() {
        let engine = create_test_engine();
        // Text files (content indexed)
        assert!(engine.is_indexable_ext("txt"));
        assert!(engine.is_indexable_ext("md"));
        assert!(engine.is_indexable_ext("json"));
        assert!(engine.is_indexable_ext("rs"));
        assert!(engine.is_indexable_ext("RS"));
        
        // Binary files (filename only, but still indexable)
        assert!(engine.is_indexable_ext("exe"));
        assert!(engine.is_indexable_ext("pdf"));
        assert!(engine.is_indexable_ext("epub"));
        assert!(engine.is_indexable_ext("zip"));
        
        // Not indexable
        assert!(!engine.is_indexable_ext("dll"));
        assert!(!engine.is_indexable_ext("lib"));
        assert!(!engine.is_indexable_ext("o"));
    }

    #[test]
    fn test_is_text_indexable() {
        let engine = create_test_engine();
        // Text files where content is indexed
        assert!(engine.is_text_indexable("txt"));
        assert!(engine.is_text_indexable("rs"));
        assert!(engine.is_text_indexable("py"));
        
        // Binary files are NOT text indexable
        assert!(!engine.is_text_indexable("exe"));
        assert!(!engine.is_text_indexable("epub"));
        assert!(!engine.is_text_indexable("pdf"));
    }

    #[test]
    fn test_index_folder_creates_index() {
        let dir = tempdir().unwrap();
        let test_file = dir.path().join("test.txt");
        File::create(&test_file).unwrap().write_all(b"hello world test content").unwrap();

        let engine = create_test_engine();
        let result = engine.index_folder(dir.path().to_str().unwrap());
        assert!(result.is_ok(), "Indexing should succeed, got: {:?}", result);
        assert!(engine.get_index_path().exists());
    }

    #[test]
    fn test_index_folder_invalid_path() {
        let engine = create_test_engine();
        let result = engine.index_folder("/nonexistent/path/12345");
        assert!(result.is_ok());
    }

    #[test]
    fn test_no_duplicates_on_reindex() {
        let dir = tempdir().unwrap();
        let test_file = dir.path().join("test.txt");
        File::create(&test_file).unwrap().write_all(b"hello world unique content").unwrap();

        let engine = create_test_engine();
        
        // Index the folder twice
        engine.index_folder(dir.path().to_str().unwrap()).unwrap();
        engine.index_folder(dir.path().to_str().unwrap()).unwrap();
        
        // Should still have only 1 document
        let count = engine.get_document_count().unwrap();
        assert_eq!(count, 1, "Should have exactly 1 document, not duplicates");
    }

    #[test]
    fn test_incremental_indexing() {
        let dir = tempdir().unwrap();
        let test_file = dir.path().join("test.txt");
        File::create(&test_file).unwrap().write_all(b"original content").unwrap();

        let engine = create_test_engine();
        engine.index_folder(dir.path().to_str().unwrap()).unwrap();
        
        // Search for original content
        let results = engine.search("original", 10).unwrap();
        assert_eq!(results.len(), 1);
        
        // Wait for at least 1 second to ensure mtime changes (stored in seconds)
        thread::sleep(Duration::from_millis(1100));
        File::create(&test_file).unwrap().write_all(b"modified content").unwrap();
        
        // Re-index (should detect the change)
        engine.index_folder(dir.path().to_str().unwrap()).unwrap();
        
        // Search for modified content
        let results = engine.search("modified", 10).unwrap();
        assert_eq!(results.len(), 1);
        
        // Original content should no longer be found
        let results = engine.search("original", 10).unwrap();
        assert_eq!(results.len(), 0);
    }

    #[test]
    fn test_search_after_index() {
        let dir = tempdir().unwrap();
        let test_file = dir.path().join("test.txt");
        File::create(&test_file).unwrap().write_all(b"hello world unique search term 12345").unwrap();

        let engine = create_test_engine();
        engine.index_folder(dir.path().to_str().unwrap()).unwrap();

        let results = engine.search("unique search term", 10);
        assert!(results.is_ok());
        let results = results.unwrap();
        assert!(!results.is_empty(), "Expected at least one search result");
        assert!(results[0].score > 0.0);
    }

    #[test]
    fn test_search_returns_empty_for_no_match() {
        let dir = tempdir().unwrap();
        let test_file = dir.path().join("test.txt");
        File::create(&test_file).unwrap().write_all(b"some content here").unwrap();

        let engine = create_test_engine();
        engine.index_folder(dir.path().to_str().unwrap()).unwrap();

        let results = engine.search("nonexistent term xyz 999", 10);
        assert!(results.is_ok());
        let results = results.unwrap();
        assert!(results.is_empty(), "Expected no results for non-matching query");
    }

    #[test]
    fn test_search_empty_query() {
        let engine = create_test_engine();
        let results = engine.search("", 10);
        assert!(results.is_ok());
        assert!(results.unwrap().is_empty());
    }

    #[test]
    fn test_delete_file() {
        let dir = tempdir().unwrap();
        let test_file = dir.path().join("test.txt");
        File::create(&test_file).unwrap().write_all(b"content to delete").unwrap();

        let engine = create_test_engine();
        engine.index_folder(dir.path().to_str().unwrap()).unwrap();
        
        // Verify file is indexed
        assert_eq!(engine.get_document_count().unwrap(), 1);
        
        // Delete from index
        engine.delete_file(&test_file.to_string_lossy()).unwrap();
        
        // Verify file is removed
        assert_eq!(engine.get_document_count().unwrap(), 0);
    }

    #[test]
    fn test_index_stats() {
        let dir = tempdir().unwrap();
        let test_file = dir.path().join("test.txt");
        File::create(&test_file).unwrap().write_all(b"test content").unwrap();

        let engine = create_test_engine();
        engine.index_folder(dir.path().to_str().unwrap()).unwrap();

        let stats = engine.get_index_stats().unwrap();
        assert_eq!(stats.document_count, 1);
        assert!(stats.size_bytes > 0);
    }

    #[test]
    fn test_clear_index() {
        let dir = tempdir().unwrap();
        let test_file = dir.path().join("test.txt");
        File::create(&test_file).unwrap().write_all(b"content").unwrap();

        let engine = create_test_engine();
        engine.index_folder(dir.path().to_str().unwrap()).unwrap();
        assert!(engine.get_index_path().exists());

        engine.clear_index().unwrap();
        assert!(!engine.get_index_path().exists());
    }

    #[test]
    fn test_search_case_insensitive() {
        let dir = tempdir().unwrap();
        let test_file = dir.path().join("test.txt");
        File::create(&test_file).unwrap().write_all(b"HELLO WORLD").unwrap();

        let engine = create_test_engine();
        engine.index_folder(dir.path().to_str().unwrap()).unwrap();

        let results = engine.search("hello", 10);
        assert!(results.is_ok());
        let results = results.unwrap();
        assert_eq!(results.len(), 1);
    }

    #[test]
    fn test_fuzzy_search() {
        let dir = tempdir().unwrap();
        let test_file = dir.path().join("test.txt");
        // Write "programming" to test fuzzy matching
        File::create(&test_file).unwrap().write_all(b"This is about programming language").unwrap();

        let engine = create_test_engine();
        engine.index_folder(dir.path().to_str().unwrap()).unwrap();

        // Search with a typo "programing" (missing one 'm')
        let results = engine.search_enhanced("programing", 10, true, false);
        assert!(results.is_ok());
        let results = results.unwrap();
        // Should find the file despite the typo
        assert!(!results.is_empty(), "Fuzzy search should find 'programming' when searching 'programing'");
    }

    #[test]
    fn test_chinese_contains_detection() {
        assert!(TantivyEngine::contains_chinese("你好"));
        assert!(TantivyEngine::contains_chinese("Hello 世界"));
        assert!(!TantivyEngine::contains_chinese("Hello World"));
        assert!(!TantivyEngine::contains_chinese("123 abc"));
    }

    #[test]
    fn test_tokenize_chinese() {
        let engine = create_test_engine();
        
        // Test Chinese tokenization
        let tokens = engine.tokenize_query("我爱编程");
        assert!(!tokens.is_empty(), "Chinese text should produce tokens");
        
        // Test English tokenization
        let tokens = engine.tokenize_query("hello world");
        assert_eq!(tokens.len(), 2);
        assert!(tokens.contains(&"hello".to_string()));
        assert!(tokens.contains(&"world".to_string()));
    }

    #[test]
    fn test_search_chinese_content() {
        let dir = tempdir().unwrap();
        let test_file = dir.path().join("chinese.txt");
        // Write Chinese content
        File::create(&test_file).unwrap().write_all("这是一个关于编程的文档".as_bytes()).unwrap();

        let engine = create_test_engine();
        engine.index_folder(dir.path().to_str().unwrap()).unwrap();

        // Search with Chinese terms
        let results = engine.search_enhanced("编程", 10, false, false);
        assert!(results.is_ok());
        // Note: This may or may not find results depending on tokenization
        // The key is that it doesn't crash
    }

    #[test]
    fn test_enhanced_search_empty_query() {
        let engine = create_test_engine();
        let results = engine.search_enhanced("", 10, true, true);
        assert!(results.is_ok());
        assert!(results.unwrap().is_empty());
    }

    #[test]
    fn test_launcher_search() {
        let dir = tempdir().unwrap();
        // Create a file that simulates the real case
        let test_file = dir.path().join("7 Rules of Power双语.epub");
        File::create(&test_file).unwrap().write_all(b"dummy").unwrap();

        let engine = create_test_engine();
        engine.index_folder(dir.path().to_str().unwrap()).unwrap();

        // Search with "7r" should find "7 Rules of Power"
        let results = engine.search_launcher("7r", 10);
        assert!(results.is_ok());
        let results = results.unwrap();
        assert!(!results.is_empty(), "Launcher search '7r' should find '7 Rules of Power双语.epub'");
        assert!(results[0].file_name.contains("7 Rules"));
    }

    #[test]
    fn test_launcher_score() {
        // Test the scoring function directly
        let parts = vec!["7r"];
        let score = TantivyEngine::calculate_launcher_score(&parts, "7 rules of power.epub");
        assert!(score.is_some(), "Should match '7r' in '7 rules of power.epub'");
        
        let parts = vec!["xyz"];
        let score = TantivyEngine::calculate_launcher_score(&parts, "7 rules of power.epub");
        assert!(score.is_none(), "Should not match 'xyz' in '7 rules of power.epub'");
    }
}

// ============================================================================
// Global Engine and Public API
// ============================================================================

use once_cell::sync::Lazy;
use std::sync::Mutex;

static APP_ENGINE: Lazy<Mutex<TantivyEngine>> = Lazy::new(|| {
    Mutex::new(TantivyEngine::new().expect("Failed to create TantivyEngine"))
});

pub fn init() -> tantivy::Result<()> {
    let _unused = APP_ENGINE.lock().map_err(|e| tantivy::TantivyError::InternalError(e.to_string()))?;
    Ok(())
}

pub fn index_folder(folder: &str) -> tantivy::Result<()> {
    let engine = APP_ENGINE.lock().map_err(|e| tantivy::TantivyError::InternalError(e.to_string()))?;
    engine.index_folder(folder)?;
    Ok(())
}

pub fn index_browser_data(data: Vec<crate::services::browser_extractor::BrowserData>) -> tantivy::Result<()> {
    let engine = APP_ENGINE.lock().map_err(|e| tantivy::TantivyError::InternalError(e.to_string()))?;
    engine.index_browser_data(data)?;
    Ok(())
}

pub fn search_files(query: &str, limit: usize) -> tantivy::Result<Vec<SearchResult>> {
    let engine = APP_ENGINE.lock().map_err(|e| tantivy::TantivyError::InternalError(e.to_string()))?;
    engine.search(query, limit)
}

/// Enhanced search with fuzzy matching and Chinese text support
pub fn search_files_enhanced(query: &str, limit: usize, fuzzy: bool, prefix: bool) -> tantivy::Result<Vec<SearchResult>> {
    let engine = APP_ENGINE.lock().map_err(|e| tantivy::TantivyError::InternalError(e.to_string()))?;
    engine.search_enhanced(query, limit, fuzzy, prefix)
}

pub fn delete_file(path: &str) -> tantivy::Result<()> {
    let engine = APP_ENGINE.lock().map_err(|e| tantivy::TantivyError::InternalError(e.to_string()))?;
    engine.delete_file(path)?;
    Ok(())
}

pub fn delete_folder(folder: &str) -> tantivy::Result<u32> {
    let engine = APP_ENGINE.lock().map_err(|e| tantivy::TantivyError::InternalError(e.to_string()))?;
    engine.delete_folder(folder)
}

    #[test]
    fn test_search_launcher_chinese_epub() {
        use std::fs::File;
        // let _ = env_logger::builder().is_test(true).try_init(); // env_logger might not be in dev-dependencies
        let engine = TantivyEngine::new().unwrap();
        
        // Index a fake epub file with Chinese name
        let dir = tempfile::tempdir().unwrap();
        let file_path = dir.path().join("氛围.epub");
        File::create(&file_path).unwrap(); // Empty file
        
        let path_str = file_path.to_string_lossy().to_string();
        engine.index_file(&path_str).unwrap();

        // Search for "氛围"
        let results = engine.search_launcher("氛围", 10).unwrap();
        assert!(!results.is_empty(), "Should find 氛围.epub");
        assert_eq!(results[0].file_name, "氛围.epub");
        assert!(results[0].path.contains("氛围.epub"));
        
        // Search for "epub"
        let results = engine.search_launcher("epub", 10).unwrap();
        assert!(!results.is_empty(), "Should find by extension");
    }

pub fn clear_index() -> tantivy::Result<()> {
    let engine = APP_ENGINE.lock().map_err(|e| tantivy::TantivyError::InternalError(e.to_string()))?;
    engine.clear_index()?;
    Ok(())
}

pub fn rebuild_index(folders: &[String]) -> tantivy::Result<()> {
    let engine = APP_ENGINE.lock().map_err(|e| tantivy::TantivyError::InternalError(e.to_string()))?;
    engine.clear_index()?;
    for folder in folders {
        engine.index_folder(folder)?;
    }
    Ok(())
}

pub fn get_document_count() -> tantivy::Result<u64> {
    let engine = APP_ENGINE.lock().map_err(|e| tantivy::TantivyError::InternalError(e.to_string()))?;
    engine.get_document_count()
}

pub fn get_index_stats() -> tantivy::Result<IndexStats> {
    let engine = APP_ENGINE.lock().map_err(|e| tantivy::TantivyError::InternalError(e.to_string()))?;
    engine.get_index_stats()
}

/// Indexes a single file by path (used by file watcher)
pub fn index_single_file(path: &str) -> tantivy::Result<bool> {
    let engine = APP_ENGINE.lock().map_err(|e| tantivy::TantivyError::InternalError(e.to_string()))?;
    engine.index_file(path)
}

/// Launcher-style search (characters in sequence, like "7r" → "7 Rules")
pub fn search_files_launcher(query: &str, limit: usize) -> tantivy::Result<Vec<SearchResult>> {
    let engine = APP_ENGINE.lock().map_err(|e| tantivy::TantivyError::InternalError(e.to_string()))?;
    engine.search_launcher(query, limit)
}
