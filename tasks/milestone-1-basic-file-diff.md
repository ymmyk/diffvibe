# Milestone 1: Basic File Diff (Two-Way)

**Status:** Not Started

## Overview
Core feature: Compare two files side-by-side with syntax highlighting.

## Backend (Rust)

- [ ] Implement file reading with encoding detection
- [ ] Implement Myers diff algorithm (or use `similar` crate)
- [ ] Create Tauri commands:
  - `read_file(path) -> FileContent`
  - `compute_diff(left, right) -> DiffResult`
- [ ] Handle binary file detection
- [ ] Support large files with streaming/chunking

## Frontend (Svelte)

- [ ] Create two-pane editor layout
- [ ] Implement synchronized scrolling between panes
- [ ] Display diff hunks with visual highlighting:
  - Added lines (green)
  - Removed lines (red)
  - Modified lines (yellow/blue)
- [ ] Add line numbers with diff indicators
- [ ] Implement "jump to next/previous change" navigation
- [ ] Add file info header (path, size, encoding, line count)

## UI/UX

- [ ] File picker dialog (open two files)
- [ ] Drag-and-drop file support
- [ ] Recent files list
- [ ] Keyboard shortcuts (Ctrl+O, Ctrl+N, etc.)

## Test Files

Sample files for testing are in `tests/`:
- `tests/example_old.py`
- `tests/example_new.py`
