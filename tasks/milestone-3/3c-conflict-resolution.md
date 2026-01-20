# Milestone 3c: Conflict Resolution

**Status:** Partial (deferred to Milestone 6)
**Parent:** Milestone 3

## Overview
UI for viewing and resolving merge conflicts.

## Completed

- [x] Three-way diff algorithm (Rust backend)
- [x] Three-pane view with diff highlighting (Local/Base/Remote)
- [x] Synchronized scrolling
- [x] Conflict counter in status bar

## Deferred to Milestone 6

See [milestone-6/README.md](../milestone-6/README.md) for:
- Highlight conflicts directly in 3-pane view (inline resolution buttons)
- Meld-style merged pane in center replacing base pane
- Use both (local then remote) resolution option
- Manual edit option for conflicts
- Keyboard shortcuts for conflict navigation

## Current UI

```
┌─────────┬─────────┬─────────┐
│ LOCAL   │  BASE   │ REMOTE  │
│ (diffs) │         │ (diffs) │
└─────────┴─────────┴─────────┘
Status: Local +X -Y | Remote +X -Y | N conflicts
```
