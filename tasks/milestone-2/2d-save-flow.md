# Milestone 2d: Save Flow

**Status:** Not Started
**Parent:** Milestone 2

## Overview
Save edited files with proper UX for dirty state and confirmation.

## Tasks

- [ ] Track dirty state (edited vs saved)
- [ ] Show dirty indicator in tab title (• or *)
- [ ] Ctrl+S / Cmd+S to save current pane
- [ ] Save As... dialog (Ctrl+Shift+S)
- [ ] Confirm dialog on tab close with unsaved changes
- [ ] Confirm on app close with any unsaved tabs

## Keyboard Shortcuts

- `Cmd+S` / `Ctrl+S` - Save
- `Cmd+Shift+S` / `Ctrl+Shift+S` - Save As

## UI

- Tab: `file.txt •` (dot indicates unsaved)
- Dialog: "You have unsaved changes. Save before closing?"
  - [Save] [Don't Save] [Cancel]

## Test

Edit file, verify dirty indicator. Save, verify indicator clears. Close with unsaved, verify prompt.
