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

## Testing

```bash
# Test with various file types
diffvibe test.js test2.js
diffvibe main.rs main2.rs
diffvibe script.py script2.py
```

Verify:
- Syntax highlighting appears correctly
- Language detection works
- Diff highlights (added/removed lines) still visible
- Performance remains good on large files
