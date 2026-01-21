# Milestone 5b: Theme Support

**Status:** Complete
**Parent:** Milestone 5

## Overview
Add theme support for syntax highlighting and overall UI.

## Tasks

- [x] Implement theme system architecture
  - [x] Create theme configuration structure
  - [x] Add theme state management (store/context)
  - [x] Add theme persistence (localStorage)
- [x] Add popular syntax highlighting themes:
  - [x] GitHub Dark (default dark)
  - [x] GitHub Dark Dimmed
  - [x] GitHub Light (default light)
  - [x] Vitesse Dark
  - [x] Vitesse Light
  - [x] Dracula
  - [x] Monokai
  - [x] Nord
  - [x] One Dark Pro
  - [x] One Light
  - [x] Tokyo Night
  - [x] Min Light
- [x] Create theme selector UI component
  - [x] Dropdown for theme selection
  - [x] Live preview of theme changes
  - [x] Group by light/dark
- [x] Ensure diff highlights work with all themes
  - [x] Diff backgrounds layer over syntax colors
  - [x] Text remains readable
- [ ] Add UI theme matching (optional - deferred)
  - [ ] Match app background to editor theme
  - [ ] Match toolbar colors

## Theme Data Structure

```typescript
interface Theme {
  name: string;
  type: 'light' | 'dark';
  colors: {
    background: string;
    foreground: string;
    lineNumber: string;
    selection: string;
    // ... syntax colors
  };
}
```

## Implementation Notes

- Theme should apply globally (not per-file)
- Should integrate with syntax highlighting library's theme system
- Consider CSS custom properties for easy theme switching
- Default to system theme preference (prefers-color-scheme)

## Implementation Summary

**Themes Added:** 12 themes (8 dark, 4 light)
- All themes loaded at highlighter initialization for instant switching
- Themes grouped in dropdown by dark/light

**Architecture:**
- Created `src/lib/stores/syntaxTheme.svelte.ts`:
  - `syntaxThemeStore` - Reactive store for current theme
  - Persists to localStorage (`diffvibe-syntax-theme`)
  - Defaults based on system preference
- Updated `src/lib/utils/syntax.ts`:
  - Added `AVAILABLE_THEMES` export with theme metadata
  - Added `setTheme()` and `getTheme()` functions
  - Modified `getHighlighter()` to load all themes upfront
  - Updated `highlightLines()` to use `currentTheme`
- Created `src/lib/components/SyntaxThemeSelector.svelte`:
  - Dropdown with grouped themes (dark/light)
  - Shows current theme name
  - Dispatches `syntax-theme-changed` event
- Modified `src/lib/components/DiffView.svelte`:
  - Listens for `syntax-theme-changed` event
  - Triggers re-highlighting on theme change
- Modified `src/routes/+layout.svelte`:
  - Initializes `syntaxThemeStore` on mount
  - Added `SyntaxThemeSelector` to header

**Key Features:**
- ✅ Instant theme switching (all themes pre-loaded)
- ✅ Theme persistence across restarts
- ✅ Live preview - changes apply immediately
- ✅ Grouped by light/dark in dropdown
- ✅ Shows active theme with checkmark
- ✅ Diff highlights remain visible (layered properly)

## Testing

- [x] Switch between themes and verify syntax highlighting updates
- [x] Test light and dark themes
- [x] Verify diff highlights remain visible in all themes
- [x] Theme persistence works (localStorage)
- [x] Works with files of different languages
