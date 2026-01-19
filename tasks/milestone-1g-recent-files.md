# Milestone 1g: Recent Files

**Status:** ✅ Complete
**Parent:** Milestone 1

## Overview
Track and display recently opened files.

## Tasks

- [x] Store recent file paths (using Tauri Store plugin)
- [x] Limit to last 10 files
- [x] Show recent files on home page
- [x] Click to re-open file
- [x] Clear recent files option
- [x] Store as pairs (left + right) for quick re-compare

## Implementation

- `src/lib/stores/recent.svelte.ts` - Store using `@tauri-apps/plugin-store`
- Persists to `recent.json` in app data dir
- HomePage displays list with click to re-open
- Individual remove (X) and clear all buttons

## Data Structure

```typescript
interface RecentComparison {
  left: string;
  right: string;
  mode: 'file' | 'directory' | 'merge';
  base?: string;
  timestamp: number;
}
```

## Test

Open files, refresh app, verify recent list shows them.
