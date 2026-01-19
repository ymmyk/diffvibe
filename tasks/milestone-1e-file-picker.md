# Milestone 1e: File Picker & Open Dialog

**Status:** Complete
**Parent:** Milestone 1

## Overview
UI to select two files for comparison.

## Tasks

- [ ] Add "Open Left" and "Open Right" buttons
- [ ] Use Tauri dialog plugin for native file picker
- [ ] Show selected file paths in header
- [ ] Auto-compute diff when both files selected
- [ ] Add "Swap" button to swap left/right
- [ ] Keyboard: `Ctrl+O` open dialog

## Implementation

```typescript
import { open } from '@tauri-apps/plugin-dialog';

const file = await open({
  multiple: false,
  filters: [{ name: 'All Files', extensions: ['*'] }]
});
```

## Test

Open two test files via dialog, verify diff displays.
