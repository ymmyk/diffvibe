# Milestone 2a: Backend Write

**Status:** Complete
**Parent:** Milestone 2

## Overview
Rust backend for writing files with encoding preservation and backup.

## Tasks

- [x] `write_file` command - write content to path
- [x] Preserve original encoding (UTF-8, Latin-1)
- [x] Create `.bak` backup before overwriting
- [x] Return success/error with message

## Tauri Command

```rust
#[tauri::command]
fn write_file(path: &str, content: &str, encoding: &str) -> Result<(), String>
```

## Test

Write file, verify content and encoding match. Check backup created.
