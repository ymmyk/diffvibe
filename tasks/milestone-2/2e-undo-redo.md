# Milestone 2e: Undo/Redo

**Status:** Not Started
**Parent:** Milestone 2

## Overview
History management for edits with undo/redo support.

## Tasks

- [ ] History stack per pane (or per tab)
- [ ] Push state on each edit
- [ ] Ctrl+Z / Cmd+Z to undo
- [ ] Ctrl+Shift+Z / Cmd+Shift+Z to redo
- [ ] Coalesce rapid typing into single undo entry
- [ ] Clear history on file load
- [ ] Undo/redo buttons in toolbar (optional)

## Data Structure

```typescript
interface HistoryEntry {
  content: string;
  cursorPosition?: number;
}

interface History {
  past: HistoryEntry[];
  present: HistoryEntry;
  future: HistoryEntry[];
}
```

## Test

Edit, undo, verify previous state. Redo, verify restored. Check rapid typing coalesced.
