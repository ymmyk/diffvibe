# Milestone 3c: Conflict Resolution

**Status:** Not Started
**Parent:** Milestone 3

## Overview
UI for viewing and resolving merge conflicts.

## Tasks

- [ ] Add merged result pane (4th pane or overlay)
- [ ] Display conflict markers in merged pane
- [ ] Resolution buttons per conflict:
  - Use local (left)
  - Use remote (right)
  - Use both (local then remote)
  - Edit manually
- [ ] Conflict navigation (next/prev conflict)
- [ ] Conflict counter in status bar
- [ ] Track resolved vs unresolved conflicts

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
