# Milestone 4d: Directory Actions

**Status:** Completed
**Parent:** Milestone 4

## Overview
Actions and filters for directory comparison.

## Tasks

### Copy Actions
- [x] "Copy to right" button/context menu for left-only files
- [x] "Copy to left" button/context menu for right-only files
- [x] Copy entire folder (recursive)
- [x] Confirm dialog for overwrites
- [ ] Show progress for large copies

### Filters
- [x] Filter dropdown: All, Changed only, Identical only
- [ ] Quick filter buttons in toolbar
- [ ] Remember filter preference

### Ignore Patterns
- [ ] Load .gitignore from directories if present
- [x] Custom ignore patterns in settings
- [x] Common defaults: node_modules, .git, __pycache__, .DS_Store, build dirs, etc.
- [x] Toggle "Show ignored files" (greyed out)
- [x] Apply ignore patterns during scanning (performance improvement)
- [x] UI performance optimizations for large directories (depth limiting, entry limiting, collapsed by default)

### Additional Features
- [x] Refresh button to rescan directories
- [x] "Expand all" / "Collapse all" buttons
- [x] File count summary in header (X identical, Y modified, Z added, W removed)
- [x] Right-click context menu with actions
- [x] Parallel directory scanning with Rayon (performance improvement)

## Ignore Pattern Format

Use gitignore-style patterns:
```
node_modules/
*.pyc
.DS_Store
.git/
__pycache__/
```
