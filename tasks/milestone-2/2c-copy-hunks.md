# Milestone 2c: Copy Hunks

**Status:** Complete
**Parent:** Milestone 2

## Overview
Add buttons to copy diff hunks between panes.

## Tasks

- [x] Add "→" button on left hunk to copy to right
- [x] Add "←" button on right hunk to copy to left
- [x] Position buttons in gutter between panes
- [x] Handle multi-line hunks (insert/replace lines)
- [x] "Copy all →" and "← Copy all" bulk buttons in toolbar
- [x] Re-compute diff after copy

## UI

```
[left pane] [→] [gutter] [←] [right pane]
```

Buttons appear on hover over hunk, or always visible in gutter.

## Test

Copy hunk, verify content matches. Check diff re-computes correctly.
