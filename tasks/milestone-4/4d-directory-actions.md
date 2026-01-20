# Milestone 4d: Directory Actions

**Status:** Not Started
**Parent:** Milestone 4

## Overview
Actions and filters for directory comparison.

## Tasks

### Copy Actions
- [ ] "Copy to right" button/context menu for left-only files
- [ ] "Copy to left" button/context menu for right-only files
- [ ] Copy entire folder (recursive)
- [ ] Confirm dialog for overwrites
- [ ] Show progress for large copies

### Filters
- [ ] Filter dropdown: All, Changed only, Identical only
- [ ] Quick filter buttons in toolbar
- [ ] Remember filter preference

### Ignore Patterns
- [ ] Load .gitignore from directories if present
- [ ] Custom ignore patterns in settings
- [ ] Common defaults: node_modules, .git, __pycache__, .DS_Store
- [ ] Toggle "Show ignored files" (greyed out)

### Additional Features
- [ ] Refresh button to rescan directories
- [ ] "Expand all" / "Collapse all" buttons
- [ ] File count summary in header (X identical, Y modified, Z added, W removed)
- [ ] Right-click context menu with actions

## Ignore Pattern Format

Use gitignore-style patterns:
```
node_modules/
*.pyc
.DS_Store
.git/
__pycache__/
```
