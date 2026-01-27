# Tree Navigation Rewrite Plan

## Current Implementation Analysis

**File:** `src/lib/components/DirectoryCompareView.svelte` (1,216 lines)

**Current Flow:**
1. Calls `scan_directory` Rust command for both dirs in parallel
2. Stores results in `leftResult` and `rightResult`
3. Frontend merges trees into `alignedEntries` using `mergeEntries()`
4. Uses basic virtual scrolling (renders only visible rows)
5. State is preserved in `tabStates` Map

**Current Issues:**
- Diff computation happens on frontend (mergeEntries runs in UI thread)
- No real progress reporting during scan
- Virtual scrolling is basic (renders all rows, just positions them)
- Lots of frontend logic for tree merging

## New Architecture

### Phase 1: Rust Background Thread for Diff Computation

**New Rust Command:** `compare_directories_async`
- Input: `left_path`, `right_path`, `ignore_patterns`
- Runs in background thread (tokio::spawn_blocking)
- Returns full aligned tree structure with diff status
- Emits progress events during scan

**Changes:**
- Create new Tauri event: `directory-scan-progress`
- Emit events: `{ phase: 'scanning-left', files: 123 }`, `{ phase: 'scanning-right', files: 456 }`, `{ phase: 'merging', total: 579 }`
- Return `AlignedScanResult` with merged tree and stats

**Rust Types:**
```rust
#[derive(Serialize, Deserialize)]
struct AlignedEntry {
    name: String,
    rel_path: String,
    is_dir: bool,
    left_size: Option<u64>,
    right_size: Option<u64>,
    status: EntryStatus, // Match, Modified, LeftOnly, RightOnly
    children: Vec<AlignedEntry>,
}

#[derive(Serialize, Deserialize)]
enum EntryStatus {
    Match,
    Modified,
    LeftOnly,
    RightOnly,
}

#[derive(Serialize, Deserialize)]
struct AlignedScanResult {
    root_left: String,
    root_right: String,
    entries: Vec<AlignedEntry>,
    stats: ComparisonStats,
}

#[derive(Serialize, Deserialize)]
struct ComparisonStats {
    total_files: usize,
    identical: usize,
    modified: usize,
    left_only: usize,
    right_only: usize,
}
```

### Phase 2: Progress Reporting

**Frontend Changes:**
- Listen to `directory-scan-progress` event
- Show progress bar with current phase and file count
- Display: "Scanning left directory... (1,234 files)" → "Scanning right directory... (2,456 files)" → "Computing differences..."

**Progress Bar UI:**
- Show during initial load
- Indeterminate spinner + text description
- Total file count updates in real-time

### Phase 3: Improved Virtual Scrolling

**Current Implementation:**
- Renders only visible rows (lines 825-827)
- Uses `flatRows` array with all entries flattened
- Problem: `flattenTree()` still processes entire tree on every expand/collapse

**New Implementation:**
- Use proper virtual list library or optimized custom implementation
- Only flatten visible portions of tree
- Lazy flatten on scroll/expand
- Cache flattened results per expansion state

**Optimization:**
- Move `flattenTree()` to only run when tree structure changes (not on every scroll)
- Use memoization for expensive computations
- Consider using Svelte stores for large data instead of $state

### Phase 4: State Preservation

**Current Implementation:**
- Saves state to `tabStates` Map on scroll/expand
- Restores on mount

**Issues:**
- Results are re-fetched on tab switch (lines 196-199)
- State is lost on component unmount

**New Implementation:**
- Store `AlignedScanResult` in tab store globally
- Never re-fetch unless user explicitly refreshes
- Persist all UI state: expanded, scroll position, selected files, filter settings

**Tab Store Changes:**
```typescript
interface DirectoryTab {
  type: 'directory';
  leftPath: string;
  rightPath: string;
  // Cached data
  scanResult: AlignedScanResult | null;
  loading: boolean;
  // UI state
  expanded: Record<string, boolean>;
  scrollTop: number;
  leftSelected: string | null;
  rightSelected: string | null;
  filter: 'all' | 'changed' | 'identical';
  showIgnored: boolean;
}
```

## Implementation Plan

### Step 1: Create Rust Types and Backend Logic
- [ ] Add `AlignedEntry`, `EntryStatus`, `AlignedScanResult`, `ComparisonStats` types
- [ ] Implement `compare_directories_async` command
- [ ] Implement tree merging logic in Rust (port from frontend)
- [ ] Add ignore pattern support
- [ ] Add progress event emissions

### Step 2: Add Progress Reporting
- [ ] Set up Tauri event listener in frontend
- [ ] Create progress UI component
- [ ] Wire up progress events to UI

### Step 3: Update Frontend to Use New Backend
- [ ] Update types in `src/lib/types/diff.ts`
- [ ] Replace `loadDirectories()` to call new command
- [ ] Remove `mergeEntries()` from frontend (now done in Rust)
- [ ] Simplify component to just display aligned tree

### Step 4: Optimize Virtual Scrolling
- [ ] Optimize `flattenTree()` to memoize results
- [ ] Only re-flatten on tree structure changes
- [ ] Add performance markers for large trees

### Step 5: Improve State Persistence
- [ ] Move scan results to tab store
- [ ] Store all UI state in tab
- [ ] Remove re-fetching on tab switch
- [ ] Test state preservation across tab switches

### Step 6: Testing
- [ ] Test with small directories (100s of files)
- [ ] Test with medium directories (1,000s of files)
- [ ] Test with large directories (20,000+ files)
- [ ] Test state preservation
- [ ] Test ignore patterns
- [ ] Test all filters and UI interactions

## Key Benefits

1. **Performance:** Diff computation happens in background Rust thread (won't block UI)
2. **Progress Feedback:** Real-time progress updates during scan
3. **Simplicity:** Frontend just displays data, no heavy computation
4. **State Preservation:** Complete state saved across tab switches
5. **Scalability:** Optimized for 20k+ files with proper virtual scrolling
6. **Better UX:** Folders expanded by default, faster initial display

## File Changes Summary

**New Files:**
- None (all changes in existing files)

**Modified Files:**
1. `src-tauri/src/lib.rs` - Add new types and command
2. `src/lib/types/diff.ts` - Add TypeScript types
3. `src/lib/components/DirectoryCompareView.svelte` - Complete rewrite
4. `src/lib/stores/tabs.svelte.ts` - Add scan result caching

**Removed:**
- Frontend `mergeEntries()` logic (moved to Rust)
- Re-scanning logic on tab switch
