# Milestone 1a: Backend Core - File Reading & Diff

**Status:** Not Started
**Parent:** Milestone 1

## Overview
Rust backend for reading files and computing diffs.

## Tasks

- [ ] Add `read_file(path) -> FileContent` Tauri command
  - Return content, encoding, size, line count
- [ ] Implement encoding detection (UTF-8, UTF-16, Latin-1)
- [ ] Use `similar` crate for diff (already added)
- [ ] Update `compute_diff` to accept file paths (not just strings)
- [ ] Handle binary file detection (check for null bytes)
- [ ] Add `FileContent` and `DiffResult` types to lib.rs

## Types

```rust
struct FileContent {
    content: String,
    encoding: String,
    size: u64,
    line_count: usize,
    is_binary: bool,
}

struct DiffResult {
    hunks: Vec<DiffHunk>,
    stats: DiffStats,
}
```

## Test

```bash
just dev
# Call read_file and compute_diff from frontend console
```
