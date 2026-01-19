# Milestone 3b: Three-Pane Layout

**Status:** Complete
**Parent:** Milestone 3

## Overview
Frontend UI showing local, base, and remote files side-by-side.

## Tasks

- [x] Create `ThreeWayView.svelte` component
- [x] Create `MergePage.svelte` page component
- [x] Update tab store with `openMerge()` function
- [x] Add 'merge' tab type and routing
- [x] Synchronized scrolling across 3 panes
- [ ] Center/merged result pane (deferred to 3c)

## Components Created

- `src/lib/components/ThreeWayView.svelte` - Three-pane diff view
- `src/lib/components/MergePage.svelte` - Page wrapper, loads files

## Layout

```
┌─────────────┬─────────────┬─────────────┐
│   LOCAL     │    BASE     │   REMOTE    │
│  (yours)    │ (ancestor)  │  (theirs)   │
│             │             │             │
└─────────────┴─────────────┴─────────────┘
```

## Test

Select "Three-Way Merge" mode, pick 3 files, click Compare.
