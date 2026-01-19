# Milestone 3a: Backend Algorithm

**Status:** Not Started
**Parent:** Milestone 3

## Overview
Rust backend for three-way diff with conflict detection.

## Tasks

- [ ] Add `MergeResult` struct with conflict tracking
- [ ] Implement three-way diff algorithm (diff3-style)
- [ ] Detect conflicts vs auto-resolvable changes
- [ ] Add `compute_three_way_diff` Tauri command
- [ ] Unit tests for three-way diff

## Data Structures

```rust
pub struct MergeChunk {
    pub chunk_type: ChunkType,  // Equal, LocalOnly, RemoteOnly, Conflict
    pub base_lines: Vec<String>,
    pub local_lines: Vec<String>,
    pub remote_lines: Vec<String>,
}

pub enum ChunkType {
    Equal,      // Same in all three
    LocalOnly,  // Changed only in local
    RemoteOnly, // Changed only in remote
    Conflict,   // Changed in both (differently)
}

pub struct MergeResult {
    pub chunks: Vec<MergeChunk>,
    pub conflict_count: usize,
    pub auto_merged: String,  // Best-effort merged content
}
```

## Tauri Command

```rust
#[tauri::command]
fn compute_three_way_diff(base: &str, local: &str, remote: &str) -> MergeResult
```

## Algorithm

1. Diff base→local, base→remote
2. Walk both diffs in parallel
3. Classify each region:
   - Both unchanged → Equal
   - Only local changed → LocalOnly (auto-merge)
   - Only remote changed → RemoteOnly (auto-merge)
   - Both changed same way → Equal (lucky)
   - Both changed differently → Conflict

## Test

Use `tests/base.txt`, `tests/local.txt`, `tests/remote.txt` for testing.
