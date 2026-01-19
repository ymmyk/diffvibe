# Milestone 1f: Drag and Drop

**Status:** ✅ Complete
**Parent:** Milestone 1

## Overview
Drop files onto panes to load them.

## Tasks

- [x] Add drop zone to each pane (left/right/base inputs on HomePage)
- [x] Visual feedback on drag over (dashed border + background)
- [x] Read dropped file path (Tauri provides file.path)
- [x] Support dropping on home page

## Implementation

Used native drag events (ondragover, ondragenter, ondragleave, ondrop).
Tauri provides file paths via `file.path` on dropped files.
Added `drag-over` CSS class for visual feedback.

## Test

Drag test files onto left/right inputs, verify paths populate.
