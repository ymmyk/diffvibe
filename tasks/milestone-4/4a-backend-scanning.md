# Milestone 4a: Backend Scanning

**Status:** Complete
**Parent:** Milestone 4

## Overview
Rust backend for recursive directory scanning and comparison.

## Tasks

- [x] Create `CompareEntry` struct (name, rel_path, is_dir, status, sizes, children)
- [x] Create `FileStatus` enum (Identical, Modified, LeftOnly, RightOnly)
- [x] Create `DirectoryCompareResult` struct with entries and stats
- [x] Implement `compare_directories(left, right)` Tauri command
- [x] Use xxhash (xxh3_64) for fast content hashing
- [x] Recursive directory scanning with `scan_dir_entries`
- [x] Build tree structure from flat file list
- [x] Skip hidden files/dirs (starting with `.`)
- [x] Add TypeScript types matching Rust structs
- [x] Add unit test for directory comparison

## Data Structures

```rust
struct DirectoryEntry {
    name: String,
    path: String,
    is_dir: bool,
    size: u64,
    modified: u64, // unix timestamp
    children: Option<Vec<DirectoryEntry>>,
}

enum FileStatus {
    Identical,
    Modified,
    LeftOnly,  // Added in left, missing in right
    RightOnly, // Added in right, missing in left
}

struct CompareEntry {
    name: String,
    path: String,
    is_dir: bool,
    status: FileStatus,
    left_size: Option<u64>,
    right_size: Option<u64>,
    children: Option<Vec<CompareEntry>>,
}
```

## Test

```bash
# Create test directories
mkdir -p /tmp/dir-test/{left,right}
echo "same" > /tmp/dir-test/left/same.txt
echo "same" > /tmp/dir-test/right/same.txt
echo "modified-left" > /tmp/dir-test/left/modified.txt
echo "modified-right" > /tmp/dir-test/right/modified.txt
echo "only-left" > /tmp/dir-test/left/only-left.txt
echo "only-right" > /tmp/dir-test/right/only-right.txt
```
