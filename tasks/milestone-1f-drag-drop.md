# Milestone 1f: Drag and Drop

**Status:** Not Started
**Parent:** Milestone 1

## Overview
Drop files onto panes to load them.

## Tasks

- [ ] Add drop zone to each pane
- [ ] Visual feedback on drag over (highlight border)
- [ ] Read dropped file path
- [ ] Load file content via Tauri command
- [ ] Support dropping on home page (first file = left)

## Implementation

Use native drag events. Tauri provides file paths for dropped files.

## Test

Drag test files onto panes, verify they load.
