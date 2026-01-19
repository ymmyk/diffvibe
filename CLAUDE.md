# DiffVibe

A cross-platform visual diff and merge tool inspired by [Meld](https://meldmerge.org/), built with Tauri 2 and Svelte 5.

## Tech Stack

- **Backend**: Rust with Tauri 2
- **Frontend**: Svelte 5 with SvelteKit, TypeScript
- **Package Manager**: pnpm
- **Task Runner**: just (see `justfile`)

## Basic Rules

In all interactions and commit messages, be extremely concise and sacrifice grammar for the sake of concision.

1. First think through the problem, read the codebase for relevant files and related claude.md, and write a plan to tasks/todo.md.
2. The plan should have a list of todo items that you can check off as you complete them
3. Before you begin working, check in with me and I will verify the plan.
4. Then, begin working on the todo items, marking them as complete as you go.
5. Please every step of the way just give me a high level explanation of what changes you made
6. Make every task and code change you do as simple as possible. We want to avoid making any massive or complex changes. Every change should impact as little code as possible. Everything is about simplicity.
7. Update the relevant claude.md files.
8. DO NOT BE LAZY. NEVER BE LAZY. IF THERE IS A BUG FIND THE ROOT CAUSE AND FIX IT. NO TEMPORARY FIXES. YOU ARE A SENIOR DEVELOPER. NEVER BE LAZY
9. MAKE ALL FIXES AND CODE CHANGES AS SIMPLE AS HUMANLY POSSIBLE. THEY SHOULD ONLY IMPACT NECESSARY CODE RELEVANT TO THE TASK AND NOTHING ELSE. IT SHOULD IMPACT AS LITTLE CODE AS POSSIBLE. YOUR GOAL IS TO NOT INTRODUCE ANY BUGS. IT'S ALL ABOUT SIMPLICITY


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

See `tasks/` folder for detailed task breakdowns. Current work tracked in `tasks/todo.md`.

| Milestone | Description | Status | Task File |
|-----------|-------------|--------|-----------|
| 0 | Project Setup | ✅ Complete | [milestone-0-project-setup.md](tasks/milestone-0-project-setup.md) |
| 1 | Basic File Diff (Two-Way) | Not Started | [milestone-1-basic-file-diff.md](tasks/milestone-1-basic-file-diff.md) |
| 2 | File Editing & Merge Actions | Not Started | [milestone-2-file-editing.md](tasks/milestone-2-file-editing.md) |
| 3 | Three-Way Merge | Not Started | [milestone-3-three-way-merge.md](tasks/milestone-3-three-way-merge.md) |
| 4 | Directory Comparison | Not Started | [milestone-4-directory-comparison.md](tasks/milestone-4-directory-comparison.md) |
| 5 | Syntax Highlighting & Editor | Not Started | [milestone-5-syntax-highlighting.md](tasks/milestone-5-syntax-highlighting.md) |
| 6 | Polish & Advanced Features | Not Started | [milestone-6-polish.md](tasks/milestone-6-polish.md) |
| 7 | Version Control Integration | Not Started | [milestone-7-version-control.md](tasks/milestone-7-version-control.md) |

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
