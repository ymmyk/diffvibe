# Milestone 0: Project Setup ✅

**Status:** Complete

## Overview
Initial project scaffolding with Tauri 2 + Svelte 5 + SvelteKit.

## Tasks

- [x] Initialize Tauri 2 + Svelte 5 + SvelteKit project
- [x] Configure pnpm as package manager
- [x] Set up justfile with development commands
- [x] Configure project metadata (name, description, window settings)
- [x] Document git difftool/mergetool configuration
- [x] Set up basic app shell with navigation/layout
- [x] Add CSS reset and base styling (with light/dark theme toggle)
- [x] Configure Rust dependencies for diff algorithms (`similar` crate)

## Files Created/Modified

- `package.json` - Node dependencies
- `src-tauri/Cargo.toml` - Rust dependencies with `similar` crate
- `src-tauri/tauri.conf.json` - App config (window size, metadata)
- `src-tauri/capabilities/default.json` - Tauri permissions
- `src/app.css` - Global CSS with theme variables
- `src/routes/+layout.svelte` - App shell with header and theme toggle
- `src/routes/+page.svelte` - Home page with mode selector
- `src/lib/types/diff.ts` - TypeScript types
- `src/lib/stores/theme.svelte.ts` - Theme store with persistence
- `src/lib/utils/index.ts` - Utility functions
- `justfile` - Development commands
- `AGENTS.md` - Project documentation
