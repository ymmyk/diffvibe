# Milestone 1c: Synchronized Scrolling

**Status:** Not Started
**Parent:** Milestone 1

## Overview
Sync scroll position between left and right diff panes.

## Tasks

- [ ] Track scroll position of each pane
- [ ] On scroll, update the other pane to match
- [ ] Handle different line heights (if content differs)
- [ ] Add toggle to enable/disable sync scroll
- [ ] Prevent scroll loops (debounce or flag)

## Implementation Notes

Use `scrollTop` sync or line-based alignment for better UX when files have different lengths.

## Test

Scroll one pane, verify other follows. Toggle sync off, verify independent scroll.
