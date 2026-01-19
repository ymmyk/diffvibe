# Milestone 3: Three-Way Merge

**Status:** Not Started

## Overview
Support for three-way merge (base, local, remote → merged).

## Backend

- [ ] Three-way diff algorithm
- [ ] Conflict detection and marking
- [ ] Auto-resolve non-conflicting changes
- [ ] Tauri command: `compute_three_way_diff(base, local, remote) -> MergeResult`

## Frontend

- [ ] Three-pane layout (or 2+1 layout like Meld)
- [ ] Center pane shows merged result
- [ ] Conflict markers with resolution options:
  - Use left
  - Use right
  - Use both
  - Edit manually
- [ ] Conflict navigation (next/prev conflict)
- [ ] Conflict counter in status bar

## Git Integration

- [ ] Accept `--output` flag for merged result
- [ ] Exit codes for git mergetool integration
- [ ] Handle git's BASE/LOCAL/REMOTE/MERGED convention
