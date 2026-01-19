# Milestone 2b: Editable Panes

**Status:** Not Started
**Parent:** Milestone 2

## Overview
Make diff panes editable with inline text editing.

## Tasks

- [ ] Replace readonly spans with contenteditable or textarea
- [ ] Track content changes per line
- [ ] Update diff highlighting on edit
- [ ] Re-compute diff after edit (debounced)
- [ ] Maintain cursor position during re-render

## Considerations

- Could use `contenteditable` divs for inline feel
- Or switch to textarea per line
- Or use a lightweight editor (CodeMirror, Monaco)
- Start simple: contenteditable with manual handling

## Test

Edit text in pane, verify diff updates. Check cursor stays in place.
