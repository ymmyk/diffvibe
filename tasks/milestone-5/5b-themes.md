# Milestone 5b: Theme Support

**Status:** Not Started
**Parent:** Milestone 5

## Overview
Add theme support for syntax highlighting and overall UI.

## Tasks

- [ ] Implement theme system architecture
  - [ ] Create theme configuration structure
  - [ ] Add theme state management (store/context)
  - [ ] Add theme persistence (localStorage)
- [ ] Add popular syntax highlighting themes:
  - [ ] VS Code Dark+ (default dark)
  - [ ] VS Code Light+ (default light)
  - [ ] GitHub Dark
  - [ ] GitHub Light
  - [ ] Dracula
  - [ ] Monokai
  - [ ] Solarized Dark/Light
- [ ] Create theme selector UI component
  - [ ] Dropdown or modal for theme selection
  - [ ] Live preview of theme changes
  - [ ] Group by light/dark
- [ ] Ensure diff highlights work with all themes
  - [ ] Test added/removed line backgrounds
  - [ ] Test changed chunk backgrounds
  - [ ] Ensure text remains readable on all backgrounds
- [ ] Add UI theme matching (optional)
  - [ ] Match app background to editor theme
  - [ ] Match toolbar colors
  - [ ] Consider light/dark mode toggle

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

## Testing

- [ ] Switch between themes and verify syntax highlighting updates
- [ ] Test light and dark themes
- [ ] Verify diff highlights remain visible in all themes
- [ ] Check theme persistence across app restarts
- [ ] Test with files of different languages
