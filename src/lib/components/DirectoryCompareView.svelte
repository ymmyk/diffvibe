<script lang="ts">
  import { invoke } from '@tauri-apps/api/core';
  import { confirm } from '@tauri-apps/plugin-dialog';
  import { onMount, tick } from 'svelte';
  import type { ScanResult, DirEntry, DiffStats } from '$lib/types';
  import { tabStore } from '$lib/stores/tabs.svelte';

  interface Props {
    leftPath: string;
    rightPath: string;
    tabId: string;
  }

  let { leftPath, rightPath, tabId }: Props = $props();

  let leftResult = $state<ScanResult | null>(null);
  let rightResult = $state<ScanResult | null>(null);
  let error = $state<string | null>(null);
  let loading = $state(true);
  let scanningProgress = $state<string>("");

  // Selected files
  let leftSelected = $state<string | null>(null);
  let rightSelected = $state<string | null>(null);

  // Expanded directories (shared between both sides) - default to EXPANDED for usability
  let expanded = $state<Record<string, boolean>>({});

  // Track if modifier key is held for independent selection
  let modifierHeld = $state(false);

  // Flat list of visible rows for virtual scrolling
  interface FlatRow {
    entry: AlignedEntry;
    depth: number;
    index: number;
  }
  let flatRows = $state<FlatRow[]>([]);

  // Diff stats for selected file pair
  let selectedStats = $state<DiffStats | null>(null);
  let loadingStats = $state(false);

  // Context menu
  let contextMenu = $state<{ x: number; y: number; entry: AlignedEntry; side: 'left' | 'right' } | null>(null);

  // Filter
  let filter = $state<'all' | 'changed' | 'identical'>('all');

  // Background deep scanning state
  let isScanningDeep = $state(false);
  let deepScanProgress = $state<string>("");
  let deepLeftResult = $state<ScanResult | null>(null);
  let deepRightResult = $state<ScanResult | null>(null);

  // Virtual scrolling state
  let scrollTop = $state(0);
  let containerHeight = $state(600);
  const ROW_HEIGHT = 28;
  const OVERSCAN = 10; // Render extra rows above/below viewport

  // Ignore patterns
  let showIgnored = $state(false);
  let ignorePatterns = $state<string[]>([
    'node_modules/',
    '.git/',
    '.next/',
    '_next/',
    'dist/',
    'build/',
    'out/',
    '.nuxt/',
    '.output/',
    '.vercel/',
    '.cache/',
    'target/',
    'Cargo.lock',
    '__pycache__/',
    '*.pyc',
    '*.log',
    '.DS_Store',
    'Thumbs.db',
    '.env*',
    '*.tmp',
    '*.swp',
    '*.swo'
  ]);

  // Merged/aligned entry for display
  interface AlignedEntry {
    name: string;
    relPath: string;
    isDir: boolean;
    left: DirEntry | null;
    right: DirEntry | null;
    status: 'match' | 'modified' | 'left-only' | 'right-only';
    children: AlignedEntry[];
  }

  async function loadDirectories() {
    loading = true;
    error = null;
    scanningProgress = "Scanning directories (shallow)...";

    // Ensure UI updates immediately
    await tick();

    try {
      // Only apply ignore patterns during scanning if showIgnored is false
      const patternsToIgnore = showIgnored ? [] : ignorePatterns;

      // Use lazy scanning with limited depth (2 levels initially)
      scanningProgress = `Scanning ${getFileName(leftPath)} and ${getFileName(rightPath)} (shallow)...`;
      await tick();

      // Scan only 2 levels deep initially for instant UI
      const results = await Promise.allSettled([
        invoke<ScanResult>('scan_directory_lazy', { 
          path: leftPath, 
          ignorePatterns: patternsToIgnore,
          maxDepth: 2 
        }),
        invoke<ScanResult>('scan_directory_lazy', { 
          path: rightPath, 
          ignorePatterns: patternsToIgnore,
          maxDepth: 2 
        }),
      ]);

      // Handle results
      const leftResultPromise = results[0];
      const rightResultPromise = results[1];

      if (leftResultPromise.status === 'fulfilled') {
        leftResult = leftResultPromise.value;
        scanningProgress = "Left directory scanned (shallow)...";
        await tick();
      } else {
        throw new Error(`Failed to scan left directory: ${leftResultPromise.reason}`);
      }

      if (rightResultPromise.status === 'fulfilled') {
        rightResult = rightResultPromise.value;
        scanningProgress = "";
        await tick();
      } else {
        throw new Error(`Failed to scan right directory: ${rightResultPromise.reason}`);
      }

      // Auto-expand all on initial load
      await tick();
      expandAll();
      await tick();

      // Start deep scan in background
      startDeepScan();

    } catch (e) {
      error = e instanceof Error ? e.message : String(e);
    } finally {
      loading = false;
      scanningProgress = "";
      await tick();
    }
  }

  async function startDeepScan() {
    // Don't start another deep scan if one is already running
    if (isScanningDeep) return;

    isScanningDeep = true;
    deepScanProgress = "(scanning...)";

    // Delay start to let UI settle first
    await new Promise(resolve => setTimeout(resolve, 500));

    try {
      const patternsToIgnore = showIgnored ? [] : ignorePatterns;

      // Scan left first (async, non-blocking)
      try {
        deepLeftResult = await invoke<ScanResult>('scan_directory', { 
          path: leftPath, 
          ignorePatterns: patternsToIgnore
        });
        // Force UI update
        await tick();
        await new Promise(resolve => setTimeout(resolve, 100));
      } catch (e) {
        console.error('Left deep scan failed:', e);
      }

      // Scan right second (async, non-blocking)
      try {
        deepRightResult = await invoke<ScanResult>('scan_directory', { 
          path: rightPath, 
          ignorePatterns: patternsToIgnore
        });
        // Force UI update
        await tick();
      } catch (e) {
        console.error('Right deep scan failed:', e);
      }

      deepScanProgress = "";
    } catch (e) {
      console.error('Deep scan failed:', e);
      deepScanProgress = "";
    } finally {
      isScanningDeep = false;
    }
  }

  onMount(() => {
    loadDirectories();

    function handleKeyDown(e: KeyboardEvent) {
      if (e.key === 'Meta' || e.key === 'Control') {
        modifierHeld = true;
      }
    }
    function handleKeyUp(e: KeyboardEvent) {
      if (e.key === 'Meta' || e.key === 'Control') {
        modifierHeld = false;
      }
    }

    window.addEventListener('keydown', handleKeyDown);
    window.addEventListener('keyup', handleKeyUp);
    return () => {
      window.removeEventListener('keydown', handleKeyDown);
      window.removeEventListener('keyup', handleKeyUp);
    };
  });

  // Merge two directory trees into aligned entries
  function mergeEntries(leftEntries: DirEntry[], rightEntries: DirEntry[]): AlignedEntry[] {
    const leftMap = new Map(leftEntries.map(e => [e.name, e]));
    const rightMap = new Map(rightEntries.map(e => [e.name, e]));
    const allNames = new Set([...leftMap.keys(), ...rightMap.keys()]);

    const result: AlignedEntry[] = [];

    for (const name of allNames) {
      const left = leftMap.get(name) || null;
      const right = rightMap.get(name) || null;

      let status: AlignedEntry['status'];
      if (left && right) {
        // Both exist - check if same type and size
        if (left.is_dir !== right.is_dir) {
          status = 'modified';
        } else if (left.is_dir) {
          // Directories - check children recursively for status
          status = 'match'; // Will refine based on children
        } else {
          status = left.size === right.size ? 'match' : 'modified';
        }
      } else if (left) {
        status = 'left-only';
      } else {
        status = 'right-only';
      }

      const relPath = left?.rel_path || right?.rel_path || name;
      const isDir = left?.is_dir || right?.is_dir || false;

      let children: AlignedEntry[] = [];
      if (isDir) {
        children = mergeEntries(
          left?.children || [],
          right?.children || []
        );
        // If any child is not a match, directory is modified
        if (status === 'match' && children.some(c => c.status !== 'match')) {
          status = 'modified';
        }
      }

      result.push({ name, relPath, isDir, left, right, status, children });
    }

    // Sort: dirs first, then by name
    result.sort((a, b) => {
      if (a.isDir !== b.isDir) return a.isDir ? -1 : 1;
      return a.name.toLowerCase().localeCompare(b.name.toLowerCase());
    });

    return result;
  }

  let alignedEntries = $derived(
    leftResult && rightResult
      ? mergeEntries(leftResult.entries, rightResult.entries)
      : []
  );

  let filteredEntries = $derived(() => {
    if (filter === 'all') {
      return alignedEntries;
    }

    function filterRecursive(entries: AlignedEntry[]): AlignedEntry[] {
      return entries
        .map(entry => ({
          ...entry,
          children: entry.isDir ? filterRecursive(entry.children) : []
        }))
        .filter(entry => {
          // Show entry if it matches the filter directly
          if (shouldShowEntry(entry)) return true;

          // For directories, show if any children are visible after filtering
          if (entry.isDir && entry.children.length > 0) return true;

          return false;
        });
    }

    return filterRecursive(alignedEntries);
  });

  async function toggleExpand(entry: AlignedEntry) {
    const path = entry.relPath;
    
    // If collapsing, just toggle
    if (expanded[path]) {
      expanded[path] = false;
      return;
    }

    // If expanding and directory has no children loaded, fetch them
    if (entry.isDir && entry.children.length === 0) {
      try {
        // Load children for both sides if they exist
        const patternsToIgnore = showIgnored ? [] : ignorePatterns;
        
        if (entry.left) {
          const leftChildren = await invoke<DirEntry[]>('expand_directory', {
            path: leftPath,
            relPath: path,
            ignorePatterns: patternsToIgnore
          });
          // Update the entry with loaded children
          entry.left.children = leftChildren;
          entry.children = mergeEntries(leftChildren, entry.right?.children || []);
        }
        
        if (entry.right) {
          const rightChildren = await invoke<DirEntry[]>('expand_directory', {
            path: rightPath,
            relPath: path,
            ignorePatterns: patternsToIgnore
          });
          entry.right.children = rightChildren;
          entry.children = mergeEntries(entry.left?.children || [], rightChildren);
        }

        // Force reactivity update
        alignedEntries = alignedEntries;
      } catch (e) {
        console.error('Failed to expand directory:', e);
      }
    }

    expanded[path] = true;
  }

  function isExpanded(path: string): boolean {
    return expanded[path] ?? true; // Default to EXPANDED
  }

  // Flatten tree into list of visible rows
  function flattenTree(entries: AlignedEntry[], depth: number = 0): FlatRow[] {
    const rows: FlatRow[] = [];

    function flatten(entries: AlignedEntry[], depth: number) {
      for (const entry of entries) {
        rows.push({ entry, depth, index: rows.length });
        
        // If directory is expanded, add children
        if (entry.isDir && isExpanded(entry.relPath) && entry.children.length > 0) {
          flatten(entry.children, depth + 1);
        }
      }
    }

    flatten(entries, depth);
    return rows;
  }

  // Update flat rows when tree or expansion changes (immediate for better UX)
  $effect(() => {
    const _ = alignedEntries; // Track dependency
    const __ = expanded; // Track dependency
    const ___ = filter; // Track dependency
    
    // Only flatten if we have data
    if (alignedEntries.length > 0) {
      flatRows = flattenTree(filteredEntries());
    }
  });

  // Calculate which rows are visible based on scroll position
  let visibleRows = $derived(() => {
    const startIndex = Math.max(0, Math.floor(scrollTop / ROW_HEIGHT) - OVERSCAN);
    const endIndex = Math.min(
      flatRows.length,
      Math.ceil((scrollTop + containerHeight) / ROW_HEIGHT) + OVERSCAN
    );
    
    return flatRows.slice(startIndex, endIndex);
  });

  function handleTreeScroll(e: Event) {
    const target = e.target as HTMLElement;
    scrollTop = target.scrollTop;
  }

  async function loadDiffStats(leftRel: string, rightRel: string) {
    loadingStats = true;
    selectedStats = null;
    try {
      const leftFile = `${leftPath}/${leftRel}`;
      const rightFile = `${rightPath}/${rightRel}`;
      selectedStats = await invoke<DiffStats>('get_diff_stats', { leftPath: leftFile, rightPath: rightFile });
    } catch (e) {
      console.error('Failed to load diff stats:', e);
    } finally {
      loadingStats = false;
    }
  }

  function handleSelect(entry: AlignedEntry, side: 'left' | 'right') {
    if (entry.isDir) {
      toggleExpand(entry);
      return;
    }

    // Check if file exists on this side
    const exists = side === 'left' ? entry.left !== null : entry.right !== null;
    if (!exists) return;

    if (modifierHeld) {
      // Independent selection
      if (side === 'left') {
        leftSelected = entry.relPath;
      } else {
        rightSelected = entry.relPath;
      }
    } else {
      // Linked selection - select same file on both sides
      leftSelected = entry.relPath;
      rightSelected = entry.relPath;
    }

    // Load diff stats when both sides are selected
    if (leftSelected && rightSelected) {
      loadDiffStats(leftSelected, rightSelected);
    } else {
      selectedStats = null;
    }
  }

  function handleDoubleClick(entry: AlignedEntry) {
    if (entry.isDir) return;

    // Double-click compares matching files (same path on both sides)
    const leftFile = `${leftPath}/${entry.relPath}`;
    const rightFile = `${rightPath}/${entry.relPath}`;
    tabStore.openCompare(leftFile, rightFile, 'file', undefined, tabId);
  }

  function compareSelected() {
    if (!leftSelected || !rightSelected) return;
    const leftFile = `${leftPath}/${leftSelected}`;
    const rightFile = `${rightPath}/${rightSelected}`;
    tabStore.openCompare(leftFile, rightFile, 'file', undefined, tabId);
  }

  async function copyToRight() {
    if (!leftSelected) return;
    const leftPathFull = `${leftPath}/${leftSelected}`;
    const rightPathFull = `${rightPath}/${leftSelected}`;

    try {
      // Check if destination exists
      const destExists = await invoke('file_exists', { path: rightPathFull });
      if (destExists) {
        const confirmed = await confirm(`Path already exists. Overwrite ${rightPathFull}?`);
        if (!confirmed) return;
      }

      // Check if it's a directory or file
      const isDir = await invoke('is_directory', { path: leftPathFull });
      if (isDir) {
        await invoke('copy_dir', { fromPath: leftPathFull, toPath: rightPathFull });
      } else {
        await invoke('copy_file', { fromPath: leftPathFull, toPath: rightPathFull });
      }

      // Refresh directories after copy
      await loadDirectories();
    } catch (e) {
      console.error('Failed to copy to right:', e);
      alert(`Failed to copy: ${e}`);
    }
  }

  async function copyToLeft() {
    if (!rightSelected) return;
    const rightPathFull = `${rightPath}/${rightSelected}`;
    const leftPathFull = `${leftPath}/${rightSelected}`;

    try {
      // Check if destination exists
      const destExists = await invoke('file_exists', { path: leftPathFull });
      if (destExists) {
        const confirmed = await confirm(`Path already exists. Overwrite ${leftPathFull}?`);
        if (!confirmed) return;
      }

      // Check if it's a directory or file
      const isDir = await invoke('is_directory', { path: rightPathFull });
      if (isDir) {
        await invoke('copy_dir', { fromPath: rightPathFull, toPath: leftPathFull });
      } else {
        await invoke('copy_file', { fromPath: rightPathFull, toPath: leftPathFull });
      }

      // Refresh directories after copy
      await loadDirectories();
    } catch (e) {
      console.error('Failed to copy to left:', e);
      alert(`Failed to copy: ${e}`);
    }
  }

  function handleContextMenu(event: MouseEvent, entry: AlignedEntry, side: 'left' | 'right') {
    event.preventDefault();
    contextMenu = { x: event.clientX, y: event.clientY, entry, side };
  }

  function closeContextMenu() {
    contextMenu = null;
  }



  function matchesIgnorePattern(path: string): boolean {
    for (const pattern of ignorePatterns) {
      if (pattern.endsWith('/')) {
        // Directory pattern
        if (path.startsWith(pattern.slice(0, -1))) return true;
      } else if (pattern.includes('*')) {
        // Simple glob matching
        const regex = new RegExp(pattern.replace(/\*/g, '.*').replace(/\?/g, '.'));
        if (regex.test(path)) return true;
      } else {
        // Exact match
        if (path === pattern) return true;
      }
    }
    return false;
  }

  function shouldShowEntry(entry: AlignedEntry): boolean {
    // When showIgnored is false, ignored files were already filtered during scanning
    // When showIgnored is true, we show everything including ignored files
    // So no additional ignore filtering needed in frontend

    if (filter === 'all') return true;
    if (filter === 'changed') return entry.status === 'modified' || entry.status === 'left-only' || entry.status === 'right-only';
    if (filter === 'identical') return entry.status === 'match';
    return true;
  }

  function expandAll() {
    function expandRecursive(entries: AlignedEntry[]) {
      for (const entry of entries) {
        if (entry.isDir) {
          expanded[entry.relPath] = true;
          expandRecursive(entry.children);
        }
      }
    }
    expandRecursive(filteredEntries());
  }

  function collapseAll() {
    function collapseRecursive(entries: AlignedEntry[]) {
      for (const entry of entries) {
        if (entry.isDir) {
          expanded[entry.relPath] = false;
          collapseRecursive(entry.children);
        }
      }
    }
    collapseRecursive(filteredEntries());
  }

  function getComparisonStats() {
    let identical = 0;
    let modified = 0;
    let added = 0;
    let removed = 0;

    // Use deep scan results if available, otherwise use shallow
    const useLeft = deepLeftResult || leftResult;
    const useRight = deepRightResult || rightResult;

    if (!useLeft || !useRight) {
      return { identical, modified, added, removed };
    }

    const entries = mergeEntries(useLeft.entries, useRight.entries);

    function countRecursive(entries: AlignedEntry[]) {
      for (const entry of entries) {
        if (entry.isDir) {
          countRecursive(entry.children);
        } else {
          switch (entry.status) {
            case 'match':
              identical++;
              break;
            case 'modified':
              modified++;
              break;
            case 'left-only':
              removed++;
              break;
            case 'right-only':
              added++;
              break;
          }
        }
      }
    }

    countRecursive(entries);
    return { identical, modified, added, removed };
  }

  async function copyFromContext(side: 'left' | 'right') {
    if (!contextMenu) return;
    const { entry } = contextMenu;

    try {
      let fromPath: string;
      let toPath: string;

      if (side === 'left') {
        // Copy from left to right
        fromPath = `${leftPath}/${entry.relPath}`;
        toPath = `${rightPath}/${entry.relPath}`;
      } else {
        // Copy from right to left
        fromPath = `${rightPath}/${entry.relPath}`;
        toPath = `${leftPath}/${entry.relPath}`;
      }

      // Check if destination exists
      const destExists = await invoke('file_exists', { path: toPath });
      if (destExists) {
        const confirmed = await confirm(`Path already exists. Overwrite ${toPath}?`);
        if (!confirmed) {
          closeContextMenu();
          return;
        }
      }

      // Check if it's a directory or file
      const isDir = await invoke('is_directory', { path: fromPath });
      if (isDir) {
        await invoke('copy_dir', { fromPath, toPath });
      } else {
        await invoke('copy_file', { fromPath, toPath });
      }

      // Refresh directories after copy
      await loadDirectories();
    } catch (e) {
      console.error('Failed to copy from context menu:', e);
      alert(`Failed to copy: ${e}`);
    } finally {
      closeContextMenu();
    }
  }

  function getFileName(path: string): string {
    return path.split('/').pop() || path.split('\\').pop() || path;
  }

  function formatSize(bytes: number): string {
    if (bytes < 1024) return `${bytes} B`;
    if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
    return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
  }
</script>

<div class="dir-compare">
  <header class="header">
    <div class="actions-left">
      <button class="action-btn" onclick={() => loadDirectories()} title="Refresh">
        <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <polyline points="23 4 23 10 17 10"></polyline>
          <polyline points="1 20 1 14 7 14"></polyline>
          <path d="M3.51 9a9 9 0 0 1 14.85-3.36L23 10M1 14l4.64 4.36A9 9 0 0 0 20.49 15"></path>
        </svg>
      </button>

      <button class="action-btn" onclick={expandAll} title="Expand All">
        <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <polyline points="6 9 12 15 18 9"></polyline>
        </svg>
      </button>

      <button class="action-btn" onclick={collapseAll} title="Collapse All">
        <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <polyline points="18 15 12 9 6 15"></polyline>
        </svg>
      </button>

      <select class="filter-select" bind:value={filter}>
        <option value="all">All files</option>
        <option value="changed">Changed only</option>
        <option value="identical">Identical only</option>
      </select>

      <label class="ignore-toggle">
        <input type="checkbox" bind:checked={showIgnored} />
        <span>Show ignored files</span>
      </label>


    </div>

    <div class="selection-info">
      {#if leftSelected && rightSelected}
        <span class="selected-files">
          <span class="file-name">{leftSelected.split('/').pop()}</span>
          <span class="arrow">↔</span>
          <span class="file-name">{rightSelected.split('/').pop()}</span>
        </span>
        {#if loadingStats}
          <span class="diff-stats loading">...</span>
        {:else if selectedStats}
          <span class="diff-stats">
            {#if selectedStats.additions > 0}<span class="additions">+{selectedStats.additions}</span>{/if}
            {#if selectedStats.deletions > 0}<span class="deletions">-{selectedStats.deletions}</span>{/if}
            {#if selectedStats.additions === 0 && selectedStats.deletions === 0}<span class="identical">identical</span>{/if}
          </span>
        {/if}
        <button class="compare-btn" onclick={compareSelected}>
          Compare
        </button>
      {:else if leftSelected && !rightSelected}
        <span class="selected-files">
          <span class="file-name">{leftSelected.split('/').pop()}</span>
          <span class="arrow">→</span>
          <span class="file-name missing">right side</span>
        </span>
        <button class="copy-btn" onclick={copyToRight}>
          Copy to Right
        </button>
      {:else if rightSelected && !leftSelected}
        <span class="selected-files">
          <span class="file-name missing">left side</span>
          <span class="arrow">←</span>
          <span class="file-name">{rightSelected.split('/').pop()}</span>
        </span>
        <button class="copy-btn" onclick={copyToLeft}>
          Copy to Left
        </button>
      {/if}
    </div>

    <div class="stats">
      {#if leftResult && rightResult}
        {@const stats = getComparisonStats()}
        {@const useDeep = deepLeftResult && deepRightResult}
        <span class="stat">{useDeep ? deepLeftResult!.file_count : leftResult.file_count} / {useDeep ? deepRightResult!.file_count : rightResult.file_count} files {deepScanProgress}</span>
        <span class="stat">{stats.identical} identical, {stats.modified} modified, {stats.added} added, {stats.removed} removed {deepScanProgress}</span>
        <span class="stat">{flatRows.length} rows visible</span>
      {/if}
    </div>
  </header>

  <div class="content">
    {#if loading}
      <div class="loading">
        <div class="spinner"></div>
        <p>{scanningProgress || "Scanning directories..."}</p>
        <p class="loading-hint">This may take a moment for large directories</p>
      </div>
    {:else if error}
      <div class="error">
        <p>Error: {error}</p>
        <button onclick={loadDirectories}>Retry</button>
      </div>
    {:else}
      <div class="panes">
        <div class="pane-header left">
          <span class="root-name" title={leftPath}>{getFileName(leftPath)}</span>
        </div>
        <div class="pane-header right">
          <span class="root-name" title={rightPath}>{getFileName(rightPath)}</span>
        </div>
      </div>
      <div class="tree-container" bind:clientHeight={containerHeight} onscroll={handleTreeScroll}>
        <div style="height: {flatRows.length * ROW_HEIGHT}px; position: relative;">
        {#snippet renderRow(entry: AlignedEntry, depth: number, absoluteIndex: number)}
          <div class="tree-row" style="position: absolute; top: {absoluteIndex * ROW_HEIGHT}px; left: 0; right: 0; height: {ROW_HEIGHT}px; display: flex;" style:--depth={depth}>
            <!-- Left side -->
            <button
              class="tree-cell left"
              class:is-dir={entry.isDir}
              class:is-selected={leftSelected === entry.relPath}
              class:is-missing={!entry.left}
              class:is-modified={entry.status === 'modified'}
              class:is-match={entry.status === 'match'}
              disabled={!entry.left && !entry.isDir}
              onclick={() => handleSelect(entry, 'left')}
              ondblclick={() => handleDoubleClick(entry)}
              oncontextmenu={(e) => handleContextMenu(e, entry, 'left')}
            >
              <span class="indent" style:width="{depth * 16}px"></span>
              {#if entry.isDir}
                <span class="expand-icon">{isExpanded(entry.relPath) ? '▼' : '▶'}</span>
                <span class="icon">📁</span>
              {:else}
                <span class="expand-icon"></span>
                <span class="icon">📄</span>
              {/if}
              {#if entry.left}
                <span class="name">{entry.name}</span>
                {#if !entry.isDir}
                  <span class="size">{formatSize(entry.left.size)}</span>
                {/if}
              {:else}
                <span class="name missing">{entry.name}</span>
              {/if}
            </button>

            <!-- Right side -->
            <button
              class="tree-cell right"
              class:is-dir={entry.isDir}
              class:is-selected={rightSelected === entry.relPath}
              class:is-missing={!entry.right}
              class:is-modified={entry.status === 'modified'}
              class:is-match={entry.status === 'match'}
              disabled={!entry.right && !entry.isDir}
              onclick={() => handleSelect(entry, 'right')}
              ondblclick={() => handleDoubleClick(entry)}
              oncontextmenu={(e) => handleContextMenu(e, entry, 'right')}
            >
              <span class="indent" style:width="{depth * 16}px"></span>
              {#if entry.isDir}
                <span class="expand-icon">{isExpanded(entry.relPath) ? '▼' : '▶'}</span>
                <span class="icon">📁</span>
              {:else}
                <span class="expand-icon"></span>
                <span class="icon">📄</span>
              {/if}
              {#if entry.right}
                <span class="name">{entry.name}</span>
                {#if !entry.isDir}
                  <span class="size">{formatSize(entry.right.size)}</span>
                {/if}
              {:else}
                <span class="name missing">{entry.name}</span>
              {/if}
            </button>
          </div>

        {/snippet}

        <!-- Render only visible rows -->
        {#each visibleRows() as row (row.entry.relPath + row.index)}
          {@render renderRow(row.entry, row.depth, row.index)}
        {/each}
        </div>
      </div>
    {/if}
  </div>

  {#if contextMenu}
    <!-- svelte-ignore a11y_no_static_element_interactions -->
    <div class="context-menu" style="left: {contextMenu?.x}px; top: {contextMenu?.y}px;" onclick={closeContextMenu} onkeydown={closeContextMenu}>
      <div class="context-menu-content" onclick={(e) => e.stopPropagation()}>
        {#if contextMenu.side === 'left' && contextMenu.entry.left && !contextMenu.entry.right}
          <button class="context-menu-item" onclick={() => copyFromContext('left')}>
            Copy to Right
          </button>
        {:else if contextMenu.side === 'right' && contextMenu.entry.right && !contextMenu.entry.left}
          <button class="context-menu-item" onclick={() => copyFromContext('right')}>
            Copy to Left
          </button>
        {/if}
        <button class="context-menu-item" onclick={() => contextMenu && handleDoubleClick(contextMenu.entry)}>
          Open Diff
        </button>
      </div>
    </div>
  {/if}
</div>

<style>
  .dir-compare {
    display: flex;
    flex-direction: column;
    height: 100%;
    overflow: hidden;
  }

  .header {
    display: flex;
    align-items: center;
    gap: var(--spacing-md);
    padding: var(--spacing-sm) var(--spacing-md);
    background: var(--color-bg-secondary);
    border-bottom: 1px solid var(--color-border);
    flex-shrink: 0;
  }

  .actions-left {
    display: flex;
    gap: var(--spacing-xs);
  }

  .action-btn {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 28px;
    height: 28px;
    border-radius: var(--radius-sm);
    color: var(--color-text-muted);
    transition: all var(--transition-fast);
  }

  .action-btn:hover {
    background: var(--color-bg-hover);
    color: var(--color-text-primary);
  }

  .filter-select {
    padding: 4px 8px;
    background: var(--color-bg-secondary);
    border: 1px solid var(--color-border);
    border-radius: var(--radius-sm);
    color: var(--color-text-primary);
    font-size: var(--font-size-sm);
  }

  .filter-select:focus {
    outline: 1px solid var(--color-accent-primary);
  }

  .ignore-toggle {
    display: flex;
    align-items: center;
    gap: var(--spacing-xs);
    font-size: var(--font-size-sm);
    color: var(--color-text-primary);
    cursor: pointer;
  }

  .ignore-toggle input {
    margin: 0;
  }



  .selection-info {
    flex: 1;
    display: flex;
    align-items: center;
    justify-content: center;
    gap: var(--spacing-md);
  }



  .selected-files {
    display: flex;
    align-items: center;
    gap: var(--spacing-sm);
    font-family: var(--font-mono);
    font-size: var(--font-size-sm);
  }

  .file-name {
    padding: 2px 8px;
    background: var(--color-bg-tertiary);
    border-radius: var(--radius-sm);
  }

  .arrow {
    color: var(--color-text-muted);
  }

  .diff-stats {
    display: flex;
    gap: var(--spacing-xs);
    font-family: var(--font-mono);
    font-size: var(--font-size-xs);
  }

  .diff-stats.loading {
    color: var(--color-text-muted);
  }

  .diff-stats .additions {
    color: var(--color-diff-insert-text);
  }

  .diff-stats .deletions {
    color: var(--color-diff-delete-text);
  }

  .diff-stats .identical {
    color: var(--color-text-muted);
  }

  .compare-btn, .copy-btn {
    padding: var(--spacing-xs) var(--spacing-md);
    background: var(--color-accent-primary);
    color: var(--color-bg-primary);
    border-radius: var(--radius-sm);
    font-size: var(--font-size-sm);
    font-weight: 500;
  }

  .compare-btn:hover, .copy-btn:hover {
    opacity: 0.9;
  }

  .stats {
    display: flex;
    gap: var(--spacing-sm);
  }

  .stat {
    font-size: var(--font-size-xs);
    color: var(--color-text-muted);
    padding: 2px 8px;
    background: var(--color-bg-tertiary);
    border-radius: var(--radius-sm);
  }

  .stat.warning {
    background: var(--color-diff-delete-bg);
    color: var(--color-diff-delete-text);
  }

  .stat.info {
    background: var(--color-accent-primary);
    color: var(--color-bg-primary);
  }

  .content {
    flex: 1;
    display: flex;
    flex-direction: column;
    min-height: 0;
    overflow: hidden;
  }

  .panes {
    display: flex;
    flex-shrink: 0;
    border-bottom: 1px solid var(--color-border);
  }

  .pane-header {
    flex: 1;
    padding: var(--spacing-sm) var(--spacing-md);
    background: var(--color-bg-secondary);
    font-size: var(--font-size-sm);
  }

  .pane-header.left {
    border-right: 1px solid var(--color-border);
  }

  .root-name {
    font-family: var(--font-mono);
    font-weight: 600;
  }

  .tree-container {
    flex: 1;
    overflow: auto;
    font-family: var(--font-mono);
    font-size: var(--font-size-sm);
  }

  .tree-row {
    display: flex;
  }

  .tree-cell {
    flex: 1;
    display: flex;
    align-items: center;
    padding: 4px 8px;
    border: none;
    background: none;
    text-align: left;
    cursor: pointer;
    transition: background var(--transition-fast);
    min-height: 28px;
  }

  .tree-cell.left {
    border-right: 1px solid var(--color-border);
  }

  .tree-cell:hover:not(:disabled) {
    background: var(--color-bg-hover);
  }

  .tree-cell.is-selected {
    background: var(--color-bg-tertiary);
    outline: 1px solid var(--color-border-active);
    outline-offset: -1px;
  }

  .tree-cell:disabled {
    cursor: default;
  }

  /* Status colors */
  .tree-cell.is-modified .name {
    color: var(--color-accent-secondary);
  }

  .tree-cell.is-missing {
    opacity: 0.5;
  }

  .tree-cell.is-missing .name {
    text-decoration: line-through;
    color: var(--color-text-muted);
  }

  .indent {
    flex-shrink: 0;
  }

  .expand-icon {
    width: 16px;
    font-size: 10px;
    color: var(--color-text-muted);
    flex-shrink: 0;
    text-align: center;
  }

  .icon {
    margin-right: 6px;
    font-size: 14px;
    flex-shrink: 0;
  }

  .name {
    flex: 1;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .name.missing {
    font-style: italic;
  }

  .is-dir .name {
    font-weight: 500;
  }

  .size {
    color: var(--color-text-muted);
    font-size: var(--font-size-xs);
    margin-left: var(--spacing-sm);
    flex-shrink: 0;
  }

  .loading, .error {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    flex: 1;
    gap: var(--spacing-md);
    color: var(--color-text-muted);
  }

  .loading-hint {
    font-size: var(--font-size-sm);
    opacity: 0.7;
    margin-top: var(--spacing-xs);
  }

  .spinner {
    width: 32px;
    height: 32px;
    border: 3px solid var(--color-border);
    border-top-color: var(--color-accent-primary);
    border-radius: 50%;
    animation: spin 1s linear infinite;
  }

  @keyframes spin {
    to { transform: rotate(360deg); }
  }

  .error {
    color: var(--color-diff-delete-text);
  }

  .error button {
    padding: var(--spacing-sm) var(--spacing-md);
    background: var(--color-bg-hover);
    border: 1px solid var(--color-border);
    border-radius: var(--radius-md);
    color: var(--color-text-primary);
  }

  .context-menu {
    position: fixed;
    z-index: 1000;
  }

  .context-menu-content {
    background: var(--color-bg-secondary);
    border: 1px solid var(--color-border);
    border-radius: var(--radius-md);
    box-shadow: 0 4px 12px rgba(0, 0, 0, 0.15);
    min-width: 120px;
  }

  .context-menu-item {
    display: block;
    width: 100%;
    padding: var(--spacing-sm) var(--spacing-md);
    background: none;
    border: none;
    text-align: left;
    color: var(--color-text-primary);
    cursor: pointer;
    font-size: var(--font-size-sm);
  }

  .context-menu-item:hover {
    background: var(--color-bg-hover);
  }

  .context-menu-item:first-child {
    border-radius: var(--radius-md) var(--radius-md) 0 0;
  }

  .context-menu-item:last-child {
    border-radius: 0 0 var(--radius-md) var(--radius-md);
  }

  .context-menu-item:only-child {
    border-radius: var(--radius-md);
  }
</style>
