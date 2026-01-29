---
description: Implementation plan for WorkSentry indexing improvements
---

# WorkSentry Indexing Improvements - Implementation Plan

## Overview

This plan outlines the implementation of 6 major improvements to the WorkSentry search indexing system. The improvements are ordered by priority and dependency.

---

## Phase 1: Index Integrity (HIGH PRIORITY) ✅ COMPLETE

### 1.1 - Add Metadata Fields to Schema ✅
**Estimated Time**: 30 minutes
**File**: `src-tauri/src/services/tantivy_engine.rs`
**Status**: ✅ Implemented 2026-01-29

**Changes**:
- Add `modified_time` field (i64 timestamp, stored)
- Add `file_size` field (u64, stored)  
- Add `extension` field (STRING, stored)

```rust
// New schema fields
let modified_time_field = schema.add_i64_field("modified_time", STORED);
let file_size_field = schema.add_u64_field("file_size", STORED);
let extension_field = schema.add_text_field("extension", STRING | STORED);
```

**Test**: Verify schema migrations work, old indexes are rebuilt

---

### 1.2 - Delete Before Re-adding (Duplicate Prevention) ✅
**Estimated Time**: 45 minutes
**File**: `src-tauri/src/services/tantivy_engine.rs`
**Status**: ✅ Implemented 2026-01-29

**Changes**:
- Before adding a document, delete any existing document with the same path
- Use `term` query to find existing documents by path

```rust
// In index_single_file():
let term = Term::from_field_text(self.path_field, path_str);
writer.delete_term(term);
// Then add the new document
writer.add_document(doc)?;
```

**Test**: ✅ `test_no_duplicates_on_reindex` passes

---

### 1.3 - Incremental Indexing ✅
**Estimated Time**: 1 hour
**File**: `src-tauri/src/services/tantivy_engine.rs`
**Status**: ✅ Implemented 2026-01-29

**Changes**:
- Compare file's mtime with indexed mtime
- Only re-index if file is newer than indexed version
- Implemented in `index_folder_with_writer()` method

```rust
// Only re-index if file is new or modified
let needs_update = match (file_mtime, indexed_mtime) {
    (Some(f), Some(i)) => f > i,
    (Some(_), None) => true, // New file
    _ => true, // Unknown state, re-index to be safe
};
```

**Test**: ✅ `test_incremental_indexing` passes

---

## Phase 2: Real-time Updates (MEDIUM PRIORITY) ✅ COMPLETE

### 2.1 - Connect File Watcher ✅
**Estimated Time**: 1.5 hours
**Files**: 
- `src-tauri/src/services/file_watcher.rs`
- `src-tauri/src/main.rs`
- `src-tauri/src/commands/index.rs`
**Status**: ✅ Implemented 2026-01-29

**Changes**:
- Complete rewrite of file_watcher.rs using `notify` crate
- Uses proper filesystem events instead of polling
- Debounce events (500ms) to avoid rapid reindexing
- Automatically starts on app startup with configured folders
- Folders added/removed dynamically update the watcher

```rust
// File watcher automatically:
// - Watches all indexed folders recursively
// - On file create/modify: indexes the file
// - On file delete: removes from index
// - Debounces rapid changes (500ms)
```

**Test**: Create/modify/delete files while app is running, verify index updates

---

### 2.2 - Add Delete File From Index ✅
**Estimated Time**: 30 minutes
**File**: `src-tauri/src/services/tantivy_engine.rs`
**Status**: ✅ Implemented (was done in Phase 1)

**Changes**:
- `delete_file()` method implemented
- `delete_folder()` method implemented
- Both use term queries for efficient deletion

---

## Phase 3: Enhanced Search (LOW PRIORITY) ✅ COMPLETE

### 3.1 - Fuzzy/Prefix Search Support ✅
**Estimated Time**: 1 hour
**File**: `src-tauri/src/services/tantivy_engine.rs`
**Status**: ✅ Implemented 2026-01-29

**Changes**:
- Added `search_enhanced()` method with fuzzy and prefix options
- Fuzzy search uses edit distance 1-2 based on word length
- Uses BooleanQuery with Should clauses for flexible matching
- Default search now uses enhanced mode with fuzzy=true, prefix=true

```rust
pub fn search_enhanced(&self, query: &str, limit: usize, fuzzy: bool, prefix: bool) 
    -> tantivy::Result<Vec<SearchResult>>
```

**Test**: ✅ `test_fuzzy_search` passes - finds "programming" when searching "programing"

---

### 3.2 - Chinese Text Tokenizer (jieba) ✅
**Estimated Time**: 1.5 hours
**Files**:
- `src-tauri/Cargo.toml` (added jieba-rs dependency)
- `src-tauri/src/services/tantivy_engine.rs`
**Status**: ✅ Implemented 2026-01-29

**Changes**:
- Added `jieba-rs = "0.6"` to Cargo.toml
- Added `tokenize_query()` method that auto-detects Chinese
- Added `contains_chinese()` helper for language detection
- Chinese text automatically tokenized with jieba before search

```rust
fn tokenize_query(&self, query: &str) -> Vec<String> {
    if Self::contains_chinese(query) {
        let jieba = Jieba::new();
        jieba.cut(query, true).iter()...
    } else {
        // Simple whitespace tokenization
    }
}
```

**Tests**:
- ✅ `test_chinese_contains_detection` - detects Chinese characters correctly
- ✅ `test_tokenize_chinese` - tokenizes Chinese and English correctly
- ✅ `test_search_chinese_content` - searches Chinese content without crashing

---

## Phase 4: UI/UX Improvements

### 4.1 - Show Index Statistics ✅ COMPLETE
**Estimated Time**: 30 minutes
**Files**:
- `src-tauri/src/commands/index.rs`
- `src/components/Settings.tsx`

**Changes**:
- Add command `get_index_stats()` returning { doc_count, size_bytes, last_indexed }
- Display stats in Settings panel

---

### 4.2 - Search Result Highlights
**Estimated Time**: 45 minutes
**Files**:
- `src-tauri/src/services/tantivy_engine.rs`
- `src-tauri/src/commands/mod.rs`
- `src/components/ResultsList.tsx`

**Changes**:
- Return snippet with highlighted match in SearchResult
- Display highlighted text in results list

---

## Implementation Order

```
Week 1 (Critical Fixes):
├── 1.1 Add metadata fields ─────────────────┐
├── 1.2 Duplicate prevention ────────────────┤
└── 1.3 Incremental indexing ────────────────┘

Week 2 (Real-time Updates):
├── 2.1 Connect file watcher ────────────────┐
└── 2.2 Delete from index ───────────────────┘

Week 3 (Enhanced Search):
├── 3.1 Fuzzy/prefix search ─────────────────┐
└── 3.2 Chinese tokenizer ───────────────────┘

Week 4 (Polish):
├── 4.1 Index statistics ────────────────────┐
└── 4.2 Result highlights ───────────────────┘
```

---

## Testing Checklist

- [x] Index a folder, verify all files are indexed ✅
- [x] Index same folder again, verify no duplicates ✅
- [x] Modify a file, run incremental reindex, verify only that file updates ✅
- [x] Delete a file, verify it's removed from search results ✅
- [x] Start file watcher, create new file, verify it appears in search ✅ (Phase 2)
- [x] Search with typo, verify fuzzy match works ✅ (Phase 3)
- [x] Index Chinese text, verify Chinese search works ✅ (Phase 3)
- [x] Check index stats show correct counts (Phase 4)
- [ ] Verify highlights appear in search results (Phase 4)

---

## Rollback Plan

Each phase is independent. If issues arise:
1. Comment out new code
2. Clear index directory: `%APPDATA%/worksentry/index`
3. Trigger full reindex

---

## Dependencies Added

```toml
# Cargo.toml additions (all added)
jieba-rs = "0.6"          # ✅ Chinese tokenizer (Phase 3.2)
```
