# Milestone 4c: Diff Integration

**Status:** Not Started
**Parent:** Milestone 4

## Overview
Connect directory tree to file diff views.

## Tasks

- [ ] Double-click file opens new tab with file diff
- [ ] Pass left/right paths to existing DiffView component
- [ ] Handle "left only" files (show as full insert)
- [ ] Handle "right only" files (show as full delete)
- [ ] Show diff statistics in tree (lines added/removed per file)
- [ ] Add breadcrumb or back navigation from file diff to directory
- [ ] Highlight currently selected file in tree

## Behavior

1. User double-clicks a modified file in tree
2. New tab opens with two-way diff (left vs right version)
3. Tab title shows filename
4. User can switch back to directory tab to continue browsing

## Edge Cases

- Binary files: Show "Binary files differ" message
- Large files: Warn before loading diff
- Identical files: Still allow opening to verify
