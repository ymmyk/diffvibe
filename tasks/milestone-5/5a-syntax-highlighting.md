# Milestone 5a: Syntax Highlighting

**Status:** Complete
**Parent:** Milestone 5

## Overview
Integrate syntax highlighting library and support common programming languages.

## Tasks

- [x] Research & choose syntax highlighting library
  - [x] Evaluate tree-sitter (more accurate, AST-based)
  - [x] Evaluate highlight.js (simpler, regex-based)
  - [x] Evaluate shiki (VS Code engine, theme support) ✅ **Chosen**
  - [x] Consider bundle size and performance
- [x] Integrate chosen library into DiffView component
- [x] Implement language auto-detection from file extension
- [x] Support common languages:
  - [x] JavaScript/TypeScript
  - [x] Python
  - [x] Rust
  - [x] Go
  - [x] Java/C#
  - [x] HTML/CSS
  - [x] JSON/YAML
  - [x] Shell/Bash
  - [x] Markdown
- [x] Apply syntax highlighting to both diff panes
- [x] Ensure highlighting works with diff blocks (added/removed/changed)
- [x] Add fallback for unsupported file types (plain text)

## Library Comparison

**Tree-sitter:**
- Pros: Very accurate, AST-based, extensible
- Cons: Larger bundle, more complex setup, requires WASM

**Highlight.js:**
- Pros: Simple, small, many languages
- Cons: Regex-based (less accurate), limited theme support

**Shiki:**
- Pros: VS Code quality, excellent themes, accurate
- Cons: Larger bundle, requires TextMate grammars

## Implementation Notes

- Should integrate with existing `DiffView.svelte` component
- Need to handle large files efficiently (don't re-highlight on every scroll)
- Should preserve existing diff highlighting (green/red backgrounds)
- Consider lazy loading language grammars to reduce initial bundle size

## Implementation Summary

**Library Chosen:** Shiki v3.21.0
- Uses VS Code's TextMate grammar engine
- ~600KB bundle size (smaller than alternatives)
- Excellent theme support built-in
- Lazy initialization on first use

**Architecture:**
- Created `src/lib/utils/syntax.ts` with:
  - `getHighlighter()` - Lazy highlighter initialization
  - `detectLanguage()` - File extension → language mapping
  - `highlightLines()` - Batch highlight lines preserving structure
- Modified `DiffView.svelte`:
  - Added `highlightedHtml` field to `PaneLine` interface
  - Async highlighting in `$effect` when pane lines change
  - Separate derived state `paneLinesWithHighlighting` with HTML
- Modified `DiffPane.svelte`:
  - Conditionally render `{@html line.highlightedHtml}` when available
  - Falls back to plain text when search is active or during editing
  - Maintains contenteditable for edit mode

**Key Features:**
- Syntax colors layer *under* diff background colors (green/red)
- Search highlighting disables syntax highlighting (for simplicity)
- Editing mode shows plain text (contenteditable doesn't work well with styled HTML)
- Async non-blocking - UI stays responsive during highlighting

## Testing

```bash
# Test with various file types
diffvibe /tmp/test1.js /tmp/test2.js
# Can also test: .rs, .py, .go, .ts, .html, .json, etc.
```

Verify:
- [x] Syntax highlighting appears correctly
- [x] Language detection works
- [x] Diff highlights (added/removed lines) still visible
- [x] Performance remains good (highlighting is async)
