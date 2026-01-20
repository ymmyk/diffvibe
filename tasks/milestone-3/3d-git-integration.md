# Milestone 3d: Git Integration

**Status:** Complete
**Parent:** Milestone 3

## Overview
CLI integration for use as git mergetool.

## Tasks

- [x] Accept `--output` flag for merged result path
- [x] Write merged content to output file on save
- [x] Exit codes: 0=resolved, 1=conflicts remain
- [x] Handle git's BASE/LOCAL/REMOTE/MERGED args
- [x] Auto-detect merge mode from arg count

## CLI Usage

```bash
# Git mergetool convention
diffvibe "$LOCAL" "$BASE" "$REMOTE" --output "$MERGED"

# Or positional
diffvibe --merge local.txt base.txt remote.txt -o merged.txt
```

## Git Config

```bash
git config --global merge.tool diffvibe
git config --global mergetool.diffvibe.cmd 'diffvibe "$LOCAL" "$BASE" "$REMOTE" --output "$MERGED"'
git config --global mergetool.diffvibe.trustExitCode true
```

## Exit Codes

- `0` - All conflicts resolved, merged file written
- `1` - Conflicts remain or user cancelled

## Test

Run as git mergetool on a conflicted file, resolve, verify merged output.
