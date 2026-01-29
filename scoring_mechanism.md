# Search Scoring Mechanism

This document outlines the "Tiered Hybrid" scoring system used by WorkSentry to rank search results. The goal is to provide a predictive, "interpreter-like" feel where the most relevant results appear at the top, similar to launchers like Alfred or Raycast.

## Philosophy

The scoring algorithm combines a **Base Score** (structural match) with **Semantic Multipliers** (contextual relevance).

`Final Score = Base Score * Extension Multiplier * Depth Penalty * Length Bonus`

---

## 1. Base Score (Structural Match)

Defines how physically similar the filename is to the query.

| Tier | Match Type | Base Score | Description |
| :--- | :--- | :--- | :--- |
| **1** | **Exact Match** | `2000` | Filename is identical to query (e.g., "cmd" -> `cmd.exe`) |
| **2** | **Prefix Match** | `1000` | Filename starts with query (e.g., "vibe" -> `vibe_coding`) |
| **3** | **Word Boundary** | `800` | Query matches start of a word (e.g., "vibe" -> `good_vibes.txt`) |
| **4** | **Contiguous Substring** | `500` | Query appears as a block (e.g., "vib" -> `archived_vibes.pdf`) |
| **5** | **Scattered Match** | `0-100` | Characters appear in order but scattered (e.g., "vibe" -> `Very_Important_Book_Entry`) |

---

## 2. Semantic Multipliers

Adjusts the score based on the *type* and *location* of the file.

### A. Extension Weights (File Type Priority)

| Category | Extensions | Multiplier | Rationale |
| :--- | :--- | :--- | :--- |
| **Applications** | `.exe`, `.lnk`, `.app`, `.bat`, `.cmd` | **1.5x** | User likely wants to run a program. |
| **Folders** | (Directories) | **1.2x** | Navigation is a primary use case. |
| **Documents** | `.pdf`, `.docx`, `.epub`, `.md`, `.txt` | **1.0x** | Standard relevance. |
| **Media** | `.png`, `.jpg`, `.mp4`, `.mp3` | **0.9x** | Less likely to be the primary search target. |
| **Code/System** | `.rs`, `.json`, `.dll`, `.xml`, `.sys` | **0.8x** | Often noise; de-prioritized. |

### B. Path Depth Penalty

Deeper files are less likely to be relevant.

- **Formula**: `Score *= (0.95 ^ depth)`
- **Example**:
    - `C:/Users/User.txt` (Depth 0 relative to root) -> **100% Score**
    - `C:/Users/Projects/Client/Src/Assets/Data/config.json` (Depth 6) -> **~73% Score**

### C. Length Bonus

Shorter filenames are "denser" matches.

- **Formula**: `Score += 100 / sqrt(filename_length)`
- **Effect**: `vibe.txt` (8 chars) scores higher than `vibe_coding_trends_2024_final_v2.txt` (35 chars).

---

## 3. Examples

Query: **"vibe"**

| Filename | Match Type | Base Score | Ext Mult | Depth | Final Score (Approx) |
| :--- | :--- | :--- | :--- | :--- | :--- |
| **`vibe.exe`** | Exact (1) | 2000 | 1.5x | -0% | **3000** |
| **`vibe_coding.epub`** | Prefix (2) | 1000 | 1.0x | -0% | **1000** |
| **`Good Vibes.pdf`** | Word Boundary (3) | 800 | 1.0x | -0% | **800** |
| **`archived_vibes.txt`** | Substring (4) | 500 | 1.0x | -5% | **475** |
| **`very_interesting_biology_entry.rs`** | Scattered (5) | 50 | 0.8x | -10% | **36** |

---

## 4. Implementation Details

- **Case Insensitivity**: All comparisons are case-insensitive.
- **Normalization**: Unicode checks are performed (e.g., treating different whitespace or accents appropriately).
- **Performance**:
    - Base scores are calculated first.
    - Multipliers are applied as float operations using `f32`.
    - Sorting uses `partial_cmp`.

