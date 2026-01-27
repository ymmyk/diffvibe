<script lang="ts">
  import { invoke } from '@tauri-apps/api/core';
  import { listen } from '@tauri-apps/api/event';
  import { confirm } from '@tauri-apps/plugin-dialog';
  import { onMount, onDestroy, tick } from 'svelte';
  import type { AlignedEntry, AlignedScanResult, ScanProgress, DiffStats } from '$lib/types';
  import { tabStore } from '$lib/stores/tabs.svelte';

  interface Props {
    leftPath: string;
    rightPath: string;
    tabId: string;
  }

  let { leftPath, rightPath, tabId }: Props = $props();

  // Get tab from store
  const tab = $derived(tabStore.getTab(tabId));
  
  // Scan result from tab store (preserved across tab switches)
  let scanResult = $derived(tab?.scanResult ?? null);
  
  // Get directory state from tab store
  let dirState = $derived(tabStore.getDirectoryState(tabId));

  // Local UI state
  let error = $state<string | null>(null);
  let loading = $state(false);
  let progress = $state<ScanProgress | null>(null);

  // Modifier key tracking for independent selection
  let modifierHeld = $state(false);

  // Virtual scrolling
  const ROW_HEIGHT = 28;
  const OVERSCAN = 10;
  let scrollTop = $state(0);
  let containerHeight = $state(600);

  // Flat list of visible rows
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

  // Ignore patterns (hardcoded for now)
  const DEFAULT_IGNORE_PATTERNS = [
    'node_modules/', '.git/', '.next/', '_next/',
    'dist/', 'build/', 'out/', '.nuxt/',
    '.output/', '.vercel/', '.cache/', 'target/',
    'Cargo.lock', '__pycache__/', '*.pyc',
    '*.log', '.DS_Store', 'Thumbs.db',
    '.env*', '*.tmp', '*.swp', '*.swo'
  ];

  let unlistenProgress: (() => void) | null = null;

  // Load directories with new backend
  async function loadDirectories() {
    loading = true;
    error = null;
    progress = { phase: 'starting', files: 0, message: 'Starting scan...' };

    await tick();

    try {
      // Set up progress listener
      unlistenProgress = await listen<ScanProgress>('directory-scan-progress', (event) => {
        progress = event.payload;
      });

      // Use ignore patterns if showIgnored is false
      const patternsToIgnore = dirState.showIgnored ? [] : DEFAULT_IGNORE_PATTERNS;

      // Call new backend command
      const result = await invoke<AlignedScanResult>('compare_directories_async', {
        window: window.__TAURI__?.window,
        leftPath,
        rightPath,
        ignorePatterns: patternsToIgnore,
      });

      // Store result in tab
      tabStore.setScanResult(tabId, result);

      // Auto-expand all directories by default
      await tick();
      expandAll(result.entries);
      
    } catch (e) {
      error = e instanceof Error ? e.message : String(e);
    } finally {
      loading = false;
      progress = null;
      if (unlistenProgress) {
        unlistenProgress();
        unlistenProgress = null;
      }
    }
  }

  // Flatten tree into list of visible rows (memoized with $derived)
  $effect(() => {
    if (!scanResult) {
      flatRows = [];
      return;
    }

    const rows: FlatRow[] = [];

    function flatten(entries: AlignedEntry[], depth: number) {
      for (const entry of entries) {
        // Apply filter
        if (!shouldShowEntry(entry)) {
          continue;
        }

        rows.push({ entry, depth, index: rows.length });
        
        // If directory is expanded, add children
        if (entry.is_dir && isExpanded(entry.rel_path) && entry.children.length > 0) {
          flatten(entry.children, depth + 1);
        }
      }
    }

    flatten(scanResult.entries, 0);
    flatRows = rows;
  });

  // Calculate visible rows based on scroll position
  let visibleRows = $derived(() => {
    const startIndex = Math.max(0, Math.floor(scrollTop / ROW_HEIGHT) - OVERSCAN);
    const endIndex = Math.min(
      flatRows.length,
      Math.ceil((scrollTop + containerHeight) / ROW_HEIGHT) + OVERSCAN
    );
    
    return flatRows.slice(startIndex, endIndex);
  });

  function isExpanded(path: string): boolean {
    return dirState.expanded[path] ?? true; // Default expanded
  }

  function toggleExpand(entry: AlignedEntry) {
    const newExpanded = { ...dirState.expanded };
    newExpanded[entry.rel_path] = !isExpanded(entry.rel_path);
    tabStore.setDirectoryState(tabId, { expanded: newExpanded });
  }

  function shouldShowEntry(entry: AlignedEntry): boolean {
    const filter = dirState.filter;
    
    if (filter === 'all') return true;
    if (filter === 'changed') {
      return entry.status === 'modified' || entry.status === 'leftonly' || entry.status === 'rightonly';
    }
    if (filter === 'identical') {
      return entry.status === 'match';
    }
    return true;
  }

  function expandAll(entries: AlignedEntry[]) {
    const newExpanded: Record<string, boolean> = {};
    
    function expandRecursive(entries: AlignedEntry[]) {
      for (const entry of entries) {
        if (entry.is_dir) {
          newExpanded[entry.rel_path] = true;
          expandRecursive(entry.children);
        }
      }
    }
    
    expandRecursive(entries);
    tabStore.setDirectoryState(tabId, { expanded: newExpanded });
  }

  function collapseAll() {
    if (!scanResult) return;
    
    const newExpanded: Record<string, boolean> = {};
    
    function collapseRecursive(entries: AlignedEntry[]) {
      for (const entry of entries) {
        if (entry.is_dir) {
          newExpanded[entry.rel_path] = false;
          collapseRecursive(entry.children);
        }
      }
    }
    
    collapseRecursive(scanResult.entries);
    tabStore.setDirectoryState(tabId, { expanded: newExpanded });
  }

  function handleTreeScroll(e: Event) {
    const target = e.target as HTMLElement;
    scrollTop = target.scrollTop;
    tabStore.setDirectoryState(tabId, { scrollTop: target.scrollTop });
  }

  function handleSelect(entry: AlignedEntry, side: 'left' | 'right') {
    if (entry.is_dir) {
      toggleExpand(entry);
      return;
    }

    // Check if file exists on this side
    const exists = side === 'left' ? entry.left_size !== null : entry.right_size !== null;
    if (!exists) return;

    let newLeftSelected = dirState.leftSelected;
    let newRightSelected = dirState.rightSelected;

    if (modifierHeld) {
      // Independent selection
      if (side === 'left') {
        newLeftSelected = entry.rel_path;
      } else {
        newRightSelected = entry.rel_path;
      }
    } else {
      // Linked selection
      newLeftSelected = entry.rel_path;
      newRightSelected = entry.rel_path;
    }

    tabStore.setDirectoryState(tabId, { 
      leftSelected: newLeftSelected,
      rightSelected: newRightSelected 
    });

    // Load diff stats
    if (newLeftSelected && newRightSelected) {
      loadDiffStats(newLeftSelected, newRightSelected);
    } else {
      selectedStats = null;
    }
  }

  async function loadDiffStats(leftRel: string, rightRel: string) {
    loadingStats = true;
    selectedStats = null;
    try {
      const leftFile = `${leftPath}/${leftRel}`;
      const rightFile = `${rightPath}/${rightRel}`;
      selectedStats = await invoke<DiffStats>('get_diff_stats', { 
        leftPath: leftFile, 
        rightPath: rightFile 
      });
    } catch (e) {
      console.error('Failed to load diff stats:', e);
    } finally {
      loadingStats = false;
    }
  }

  function handleDoubleClick(entry: AlignedEntry) {
    if (entry.is_dir) return;

    const leftFile = `${leftPath}/${entry.rel_path}`;
    const rightFile = `${rightPath}/${entry.rel_path}`;
    tabStore.openCompare(leftFile, rightFile, 'file', undefined, tabId);
  }

  function compareSelected() {
    if (!dirState.leftSelected || !dirState.rightSelected) return;
    const leftFile = `${leftPath}/${dirState.leftSelected}`;
    const rightFile = `${rightPath}/${dirState.rightSelected}`;
    tabStore.openCompare(leftFile, rightFile, 'file', undefined, tabId);
  }

  async function copyToRight() {
    if (!dirState.leftSelected) return;
    const leftPathFull = `${leftPath}/${dirState.leftSelected}`;
    const rightPathFull = `${rightPath}/${dirState.leftSelected}`;

    try {
      const destExists = await invoke('file_exists', { path: rightPathFull });
      if (destExists) {
        const confirmed = await confirm(`Path already exists. Overwrite ${rightPathFull}?`);
        if (!confirmed) return;
      }

      const isDir = await invoke('is_directory', { path: leftPathFull });
      if (isDir) {
        await invoke('copy_dir', { fromPath: leftPathFull, toPath: rightPathFull });
      } else {
        await invoke('copy_file', { fromPath: leftPathFull, toPath: rightPathFull });
      }

      // Refresh
      await loadDirectories();
    } catch (e) {
      console.error('Failed to copy to right:', e);
      alert(`Failed to copy: ${e}`);
    }
  }

  async function copyToLeft() {
    if (!dirState.rightSelected) return;
    const rightPathFull = `${rightPath}/${dirState.rightSelected}`;
    const leftPathFull = `${leftPath}/${dirState.rightSelected}`;

    try {
      const destExists = await invoke('file_exists', { path: leftPathFull });
      if (destExists) {
        const confirmed = await confirm(`Path already exists. Overwrite ${leftPathFull}?`);
        if (!confirmed) return;
      }

      const isDir = await invoke('is_directory', { path: rightPathFull });
      if (isDir) {
        await invoke('copy_dir', { fromPath: rightPathFull, toPath: leftPathFull });
      } else {
        await invoke('copy_file', { fromPath: rightPathFull, toPath: leftPathFull });
      }

      // Refresh
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

  async function copyFromContext(side: 'left' | 'right') {
    if (!contextMenu) return;
    const { entry } = contextMenu;

    try {
      let fromPath: string;
      let toPath: string;

      if (side === 'left') {
        fromPath = `${leftPath}/${entry.rel_path}`;
        toPath = `${rightPath}/${entry.rel_path}`;
      } else {
        fromPath = `${rightPath}/${entry.rel_path}`;
        toPath = `${leftPath}/${entry.rel_path}`;
      }

      const destExists = await invoke('file_exists', { path: toPath });
      if (destExists) {
        const confirmed = await confirm(`Path already exists. Overwrite ${toPath}?`);
        if (!confirmed) {
          closeContextMenu();
          return;
        }
      }

      const isDir = await invoke('is_directory', { path: fromPath });
      if (isDir) {
        await invoke('copy_dir', { fromPath, toPath });
      } else {
        await invoke('copy_file', { fromPath, toPath });
      }

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

  function updateFilter(newFilter: 'all' | 'changed' | 'identical') {
    tabStore.setDirectoryState(tabId, { filter: newFilter });
  }

  function toggleShowIgnored(checked: boolean) {
    tabStore.setDirectoryState(tabId, { showIgnored: checked });
    // Reload with new ignore settings
    loadDirectories();
  }

  onMount(() => {
    // Restore scroll position
    const scrollContainer = document.querySelector('.tree-container');
    if (scrollContainer && dirState.scrollTop) {
      scrollContainer.scrollTop = dirState.scrollTop;
    }

    // Load if no cached result
    if (!scanResult) {
      loadDirectories();
    }

    // Keyboard handlers
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
      if (unlistenProgress) {
        unlistenProgress();
      }
    };
  });

  onDestroy(() => {
    if (unlistenProgress) {
      unlistenProgress();
    }
  });
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

      <button class="action-btn" onclick={() => scanResult && expandAll(scanResult.entries)} title="Expand All">
        <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <polyline points="6 9 12 15 18 9"></polyline>
        </svg>
      </button>

      <button class="action-btn" onclick={collapseAll} title="Collapse All">
        <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <polyline points="18 15 12 9 6 15"></polyline>
        </svg>
      </button>

      <select class="filter-select" value={dirState.filter} onchange={(e) => updateFilter(e.currentTarget.value as any)}>
        <option value="all">All files</option>
        <option value="changed">Changed only</option>
        <option value="identical">Identical only</option>
      </select>

      <label class="ignore-toggle">
        <input type="checkbox" checked={dirState.showIgnored} onchange={(e) => toggleShowIgnored(e.currentTarget.checked)} />
        <span>Show ignored files</span>
      </label>
    </div>

    <div class="selection-info">
      {#if dirState.leftSelected && dirState.rightSelected}
        <span class="selected-files">
          <span class="file-name">{dirState.leftSelected.split('/').pop()}</span>
          <span class="arrow">↔</span>
          <span class="file-name">{dirState.rightSelected.split('/').pop()}</span>
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
        <button class="compare-btn" onclick={compareSelected}>Compare</button>
      {:else if dirState.leftSelected && !dirState.rightSelected}
        <span class="selected-files">
          <span class="file-name">{dirState.leftSelected.split('/').pop()}</span>
          <span class="arrow">→</span>
          <span class="file-name missing">right side</span>
        </span>
        <button class="copy-btn" onclick={copyToRight}>Copy to Right</button>
      {:else if dirState.rightSelected && !dirState.leftSelected}
        <span class="selected-files">
          <span class="file-name missing">left side</span>
          <span class="arrow">←</span>
          <span class="file-name">{dirState.rightSelected.split('/').pop()}</span>
        </span>
        <button class="copy-btn" onclick={copyToLeft}>Copy to Left</button>
      {/if}
    </div>

    <div class="stats">
      {#if scanResult}
        <span class="stat">{scanResult.stats.total_files} files</span>
        <span class="stat">{scanResult.stats.identical} identical, {scanResult.stats.modified} modified, {scanResult.stats.right_only} added, {scanResult.stats.left_only} removed</span>
        <span class="stat">{flatRows.length} rows visible</span>
      {/if}
    </div>
  </header>

  <div class="content">
    {#if loading}
      <div class="loading">
        <div class="spinner"></div>
        {#if progress}
          <p>{progress.message}</p>
          {#if progress.files > 0}
            <p class="loading-hint">{progress.files} files scanned</p>
          {:else}
            <p class="loading-hint">This may take a moment for large directories</p>
          {/if}
        {:else}
          <p>Loading directories...</p>
        {/if}
      </div>
    {:else if error}
      <div class="error">
        <p>Error: {error}</p>
        <button onclick={loadDirectories}>Retry</button>
      </div>
    {:else if scanResult}
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
              class:is-dir={entry.is_dir}
              class:is-selected={dirState.leftSelected === entry.rel_path}
              class:is-missing={entry.left_size === null}
              class:is-modified={entry.status === 'modified'}
              class:is-match={entry.status === 'match'}
              disabled={entry.left_size === null && !entry.is_dir}
              onclick={() => handleSelect(entry, 'left')}
              ondblclick={() => handleDoubleClick(entry)}
              oncontextmenu={(e) => handleContextMenu(e, entry, 'left')}
            >
              <span class="indent" style:width="{depth * 16}px"></span>
              {#if entry.is_dir}
                <span class="expand-icon">{isExpanded(entry.rel_path) ? '▼' : '▶'}</span>
                <span class="icon">📁</span>
              {:else}
                <span class="expand-icon"></span>
                <span class="icon">📄</span>
              {/if}
              {#if entry.left_size !== null}
                <span class="name">{entry.name}</span>
                {#if !entry.is_dir && entry.left_size !== null}
                  <span class="size">{formatSize(entry.left_size)}</span>
                {/if}
              {:else}
                <span class="name missing">{entry.name}</span>
              {/if}
            </button>

            <!-- Right side -->
            <button
              class="tree-cell right"
              class:is-dir={entry.is_dir}
              class:is-selected={dirState.rightSelected === entry.rel_path}
              class:is-missing={entry.right_size === null}
              class:is-modified={entry.status === 'modified'}
              class:is-match={entry.status === 'match'}
              disabled={entry.right_size === null && !entry.is_dir}
              onclick={() => handleSelect(entry, 'right')}
              ondblclick={() => handleDoubleClick(entry)}
              oncontextmenu={(e) => handleContextMenu(e, entry, 'right')}
            >
              <span class="indent" style:width="{depth * 16}px"></span>
              {#if entry.is_dir}
                <span class="expand-icon">{isExpanded(entry.rel_path) ? '▼' : '▶'}</span>
                <span class="icon">📁</span>
              {:else}
                <span class="expand-icon"></span>
                <span class="icon">📄</span>
              {/if}
              {#if entry.right_size !== null}
                <span class="name">{entry.name}</span>
                {#if !entry.is_dir && entry.right_size !== null}
                  <span class="size">{formatSize(entry.right_size)}</span>
                {/if}
              {:else}
                <span class="name missing">{entry.name}</span>
              {/if}
            </button>
          </div>
        {/snippet}

        <!-- Render only visible rows -->
        {#each visibleRows() as row (row.entry.rel_path + row.index)}
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
        {#if contextMenu.side === 'left' && contextMenu.entry.left_size !== null && contextMenu.entry.right_size === null}
          <button class="context-menu-item" onclick={() => copyFromContext('left')}>
            Copy to Right
          </button>
        {:else if contextMenu.side === 'right' && contextMenu.entry.right_size !== null && contextMenu.entry.left_size === null}
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
