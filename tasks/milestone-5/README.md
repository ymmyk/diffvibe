# Milestone 5: Syntax Highlighting & Editor Features

**Status:** Not Started

## Overview
Enhance the editing experience with proper code editing features, syntax highlighting, themes, and a professional editor library.

## Subtasks

| # | Subtask | Description | Status |
|---|---------|-------------|--------|
| 5a | [Syntax Highlighting](5a-syntax-highlighting.md) | Integrate syntax highlighting library, language detection | Not Started |
| 5b | [Themes](5b-themes.md) | Theme support for syntax highlighting and UI | Not Started |
| 5c | [Editor Features](5c-editor-features.md) | Line wrap, whitespace, find, go-to-line, zoom | Not Started |
| 5d | [Editor Integration](5d-editor-integration.md) | Replace textarea with CodeMirror/Monaco | Not Started |

## Recommended Order

1. **5d Editor Integration** - Foundation: replace textarea with proper editor library
2. **5a Syntax Highlighting** - Core feature: add syntax highlighting
3. **5b Themes** - Polish: add theme support for better UX
4. **5c Editor Features** - Enhancement: add navigation and display options

## Notes

- 5d is foundational and should be done first (switching from textarea to real editor)
- 5a and 5b are tightly coupled (syntax highlighting + themes)
- 5c adds nice-to-have features on top of the editor foundation
- All features should work in both file diff and directory diff modes
