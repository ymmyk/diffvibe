# DiffVibe

A cross-platform visual diff and merge tool inspired by [Meld](https://meldmerge.org/), built with Tauri 2 and Svelte 5.

## Tech Stack

- **Backend**: Rust with Tauri 2
- **Frontend**: Svelte 5 with SvelteKit, TypeScript
- **Package Manager**: pnpm
- **Task Runner**: just (see `justfile`)

## Development Commands

```bash
just install    # Install dependencies
just dev        # Run in development mode
just build      # Build for production
just test       # Run Rust tests
just check-all  # Check Rust + frontend types
just fmt        # Format Rust code
just lint       # Lint with clippy
```

## Project Structure

```
diffvibe/
├── src/                    # Svelte frontend
│   ├── routes/             # SvelteKit routes
│   ├── lib/                # Shared components and utilities
│   └── app.html            # HTML template
├── src-tauri/              # Rust backend
│   ├── src/
│   │   ├── lib.rs          # Tauri commands and setup
│   │   └── main.rs         # Entry point
│   ├── Cargo.toml          # Rust dependencies
│   └── tauri.conf.json     # Tauri configuration
├── justfile                # Task runner commands
└── package.json            # Node dependencies
```

## Git Integration

DiffVibe can be used as a git difftool and mergetool:

```bash
# As difftool
git config --global diff.tool diffvibe
git config --global difftool.diffvibe.cmd 'diffvibe "$LOCAL" "$REMOTE"'
git config --global difftool.prompt false

# As mergetool
git config --global merge.tool diffvibe
git config --global mergetool.diffvibe.cmd 'diffvibe "$LOCAL" "$BASE" "$REMOTE" --output "$MERGED"'
git config --global mergetool.diffvibe.trustExitCode true
```

---

# Milestones

## Milestone 0: Project Setup ✅

- [x] Initialize Tauri 2 + Svelte 5 + SvelteKit project
- [x] Configure pnpm as package manager
- [x] Set up justfile with development commands
- [x] Configure project metadata (name, description, window settings)
- [x] Document git difftool/mergetool configuration
- [ ] Set up basic app shell with navigation/layout
- [ ] Add CSS reset and base styling
- [ ] Configure Rust dependencies for diff algorithms

## Milestone 1: Basic File Diff (Two-Way)

Core feature: Compare two files side-by-side with syntax highlighting.

### Backend (Rust)
- [ ] Implement file reading with encoding detection
- [ ] Implement Myers diff algorithm (or use `similar` crate)
- [ ] Create Tauri commands:
  - `read_file(path) -> FileContent`
  - `compute_diff(left, right) -> DiffResult`
- [ ] Handle binary file detection
- [ ] Support large files with streaming/chunking

### Frontend (Svelte)
- [ ] Create two-pane editor layout
- [ ] Implement synchronized scrolling between panes
- [ ] Display diff hunks with visual highlighting:
  - Added lines (green)
  - Removed lines (red)
  - Modified lines (yellow/blue)
- [ ] Add line numbers with diff indicators
- [ ] Implement "jump to next/previous change" navigation
- [ ] Add file info header (path, size, encoding, line count)

### UI/UX
- [ ] File picker dialog (open two files)
- [ ] Drag-and-drop file support
- [ ] Recent files list
- [ ] Keyboard shortcuts (Ctrl+O, Ctrl+N, etc.)

## Milestone 2: File Editing & Merge Actions

Add ability to edit files and merge changes between panes.

### Features
- [ ] Inline editing in both panes
- [ ] "Copy to left/right" buttons for each diff hunk
- [ ] "Copy all from left/right" bulk action
- [ ] Undo/redo support
- [ ] Save file (Ctrl+S)
- [ ] Save as... dialog
- [ ] Dirty state indicator (unsaved changes)
- [ ] Confirm dialog on close with unsaved changes

### Backend
- [ ] Write file with encoding preservation
- [ ] Create backup before overwriting
- [ ] File watching for external changes

## Milestone 3: Three-Way Merge

Support for three-way merge (base, local, remote → merged).

### Backend
- [ ] Three-way diff algorithm
- [ ] Conflict detection and marking
- [ ] Auto-resolve non-conflicting changes
- [ ] Tauri command: `compute_three_way_diff(base, local, remote) -> MergeResult`

### Frontend
- [ ] Three-pane layout (or 2+1 layout like Meld)
- [ ] Center pane shows merged result
- [ ] Conflict markers with resolution options:
  - Use left
  - Use right
  - Use both
  - Edit manually
- [ ] Conflict navigation (next/prev conflict)
- [ ] Conflict counter in status bar

### Git Integration
- [ ] Accept `--output` flag for merged result
- [ ] Exit codes for git mergetool integration
- [ ] Handle git's BASE/LOCAL/REMOTE/MERGED convention

## Milestone 4: Directory Comparison

Compare two directories recursively.

### Backend
- [ ] Recursive directory scanning
- [ ] File comparison (content hash or quick byte comparison)
- [ ] Tauri commands:
  - `scan_directory(path) -> DirectoryTree`
  - `compare_directories(left, right) -> DirectoryDiff`
- [ ] Handle symlinks appropriately
- [ ] Ignore patterns (.gitignore, custom patterns)

### Frontend
- [ ] Tree view for directory structure
- [ ] Status icons for each file/folder:
  - Identical (checkmark)
  - Modified (dot)
  - Added (plus)
  - Removed (minus)
  - Conflict (exclamation)
- [ ] Filter by status (show only changed, etc.)
- [ ] Double-click to open file diff
- [ ] Expand/collapse directories
- [ ] "Copy to left/right" for files and folders

### Features
- [ ] Directory picker dialog
- [ ] Recursive diff statistics
- [ ] Ignore binary files option
- [ ] File size and date comparison mode

## Milestone 5: Syntax Highlighting & Editor Features

Enhance the editing experience with proper code editing features.

### Syntax Highlighting
- [ ] Integrate syntax highlighting library (tree-sitter or highlight.js)
- [ ] Support common languages (JS, TS, Python, Rust, Go, etc.)
- [ ] Auto-detect language from file extension
- [ ] Theme support (light/dark, popular themes)

### Editor Features
- [ ] Line wrapping toggle
- [ ] Show/hide whitespace
- [ ] Show/hide line numbers
- [ ] Find in file (Ctrl+F)
- [ ] Go to line (Ctrl+G)
- [ ] Font size adjustment
- [ ] Tab size configuration

## Milestone 6: Polish & Advanced Features

### UI Polish
- [ ] Application menu bar
- [ ] Toolbar with common actions
- [ ] Status bar with file info and diff statistics
- [ ] Tab support for multiple comparisons
- [ ] Preferences/settings dialog
- [ ] Keyboard shortcut customization

### Advanced Diff Features
- [ ] Ignore whitespace option
- [ ] Ignore case option
- [ ] Ignore blank lines option
- [ ] Word-level diff highlighting (within changed lines)
- [ ] Character-level diff for small changes
- [ ] Move detection (detect moved blocks)

### Performance
- [ ] Virtual scrolling for large files
- [ ] Background diff computation
- [ ] Caching for directory comparisons
- [ ] Memory-efficient handling of large files

### Platform Integration
- [ ] System tray support (optional)
- [ ] File association registration
- [ ] "Open with DiffVibe" context menu (installer)
- [ ] Auto-update mechanism
- [ ] Crash reporting

## Milestone 7: Version Control Integration

Deep integration with version control systems.

### Git Integration
- [ ] Show git status in directory view
- [ ] Compare with HEAD/index/specific commit
- [ ] View file history
- [ ] Blame view
- [ ] Stage/unstage changes from diff view

### Commands
- [ ] `diffvibe --git-diff <commit>` - Compare working tree with commit
- [ ] `diffvibe --git-log <file>` - Show file history
- [ ] `diffvibe --git-blame <file>` - Show blame annotations

---

## Feature Parity with Meld

Key Meld features to replicate:

- [x] Two-way file comparison
- [ ] Three-way file comparison and merge
- [ ] Directory comparison
- [ ] Version control integration
- [ ] Syntax highlighting
- [ ] Synchronized scrolling
- [ ] Inline editing
- [ ] Copy changes between panes
- [ ] Ignore options (whitespace, case, etc.)
- [ ] Filter patterns for directory comparison

## Non-Goals (for now)

- CVS/SVN/Bazaar integration (focus on git)
- Plugin system
- Remote file comparison (SSH, etc.)
- Image diff
- Hex/binary diff
