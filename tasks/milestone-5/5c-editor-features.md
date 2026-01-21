# Milestone 5c: Editor Features

**Status:** Not Started
**Parent:** Milestone 5

## Overview
Add essential editor features to improve usability and code viewing.

## Tasks

### Display Options
- [ ] Line wrapping toggle
  - [ ] Add checkbox/button to toolbar
  - [ ] Implement wrap mode in editor
  - [ ] Persist preference
  - [ ] Ensure synchronized scrolling works with wrapping
- [ ] Show/hide whitespace
  - [ ] Display spaces as dots
  - [ ] Display tabs as arrows
  - [ ] Toggle button in toolbar
  - [ ] Persist preference
- [ ] Show/hide line numbers
  - [ ] Toggle button in toolbar
  - [ ] Persist preference
  - [ ] Update diff chunk markers accordingly
- [ ] Font size adjustment
  - [ ] Add zoom in/out buttons or slider
  - [ ] Support Ctrl+Plus/Minus keyboard shortcuts
  - [ ] Persist preference
  - [ ] Range: 8px - 24px
- [ ] Tab size configuration
  - [ ] Settings for 2/4/8 spaces
  - [ ] Apply to display (not file content)
  - [ ] Persist preference

### Navigation Features
- [ ] Find in file (Ctrl+F)
  - [ ] Find dialog/bar
  - [ ] Highlight matches in both panes
  - [ ] Next/Previous navigation
  - [ ] Case sensitive toggle
  - [ ] Regex support (optional)
  - [ ] Match count display
- [ ] Go to line (Ctrl+G)
  - [ ] Input dialog
  - [ ] Jump to line in both panes
  - [ ] Support line:column syntax (optional)

### Keyboard Shortcuts
- [ ] Document all shortcuts
- [ ] Implement shortcuts:
  - [ ] Ctrl+F: Find
  - [ ] Ctrl+G: Go to line
  - [ ] Ctrl+Plus: Zoom in
  - [ ] Ctrl+Minus: Zoom out
  - [ ] Ctrl+0: Reset zoom
  - [ ] Escape: Close dialogs

## Implementation Notes

- Editor features should work in both file diff and directory diff modes
- Preferences should be stored globally (apply to all files)
- Consider using a modal or sidebar for settings
- Some features may require editor library support (e.g., CodeMirror, Monaco)

## UI Mockup

```
Toolbar: [< Back] [Refresh] | [Line Wrap] [Whitespace] [Line Numbers] | [Find] [Go To] | [A-] [A+] [Theme ▾]
```

## Testing

- [ ] Toggle each display option and verify it works
- [ ] Test keyboard shortcuts
- [ ] Find in file with various patterns
- [ ] Go to line with valid/invalid input
- [ ] Zoom in/out and verify readability
- [ ] Verify preferences persist across restarts
