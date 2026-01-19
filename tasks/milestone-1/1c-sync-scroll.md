# Milestone 1c: Synchronized Scrolling

**Status:** Complete
**Parent:** Milestone 1

## Overview
Sync scroll position between left and right diff panes.

## Tasks

- [x] Track scroll position of each pane
- [x] On scroll, update the other pane to match
- [x] Handle different line heights (if content differs)
- [x] Add toggle to enable/disable sync scroll
- [x] Prevent scroll loops (debounce or flag)

## Implementation Notes

Use `scrollTop` sync or line-based alignment for better UX when files have different lengths.

## Test

Scroll one pane, verify other follows. Toggle sync off, verify independent scroll.
