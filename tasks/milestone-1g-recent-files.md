# Milestone 1g: Recent Files

**Status:** Not Started
**Parent:** Milestone 1

## Overview
Track and display recently opened files.

## Tasks

- [ ] Store recent file paths in localStorage
- [ ] Limit to last 10 files
- [ ] Show recent files on home page
- [ ] Click to re-open file
- [ ] Clear recent files option
- [ ] Store as pairs (left + right) for quick re-compare

## Data Structure

```typescript
interface RecentComparison {
  left: string;
  right: string;
  timestamp: number;
}
```

## Test

Open files, refresh app, verify recent list shows them.
