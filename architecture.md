# WorkSentry Architecture

## Overview

WorkSentry is a unified workstation entry point built with Tauri, providing quick access to files and applications via Alt+Space hotkey, powered by Tantivy full-text search.

## Tech Stack

- **Frontend**: React 18 + TypeScript + Vite
- **Backend**: Rust + Tauri 2.0
- **Search Engine**: Tantivy (Rust-based full-text search)
- **Text Processing**: jieba-rs (Chinese tokenization)
- **Platform**: Windows 10/11

## Architecture Diagram

```
┌─────────────────────────────────────────────────────────────────┐
│                        WorkSentry App                           │
├─────────────────────────────────────────────────────────────────┤
│  ┌─────────────────┐    ┌─────────────────────────────────┐    │
│  │   React UI      │    │      Tauri Backend              │    │
│  │  ┌───────────┐  │    │  ┌───────────────────────────┐  │    │
│  │  │ SearchBar │◄─┼────┼──│  Command Handler          │  │    │
│  │  └───────────┘  │    │  │  - Parse commands         │  │    │
│  │  ┌───────────┐  │    │  │  - Route to handlers      │  │    │
│  │  │ResultsList│◄─┼────┼──│  └─────────────────────────┘  │    │
│  │  └───────────┘  │    │  ┌───────────────────────────┐  │    │
│  │  ┌───────────┐  │    │  │  Search Engine (Tantivy)  │  │    │
│  │  │  Settings │  │    │  │  - Index management       │  │    │
│  │  └───────────┘  │    │  │  - Full-text search       │  │    │
│  └─────────────────┘    │  │  - Incremental updates    │  │    │
│                         │  │  - Fuzzy/prefix matching  │  │    │
│  ┌─────────────────────┐│  └───────────────────────────┘  │    │
│  │   Global Hotkey     ││  ┌───────────────────────────┐  │    │
│  │   (Alt+Space)       ││  │  Config Manager           │  │    │
│  │                     ││  │  - Hotkey binding         │  │    │
│  └─────────────────────┘│  │  - Folder paths           │  │    │
│                         │  │  - Theme preferences      │  │    │
│  ┌─────────────────────┐│  └───────────────────────────┘  │    │
│  │   System Tray       ││  ┌───────────────────────────┐  │    │
│  │   (optional)        ││  │  File Watcher             │  │    │
│  └─────────────────────┘│  │  - Real-time monitoring   │  │    │
│                         │  │  - Debounced auto-reindex │  │    │
│                         │  │  - Create/Modify/Delete   │  │    │
│                         │  └───────────────────────────┘  │    │
│                         └────────────────────────────────┘    │
└─────────────────────────────────────────────────────────────────┘
```

## Directory Structure

```
worksentry/
├── src/                          # React frontend
│   ├── components/
│   │   ├── SearchBar.tsx
│   │   ├── ResultsList.tsx
│   │   ├── Settings.tsx
│   │   └── HotkeyConfig.tsx
│   ├── App.tsx
│   ├── main.tsx
│   ├── index.css
│   └── hooks/
│       ├── useSearch.ts
│       └── useHotkey.ts
├── src-tauri/                    # Rust backend
│   ├── Cargo.toml
│   ├── tauri.conf.json
│   ├── build.rs
│   ├── icons/
│   ├── gen/
│   └── src/
│       ├── main.rs
│       ├── commands/
│       │   ├── mod.rs
│       │   ├── search.rs
│       │   ├── index.rs
│       │   └── config.rs
│       ├── services/
│       │   ├── mod.rs
│       │   ├── tantivy_engine.rs
│       │   ├── file_watcher.rs
│       │   ├── tokenizer.rs       # Custom multilingual tokenizer
│       │   └── hotkey_manager.rs
│       └── utils/
│           ├── mod.rs
│           └── path_utils.rs
├── .agent/
│   └── workflows/
│       └── index-improvements.md  # Implementation plan
├── tests/                        # Unit tests
│   ├── frontend/
│   │   └── *.test.tsx
│   └── backend/
│       └── *.rs
├── package.json
├── vite.config.ts
├── tsconfig.json
├── Cargo.toml
└── README.md
```

## Core Components

### 1. Hotkey Manager
- Registers global hotkey (Alt+Space by default)
- Configurable via settings
- Shows/hides the app window

### 2. Search Engine (Tantivy)
- Indexes files in user-configured folders
- Supports common file types: txt, md, json, code files
- **Persistent index** stored at `%APPDATA%/worksentry/index`
- **Incremental indexing** - only updates changed files
- **Duplicate prevention** - deletes old entry before re-adding
- **Fuzzy/prefix search** - finds matches even with typos
- **Multilingual support** - handles English and Chinese text

### 3. Config Manager
- Persists settings to JSON at `%APPDATA%/worksentry/config.json`
- User-defined folders to index
- Hotkey configuration
- Theme preferences

### 4. File Watcher
- Watches indexed folders for changes in real-time
- **Auto-updates index** on file add/modify/delete
- **Debounced updates** (300ms) to avoid performance issues
- Handles rapid file changes gracefully

## IPC Interface

### Frontend → Backend Commands

| Command | Parameters | Description |
|---------|------------|-------------|
| `search` | `{ query: string, limit: number, options?: SearchOptions }` | Search indexed files |
| `addFolder` | `{ path: string }` | Add folder to index |
| `removeFolder` | `{ path: string }` | Remove folder from index |
| `getFolders` | `()` | List configured folders |
| `reindex` | `()` | Trigger full reindex |
| `getIndexStats` | `()` | Get index statistics |
| `getHotkey` | `()` | Current hotkey config |
| `setHotkey` | `{ modifiers: string[], key: string }` | Set hotkey |
| `getConfig` | `()` | Get all settings |
| `saveConfig` | `config: object` | Save settings |

## Data Models

```rust
// File metadata stored in index
struct FileDoc {
    path: String,           // STRING | STORED - unique identifier
    file_name: String,      // TEXT | STORED - searchable filename
    content: String,        // TEXT - searchable content (not stored)
    extension: String,      // STRING | STORED - file extension
    size: u64,              // STORED - file size in bytes
    modified_time: i64,     // STORED - Unix timestamp for change detection
}

// Search result
struct SearchResult {
    path: String,
    file_name: String,
    score: f32,
    extension: String,
    size: u64,
    modified_time: i64,
    highlights: Vec<String>,  // Matched snippets with highlighting
}

// Search options
struct SearchOptions {
    fuzzy: bool,           // Enable fuzzy matching for typos
    prefix: bool,          // Enable prefix matching
    extensions: Vec<String>, // Filter by file extensions
}

// Config
struct Config {
    indexed_folders: Vec<String>,
    hotkey: HotkeyConfig,
    theme: Theme,
    max_results: u32,
}

struct HotkeyConfig {
    modifiers: Vec<String>,
    key: String,
}

// Index statistics
struct IndexStats {
    document_count: u64,
    size_bytes: u64,
    last_indexed: Option<i64>,
    watched_folders: Vec<String>,
}
```

## Indexing Strategy

### Schema Design
```
┌─────────────────────────────────────────────────────────┐
│                    Tantivy Schema                        │
├─────────────────────────────────────────────────────────┤
│ path          │ STRING | STORED    │ Unique key         │
│ file_name     │ TEXT | STORED      │ Tokenized, stored  │
│ content       │ TEXT               │ Tokenized only     │
│ extension     │ STRING | STORED    │ For filtering      │
│ size          │ u64 | STORED       │ File size          │
│ modified_time │ i64 | STORED       │ Change detection   │
└─────────────────────────────────────────────────────────┘
```

### Indexing Flow
```
┌──────────────┐     ┌───────────────┐     ┌────────────────┐
│  Walk Folder │────►│ Check if New  │────►│ Index/Update   │
└──────────────┘     │  or Modified  │     │   Document     │
                     └───────────────┘     └────────────────┘
                            │
                            ▼ (unchanged)
                     ┌───────────────┐
                     │     Skip      │
                     └───────────────┘
```

### File Watcher Flow
```
┌──────────────┐     ┌───────────────┐     ┌────────────────┐
│  FS Event    │────►│   Debounce    │────►│ Process Event  │
│ (300ms wait) │     │   (300ms)     │     │                │
└──────────────┘     └───────────────┘     └────────────────┘
                                                   │
                     ┌─────────────────────────────┴─────────────────────────────┐
                     │                             │                             │
                     ▼                             ▼                             ▼
              ┌────────────┐               ┌────────────┐               ┌────────────┐
              │  Created   │               │  Modified  │               │  Deleted   │
              │ index_file │               │ index_file │               │ delete_doc │
              └────────────┘               └────────────┘               └────────────┘
```

### Tokenization Strategy
```
┌─────────────────────────────────────────────────────────┐
│              Multilingual Tokenizer                      │
├─────────────────────────────────────────────────────────┤
│  Input Text                                              │
│      │                                                   │
│      ▼                                                   │
│  ┌────────────────────────┐                              │
│  │  Detect Language       │                              │
│  └────────────────────────┘                              │
│      │                                                   │
│      ├── English ──► SimpleTokenizer (whitespace/punct)  │
│      │                                                   │
│      └── Chinese ──► Jieba Tokenizer (word segmentation) │
│                                                          │
└─────────────────────────────────────────────────────────┘
```

## Supported File Types

| Category | Extensions |
|----------|------------|
| Text | `.txt`, `.md`, `.log` |
| Data | `.json`, `.yaml`, `.yml`, `.toml`, `.xml`, `.csv`, `.ini`, `.conf` |
| Code | `.rs`, `.py`, `.js`, `.ts`, `.tsx`, `.html`, `.css` |

## MVP Features

1. **Global Hotkey Activation**
   - Alt+Space shows the search window
   - Escape closes the window

2. **File Search**
   - Full-text search across indexed folders
   - Real-time results as you type
   - Keyboard navigation (Arrow keys, Enter to open)
   - Fuzzy matching for typo tolerance
   - Prefix matching for partial queries

3. **Folder Management**
   - Add/remove folders to index
   - Automatic initial indexing
   - Real-time file watching

4. **Settings Panel**
   - Configure hotkey
   - Manage indexed folders
   - View index statistics

## Performance Considerations

- **Index Location**: Persistent storage at `%APPDATA%/worksentry/index`
- **Memory Limit**: Writer buffer capped at 50MB
- **Large Files**: Skip files > 1MB (configurable)
- **Debouncing**: File watcher events debounced at 300ms
- **Batch Commits**: Group document updates before committing

## Future Enhancements

- Application launcher (run commands)
- Calculator integration
- System commands (shutdown, lock, etc.)
- Web search shortcuts
- Plugin system
- Cloud sync
- PDF/Office document indexing
- Image OCR support
