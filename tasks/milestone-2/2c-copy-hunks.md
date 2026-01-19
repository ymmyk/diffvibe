# Milestone 2c: Copy Hunks

**Status:** Not Started
**Parent:** Milestone 2

## Overview
Add buttons to copy diff hunks between panes.

## Tasks

- [ ] Add "→" button on left hunk to copy to right
- [ ] Add "←" button on right hunk to copy to left
- [ ] Position buttons in gutter between panes
- [ ] Handle multi-line hunks (insert/replace lines)
- [ ] "Copy all →" and "← Copy all" bulk buttons in toolbar
- [ ] Re-compute diff after copy

## UI

```
[left pane] [→] [gutter] [←] [right pane]
```

Buttons appear on hover over hunk, or always visible in gutter.

## Test

Copy hunk, verify content matches. Check diff re-computes correctly.
