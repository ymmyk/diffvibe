# Milestone 1b: Two-Pane Diff View

**Status:** Not Started
**Parent:** Milestone 1

## Overview
Create the core side-by-side diff view component.

## Tasks

- [ ] Create `/compare` route for diff view
- [ ] Create `DiffView.svelte` component with two panes
- [ ] Create `DiffPane.svelte` for single file display
- [ ] Display line numbers
- [ ] Highlight diff lines:
  - Added (green background)
  - Removed (red background)
  - Modified (yellow/blue)
  - Unchanged (no highlight)
- [ ] Add file info header per pane (path, size, lines)
- [ ] Style with CSS variables for theme support

## Components

```
src/lib/components/
├── DiffView.svelte      # Container with two panes
├── DiffPane.svelte      # Single file pane
└── DiffLine.svelte      # Single line with highlighting
```

## Test

Load test files and verify visual diff display.
