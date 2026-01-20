# Milestone 4b: Tree View UI

**Status:** Complete
**Parent:** Milestone 4

## Overview
Svelte component for displaying directory comparison as a tree.

## Tasks

- [x] Create `DirectoryTree.svelte` component
- [x] Display entries as expandable/collapsible tree
- [x] Show file/folder icons (unicode emojis)
- [x] Indent children properly
- [x] Track expanded/collapsed state per directory
- [x] Keyboard navigation (arrow keys, enter to expand)
- [x] Create `DirectoryCompareView.svelte` (main view for dir tabs)
- [x] Wire up to tab system (new tab type: 'directory')
- [x] "Compare Directories" option already exists on home screen

## Component Structure

```
DirectoryCompareView.svelte
├── Header (left path | right path)
├── Toolbar (refresh, expand all, collapse all)
└── TreeContainer
    └── DirectoryTree.svelte (recursive)
        └── TreeNode (file or folder row)
```

## Tree Node Design

```
▶ folder-name/          [status-icon]
  ├── file.txt          [status-icon]
  ├── subfolder/        [status-icon]
  └── another.js        [status-icon]
```

## Status Icons

- ✓ or green dot: Identical
- ● or yellow dot: Modified
- + or green: Left only (added)
- − or red: Right only (removed)
