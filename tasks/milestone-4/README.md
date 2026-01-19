# Milestone 4: Directory Comparison

**Status:** Not Started

## Overview
Compare two directories recursively.

## Backend

- [ ] Recursive directory scanning
- [ ] File comparison (content hash or quick byte comparison)
- [ ] Tauri commands:
  - `scan_directory(path) -> DirectoryTree`
  - `compare_directories(left, right) -> DirectoryDiff`
- [ ] Handle symlinks appropriately
- [ ] Ignore patterns (.gitignore, custom patterns)

## Frontend

- [ ] Tree view for directory structure
- [ ] Status icons for each file/folder:
  - Identical (checkmark)
  - Modified (dot)
  - Added (plus)
  - Removed (minus)
  - Conflict (exclamation)
- [ ] Filter by status (show only changed, etc.)
- [ ] Double-click to open file diff
- [ ] Expand/collapse directories
- [ ] "Copy to left/right" for files and folders

## Features

- [ ] Directory picker dialog
- [ ] Recursive diff statistics
- [ ] Ignore binary files option
- [ ] File size and date comparison mode
