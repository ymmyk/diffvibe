# Current Todo

This file tracks the current work in progress.

## Active Work

### Milestone 3: Three-Way Merge

#### 3a: Backend - Three-Way Diff Algorithm
- [ ] Add `MergeResult` struct with conflict tracking
- [ ] Implement three-way diff algorithm (LCS-based)
- [ ] Detect conflicts vs auto-resolvable changes
- [ ] Add `compute_three_way_diff(base, local, remote)` Tauri command
- [ ] Unit tests for three-way diff

#### 3b: Frontend - Three-Pane Layout
- [ ] Create `ThreeWayView.svelte` component (base/local/remote panes)
- [ ] Center/merged result pane
- [ ] Synchronized scrolling across 3 panes
- [ ] Wire up to new Tauri command

#### 3c: Conflict Resolution UI
- [ ] Conflict markers in merged pane
- [ ] Resolution buttons: use left / use right / use both / edit
- [ ] Conflict navigation (next/prev)
- [ ] Conflict counter in status bar

#### 3d: Git Mergetool Integration
- [ ] Accept `--output` flag for merged result
- [ ] Write merged content to output file
- [ ] Exit codes (0=resolved, 1=conflicts remain)
- [ ] Handle BASE/LOCAL/REMOTE/MERGED args

## Backlog

- [ ] Add GitHub Actions release workflow (Mac/Linux/Windows builds)
- [ ] Add `just release x.y.z` command (already exists, verify it works)
- [ ] Update README.md with proper open source format

## Completed

- [x] Milestone 0: Project Setup
- [x] Milestone 1: Basic File Diff (Two-Way)
- [x] Tab system
- [x] Diff viewer search (Cmd+F)
- [x] Milestone 2: File Editing & Merge Actions

## Next Up

- [ ] Milestone 3: Three-Way Merge
- [ ] Milestone 4: Directory Comparison
- [ ] Milestone 5: Syntax Highlighting & Editor
- [ ] Milestone 6: Polish & Advanced Features
- [ ] Milestone 7: Version Control Integration
