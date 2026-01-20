# Milestone 3c: Conflict Resolution

**Status:** Complete
**Parent:** Milestone 3

## Overview
UI for viewing and resolving merge conflicts.

## Tasks

- [x] Add merged result pane (4th pane below three-way view)
- [x] Display conflict markers in merged pane
- [x] Resolution buttons per conflict:
  - [x] Use local (left arrow button)
  - [x] Use remote (right arrow button)
  - [ ] Use both (local then remote) - future
  - [ ] Edit manually - future
- [x] Conflict navigation (next/prev conflict with N/P keys)
- [x] Conflict counter in status bar
- [x] Track resolved vs unresolved conflicts

## UI Design

```
┌─────────┬─────────┬─────────┐
│ LOCAL   │  BASE   │ REMOTE  │
├─────────┴─────────┴─────────┤
│         MERGED RESULT       │
│  [Use Local] [Use Remote]   │
│  [Use Both]  [Edit]         │
└─────────────────────────────┘
```

Or Meld-style with merged in center:

```
┌─────────┬─────────┬─────────┐
│ LOCAL   │ MERGED  │ REMOTE  │
└─────────┴─────────┴─────────┘
```

## Keyboard Shortcuts

- `N` / `P` - Next/prev conflict
- `1` - Use local
- `2` - Use remote
- `3` - Use both

## Test

Open three-way merge with conflicts, resolve each, verify merged output.
