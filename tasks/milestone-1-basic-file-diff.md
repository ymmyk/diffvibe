# Milestone 1: Basic File Diff (Two-Way)

**Status:** Not Started

## Overview
Core feature: Compare two files side-by-side with diff highlighting.

## Subtasks

| # | Subtask | Description | Status |
|---|---------|-------------|--------|
| 1a | [Backend Core](milestone-1a-backend-core.md) | File reading, encoding, diff compute | ✅ Complete |
| 1b | [Diff View](milestone-1b-diff-view.md) | Two-pane layout with highlighting | Not Started |
| 1c | [Sync Scroll](milestone-1c-sync-scroll.md) | Synchronized scrolling between panes | Not Started |
| 1d | [Navigation](milestone-1d-navigation.md) | Jump to next/prev change | Not Started |
| 1e | [File Picker](milestone-1e-file-picker.md) | Open dialog for file selection | ✅ Complete |
| 1f | [Drag & Drop](milestone-1f-drag-drop.md) | Drop files to load | Not Started |
| 1g | [Recent Files](milestone-1g-recent-files.md) | Track recently opened files | Not Started |

## Recommended Order

1. **1a Backend Core** - Foundation for everything
2. **1e File Picker** - Need way to select files
3. **1b Diff View** - Core visual component
4. **1c Sync Scroll** - Essential UX
5. **1d Navigation** - Quality of life
6. **1f Drag & Drop** - Nice to have
7. **1g Recent Files** - Nice to have

## Test Files

Sample files for testing in `tests/`:
- `tests/example_old.py`
- `tests/example_new.py`
