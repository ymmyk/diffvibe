# Milestone 5d: Editor Library Integration

**Status:** Not Started
**Parent:** Milestone 5

## Overview
Replace basic textarea with a proper code editor library to support advanced features.

## Tasks

- [ ] Choose editor library
  - [ ] Evaluate CodeMirror 6 (modern, lightweight, extensible)
  - [ ] Evaluate Monaco Editor (VS Code engine, feature-rich, heavier)
  - [ ] Consider bundle size and performance
  - [ ] Consider Svelte integration
- [ ] Replace current diff view with editor instances
  - [ ] Integrate two editor instances (left/right)
  - [ ] Migrate existing diff rendering logic
  - [ ] Maintain line-by-line alignment
- [ ] Configure editor for diff mode
  - [ ] Read-only mode (no editing yet - Milestone 2 handles editing)
  - [ ] Disable unnecessary features (autocomplete, etc.)
  - [ ] Configure line gutters (line numbers, diff markers)
- [ ] Integrate syntax highlighting from 5a
  - [ ] Connect language detection
  - [ ] Apply highlighting extensions
- [ ] Integrate themes from 5b
  - [ ] Apply theme to editor instances
  - [ ] Ensure theme switching works
- [ ] Implement synchronized scrolling
  - [ ] Link scroll positions between editors
  - [ ] Handle different file lengths
  - [ ] Smooth scrolling behavior
- [ ] Add diff decorations
  - [ ] Background colors for added/removed/changed lines
  - [ ] Inline change highlighting (word-level diffs)
  - [ ] Diff markers in gutter
- [ ] Optimize performance
  - [ ] Lazy rendering for large files
  - [ ] Virtual scrolling if needed
  - [ ] Debounce expensive operations

## Editor Comparison

**CodeMirror 6:**
- Pros: Modern, modular, good performance, smaller bundle
- Cons: Fewer built-in features, more manual setup
- Bundle: ~100-200KB

**Monaco Editor:**
- Pros: Full VS Code experience, many features, excellent syntax highlighting
- Cons: Large bundle (~2-3MB), complex setup, may be overkill
- Bundle: ~2-3MB

## Recommended Choice

**CodeMirror 6** - Better balance of features, performance, and bundle size for a diff tool.

## Implementation Notes

- This is a foundational change affecting all diff views
- Should maintain existing diff computation logic (Rust backend)
- Editor is just for display and navigation, not editing (yet)
- Consider creating a reusable `EditorPane` component
- May need custom extensions for diff-specific features

## Migration Strategy

1. Create new `EditorPane.svelte` component with CodeMirror
2. Test with simple file diff
3. Migrate diff decorations (colors, markers)
4. Implement synchronized scrolling
5. Replace `DiffView.svelte` to use new editor component
6. Test thoroughly with various file types and sizes
7. Clean up old textarea-based implementation

## Testing

- [ ] Compare two files and verify editor appears
- [ ] Verify syntax highlighting works
- [ ] Test synchronized scrolling
- [ ] Test with large files (10k+ lines)
- [ ] Test with various file types
- [ ] Verify diff decorations (colors, markers)
- [ ] Test theme switching
- [ ] Test all display options (wrap, whitespace, etc.)

## Dependencies

```bash
pnpm add @codemirror/state @codemirror/view @codemirror/language @codemirror/commands
```

Additional language packages as needed.
