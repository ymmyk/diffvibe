<script lang="ts">
  import { invoke } from '@tauri-apps/api/core';
  import { onMount } from 'svelte';
  import type { ScanResult, DirEntry } from '$lib/types';
  import { tabStore } from '$lib/stores/tabs.svelte';

  interface Props {
    leftPath: string;
    rightPath: string;
  }

  let { leftPath, rightPath }: Props = $props();

  let leftResult = $state<ScanResult | null>(null);
  let rightResult = $state<ScanResult | null>(null);
  let error = $state<string | null>(null);
  let loading = $state(true);

  // Selected files
  let leftSelected = $state<string | null>(null);
  let rightSelected = $state<string | null>(null);

  // Expanded directories (shared between both sides)
  let expanded = $state<Record<string, boolean>>({});

  // Track if modifier key is held for independent selection
  let modifierHeld = $state(false);

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
    try {
      const [left, right] = await Promise.all([
        invoke<ScanResult>('scan_directory', { path: leftPath }),
        invoke<ScanResult>('scan_directory', { path: rightPath }),
      ]);
      leftResult = left;
      rightResult = right;
    } catch (e) {
      error = e instanceof Error ? e.message : String(e);
    } finally {
      loading = false;
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

  function toggleExpand(path: string) {
    expanded[path] = !expanded[path];
  }

  function isExpanded(path: string): boolean {
    return expanded[path] ?? true;
  }

  function handleSelect(entry: AlignedEntry, side: 'left' | 'right') {
    if (entry.isDir) {
      toggleExpand(entry.relPath);
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
  }

  function handleDoubleClick(entry: AlignedEntry) {
    if (entry.isDir) return;

    // Double-click compares matching files (same path on both sides)
    const leftFile = `${leftPath}/${entry.relPath}`;
    const rightFile = `${rightPath}/${entry.relPath}`;
    tabStore.openCompare(leftFile, rightFile, 'file');
  }

  function compareSelected() {
    if (!leftSelected || !rightSelected) return;
    const leftFile = `${leftPath}/${leftSelected}`;
    const rightFile = `${rightPath}/${rightSelected}`;
    tabStore.openCompare(leftFile, rightFile, 'file');
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
      <button class="action-btn" onclick={loadDirectories} title="Refresh">
        <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <polyline points="23 4 23 10 17 10"></polyline>
          <polyline points="1 20 1 14 7 14"></polyline>
          <path d="M3.51 9a9 9 0 0 1 14.85-3.36L23 10M1 14l4.64 4.36A9 9 0 0 0 20.49 15"></path>
        </svg>
      </button>
    </div>

    <div class="selection-info">
      {#if leftSelected && rightSelected}
        <span class="selected-files">
          <span class="file-name">{leftSelected.split('/').pop()}</span>
          <span class="arrow">↔</span>
          <span class="file-name">{rightSelected.split('/').pop()}</span>
        </span>
        <button class="compare-btn" onclick={compareSelected}>
          Compare
        </button>
      {:else}
        <span class="hint">
          Click to select matching files, hold {navigator.platform.includes('Mac') ? '⌘' : 'Ctrl'} to select different files
        </span>
      {/if}
    </div>

    <div class="stats">
      {#if leftResult && rightResult}
        <span class="stat">{leftResult.file_count} / {rightResult.file_count} files</span>
      {/if}
    </div>
  </header>

  <div class="content">
    {#if loading}
      <div class="loading">
        <div class="spinner"></div>
        <p>Scanning directories...</p>
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
      <div class="tree-container">
        {#snippet renderRow(entry: AlignedEntry, depth: number)}
          <div class="tree-row" style:--depth={depth}>
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

          {#if entry.isDir && isExpanded(entry.relPath)}
            {#each entry.children as child (child.relPath)}
              {@render renderRow(child, depth + 1)}
            {/each}
          {/if}
        {/snippet}

        {#each alignedEntries as entry (entry.relPath)}
          {@render renderRow(entry, 0)}
        {/each}
      </div>
    {/if}
  </div>
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

  .selection-info {
    flex: 1;
    display: flex;
    align-items: center;
    justify-content: center;
    gap: var(--spacing-md);
  }

  .hint {
    color: var(--color-text-muted);
    font-size: var(--font-size-sm);
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

  .compare-btn {
    padding: var(--spacing-xs) var(--spacing-md);
    background: var(--color-accent-primary);
    color: var(--color-bg-primary);
    border-radius: var(--radius-sm);
    font-size: var(--font-size-sm);
    font-weight: 500;
  }

  .compare-btn:hover {
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
</style>
