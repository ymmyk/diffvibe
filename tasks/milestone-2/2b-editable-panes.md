# Milestone 2b: Editable Panes

**Status:** Complete
**Parent:** Milestone 2

## Overview
Make diff panes editable with inline text editing.

## Tasks

- [x] Replace readonly spans with contenteditable or textarea
- [x] Track content changes per line
- [x] Update diff highlighting on edit
- [x] Re-compute diff after edit (debounced)
- [ ] Maintain cursor position during re-render (deferred - test first)

## Considerations

- Could use `contenteditable` divs for inline feel
- Or switch to textarea per line
- Or use a lightweight editor (CodeMirror, Monaco)
- Start simple: contenteditable with manual handling

## Test

Edit text in pane, verify diff updates. Check cursor stays in place.
