---
description: Implementation plan for Browser History and Bookmarks integration
---

# Browser Search Integration

## Overview
Expand WorkSentry search capabilities to include Bookmarks and Browsing History from major browsers (Chrome, Edge).

## Dependencies
- `rusqlite`: To read Chrome/Edge History SQLite databases.
- `serde_json`: To parse Bookmarks files.

## Phase 1: Browser Data Extraction API
**File**: `src-tauri/src/services/browser_extractor.rs`

### 1.1 Detection logic
- Identify installation paths for:
  - Google Chrome: `%LOCALAPPDATA%\Google\Chrome\User Data\Default`
  - Microsoft Edge: `%LOCALAPPDATA%\Microsoft\Edge\User Data\Default`

### 1.2 Data Reading
- **History**:
  - Located at `.../History` (SQLite).
  - Query: `SELECT url, title, visit_count, last_visit_time FROM urls`.
  - **Constraint**: File is locked when browser is open. **Solution**: Copy to `%TEMP%` before reading.
- **Bookmarks**:
  - Located at `.../Bookmarks` (JSON).
  - Parse JSON tree to extract name and URL.

## Phase 2: Indexing Schema Updates
**File**: `src-tauri/src/services/tantivy_engine.rs`

### 2.1 Schema Migration
- Add new fields:
  - `record_type`: STRING (STORED) -> "file", "bookmark", "history"
  - `url`: STRING (STORED) -> Actual URL
  - `source`: STRING (STORED) -> "Chrome", "Edge", "FileSystem"

### 2.2 Re-indexing
- Need to handle backward compatibility (older indexes won't have these fields).
- Likely easiest to recommend a full re-index or migration strategy.

## Phase 3: Integration & Configuration
**File**: `src-tauri/src/commands/config.rs`, `src-tauri/src/services/mod.rs`

- **Configuration**:
  - Add `enable_browser_search` (bool) to `Config` struct.
  - Add `detected_browsers` (Vec<String>) to return to UI.
- **Commands**:
  - `get_browser_status()` -> Returns { installed: ["Chrome"], enabled: bool }
  - `set_browser_enabled(bool)` -> Updates config and triggers index/purge.

## Phase 4: UI Updates
**File**: `src/components/Settings.tsx`, `src/components/ResultsList.tsx`

- **Settings Panel**:
  - Add "Browser Integration" section.
  - Show status: "Chrome detected", "Edge detected".
  - Toggle switch: "Include Browsing History".
- **Search Results**:
  - Update `SearchResult` interface.
  - Differentiate UI for Files vs URLs.
    - Files: Show path, open file.
    - URLs: Show URL, open in default browser.

## Risks & Mitigations
- **Privacy**: User might not want history indexed. -> **Action**: Make it opt-in or configurable in Settings.
- **Performance**: History can be huge (10k+ URLs). -> **Action**: Limit to top 1000 most visited or last 30 days.

## Step-by-Step Implementation

1. [ ] Add `rusqlite` to Cargo.toml.
2. [ ] Create `browser_extractor.rs` service.
3. [ ] Update Tantivy schema.
4. [ ] Implement indexing logic.
5. [ ] Update Frontend to render URL results.
