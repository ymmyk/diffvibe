# Milestone 2a: Backend Write

**Status:** Not Started
**Parent:** Milestone 2

## Overview
Rust backend for writing files with encoding preservation and backup.

## Tasks

- [ ] `write_file` command - write content to path
- [ ] Preserve original encoding (UTF-8, Latin-1)
- [ ] Create `.bak` backup before overwriting
- [ ] Return success/error with message

## Tauri Command

```rust
#[tauri::command]
fn write_file(path: &str, content: &str, encoding: &str) -> Result<(), String>
```

## Test

Write file, verify content and encoding match. Check backup created.
