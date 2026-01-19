<script lang="ts">
  import { onMount } from 'svelte';
  import { invoke } from '@tauri-apps/api/core';
  import type { FileContent, DiffLine, DiffResult } from '$lib/types';
  import DiffPane from './DiffPane.svelte';

  interface Props {
    base: FileContent;
    local: FileContent;
    remote: FileContent;
    onDirtyChange?: (dirty: boolean) => void;
    onSaveMerged?: (content: string) => Promise<void>;
  }

  let { base, local, remote, onDirtyChange, onSaveMerged }: Props = $props();

  // Diffs state - loaded asynchronously
  let baseLocalDiff = $state<DiffResult | null>(null);
  let baseRemoteDiff = $state<DiffResult | null>(null);
  let loadedFileKey = $state('');

  // Load diffs on mount and when files change
  onMount(() => {
    loadDiffs();
  });

  // Reactive file key to detect changes
  const fileKey = $derived(`${base.path}:${local.path}:${remote.path}`);

  // Reload when files change
  $effect(() => {
    if (fileKey !== loadedFileKey) {
      loadDiffs();
    }
  });

  async function loadDiffs() {
    const key = fileKey;
    try {
      const [blDiff, brDiff] = await Promise.all([
        invoke<DiffResult>('compute_diff', { left: base.content, right: local.content }),
        invoke<DiffResult>('compute_diff', { left: base.content, right: remote.content }),
      ]);
      baseLocalDiff = blDiff;
      baseRemoteDiff = brDiff;
      loadedFileKey = key;
    } catch (e) {
      console.error('Failed to compute diffs:', e);
    }
  }

  // Scroll sync state
  let baseScrollRef: HTMLDivElement | null = $state(null);
  let localScrollRef: HTMLDivElement | null = $state(null);
  let remoteScrollRef: HTMLDivElement | null = $state(null);
  let syncScroll = $state(true);
  let isScrolling = false;

  // Navigation state
  let currentHunkIndex = $state(-1);

  // Build pane lines for display
  interface PaneLine {
    lineNum: number | null;
    content: string;
    tag: 'equal' | 'insert' | 'delete' | 'empty';
  }

  function buildPaneLines(lines: DiffLine[]): { left: PaneLine[]; right: PaneLine[] } {
    const left: PaneLine[] = [];
    const right: PaneLine[] = [];

    for (const line of lines) {
      if (line.tag === 'equal') {
        left.push({ lineNum: line.old_index !== null ? line.old_index + 1 : null, content: line.value, tag: 'equal' });
        right.push({ lineNum: line.new_index !== null ? line.new_index + 1 : null, content: line.value, tag: 'equal' });
      } else if (line.tag === 'delete') {
        left.push({ lineNum: line.old_index !== null ? line.old_index + 1 : null, content: line.value, tag: 'delete' });
        right.push({ lineNum: null, content: '', tag: 'empty' });
      } else if (line.tag === 'insert') {
        left.push({ lineNum: null, content: '', tag: 'empty' });
        right.push({ lineNum: line.new_index !== null ? line.new_index + 1 : null, content: line.value, tag: 'insert' });
      }
    }

    return { left, right };
  }

  // Pane lines for each diff
  const baseLocalLines = $derived(baseLocalDiff ? buildPaneLines(baseLocalDiff.lines) : { left: [], right: [] });
  const baseRemoteLines = $derived(baseRemoteDiff ? buildPaneLines(baseRemoteDiff.lines) : { left: [], right: [] });

  // For three-way display: base | local | remote
  // Base pane shows base content (left side of base→local diff)
  // Local pane shows local content (right side of base→local diff)
  // Remote pane shows remote content (right side of base→remote diff)

  // Compute hunk positions for navigation (changes in either diff)
  interface HunkRange {
    start: number;
    end: number;
  }

  const hunkRanges = $derived.by(() => {
    const ranges: HunkRange[] = [];
    let inChange = false;
    let start = 0;
    const maxLen = Math.max(baseLocalLines.left.length, baseRemoteLines.left.length);

    for (let i = 0; i < maxLen; i++) {
      const localTag = baseLocalLines.right[i]?.tag ?? 'equal';
      const remoteTag = baseRemoteLines.right[i]?.tag ?? 'equal';
      const isChange = localTag !== 'equal' || remoteTag !== 'equal';

      if (isChange && !inChange) {
        start = i;
        inChange = true;
      } else if (!isChange && inChange) {
        ranges.push({ start, end: i });
        inChange = false;
      }
    }

    if (inChange) {
      ranges.push({ start, end: maxLen });
    }

    return ranges;
  });

  const hunkPositions = $derived(hunkRanges.map(r => r.start));

  function scrollToRow(rowIndex: number) {
    if (!baseScrollRef) return;
    const firstLine = baseScrollRef.querySelector('.line') as HTMLElement | null;
    const lineHeight = firstLine?.offsetHeight || 21;
    const targetScroll = rowIndex * lineHeight;

    baseScrollRef.scrollTop = targetScroll;
    if (syncScroll) {
      if (localScrollRef) localScrollRef.scrollTop = targetScroll;
      if (remoteScrollRef) remoteScrollRef.scrollTop = targetScroll;
    }
  }

  function goToHunk(index: number) {
    if (index < 0 || index >= hunkPositions.length) return;
    currentHunkIndex = index;
    scrollToRow(hunkPositions[index]);
  }

  function nextHunk() {
    if (hunkPositions.length === 0) return;
    const next = currentHunkIndex < hunkPositions.length - 1 ? currentHunkIndex + 1 : 0;
    goToHunk(next);
  }

  function prevHunk() {
    if (hunkPositions.length === 0) return;
    const prev = currentHunkIndex > 0 ? currentHunkIndex - 1 : hunkPositions.length - 1;
    goToHunk(prev);
  }

  // Keyboard navigation
  onMount(() => {
    function handleKeydown(e: KeyboardEvent) {
      if (e.target instanceof HTMLInputElement || e.target instanceof HTMLTextAreaElement) return;
      if ((e.target as HTMLElement)?.isContentEditable) return;

      if (e.key === 'n' || e.key === 'N') {
        e.preventDefault();
        nextHunk();
      } else if (e.key === 'p' || e.key === 'P') {
        e.preventDefault();
        prevHunk();
      }
    }

    window.addEventListener('keydown', handleKeydown);
    return () => window.removeEventListener('keydown', handleKeydown);
  });

  function handleScroll(source: 'base' | 'local' | 'remote') {
    if (!syncScroll || isScrolling) return;

    const sourceEl = source === 'base' ? baseScrollRef : source === 'local' ? localScrollRef : remoteScrollRef;
    if (!sourceEl) return;

    isScrolling = true;
    const scrollTop = sourceEl.scrollTop;
    const scrollLeft = sourceEl.scrollLeft;

    if (source !== 'base' && baseScrollRef) {
      baseScrollRef.scrollTop = scrollTop;
      baseScrollRef.scrollLeft = scrollLeft;
    }
    if (source !== 'local' && localScrollRef) {
      localScrollRef.scrollTop = scrollTop;
      localScrollRef.scrollLeft = scrollLeft;
    }
    if (source !== 'remote' && remoteScrollRef) {
      remoteScrollRef.scrollTop = scrollTop;
      remoteScrollRef.scrollLeft = scrollLeft;
    }

    requestAnimationFrame(() => {
      isScrolling = false;
    });
  }

  const lineHeight = 21;
</script>

<div class="three-way-view">
  {#if baseLocalDiff && baseRemoteDiff}
  <div class="diff-container">
    <DiffPane
      file={local}
      lines={baseLocalLines.right}
      side="left"
      bind:scrollRef={localScrollRef}
      onscroll={() => handleScroll('local')}
    />
    <div class="pane-divider"></div>
    <DiffPane
      file={base}
      lines={baseLocalLines.left}
      side="left"
      bind:scrollRef={baseScrollRef}
      onscroll={() => handleScroll('base')}
    />
    <div class="pane-divider"></div>
    <DiffPane
      file={remote}
      lines={baseRemoteLines.right}
      side="right"
      bind:scrollRef={remoteScrollRef}
      onscroll={() => handleScroll('remote')}
    />
  </div>
  {:else}
  <div class="loading">Loading diffs...</div>
  {/if}

  <div class="diff-stats">
    <span class="stat">Local: <span class="additions">+{baseLocalDiff?.stats.additions ?? 0}</span> <span class="deletions">-{baseLocalDiff?.stats.deletions ?? 0}</span></span>
    <span class="stat">Remote: <span class="additions">+{baseRemoteDiff?.stats.additions ?? 0}</span> <span class="deletions">-{baseRemoteDiff?.stats.deletions ?? 0}</span></span>

    {#if hunkPositions.length > 0}
      <div class="nav-controls">
        <button
          class="nav-button"
          onclick={prevHunk}
          title="Previous change (P)"
        >
          <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <polyline points="15 18 9 12 15 6"></polyline>
          </svg>
        </button>
        <span class="nav-indicator">
          {currentHunkIndex >= 0 ? currentHunkIndex + 1 : '-'} / {hunkPositions.length}
        </span>
        <button
          class="nav-button"
          onclick={nextHunk}
          title="Next change (N)"
        >
          <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <polyline points="9 18 15 12 9 6"></polyline>
          </svg>
        </button>
      </div>
    {/if}

    <button
      class="sync-toggle"
      class:active={syncScroll}
      onclick={() => (syncScroll = !syncScroll)}
      title={syncScroll ? 'Disable sync scroll' : 'Enable sync scroll'}
    >
      <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
        <polyline points="17 1 21 5 17 9"></polyline>
        <path d="M3 11V9a4 4 0 0 1 4-4h14"></path>
        <polyline points="7 23 3 19 7 15"></polyline>
        <path d="M21 13v2a4 4 0 0 1-4 4H3"></path>
      </svg>
      Sync
    </button>
  </div>
</div>

<style>
  .three-way-view {
    display: flex;
    flex-direction: column;
    flex: 1;
    min-height: 0;
  }

  .diff-container {
    display: flex;
    flex: 1;
    min-height: 0;
    border: 1px solid var(--color-border);
    border-radius: var(--radius-md);
    overflow: hidden;
  }

  .pane-divider {
    width: 1px;
    background: var(--color-border);
    flex-shrink: 0;
  }

  .loading {
    flex: 1;
    display: flex;
    align-items: center;
    justify-content: center;
    color: var(--color-text-muted);
  }

  .diff-stats {
    display: flex;
    gap: var(--spacing-lg);
    padding: var(--spacing-sm) var(--spacing-md);
    font-size: var(--font-size-sm);
    color: var(--color-text-muted);
  }

  .stat {
    font-family: var(--font-mono);
  }

  .additions {
    color: var(--color-diff-insert-text);
  }

  .deletions {
    color: var(--color-diff-delete-text);
  }

  .nav-controls {
    display: flex;
    align-items: center;
    gap: var(--spacing-xs);
    margin-left: var(--spacing-lg);
  }

  .nav-button {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 24px;
    height: 24px;
    border-radius: var(--radius-sm);
    color: var(--color-text-muted);
    background: transparent;
    border: 1px solid var(--color-border);
    transition: all var(--transition-fast);
  }

  .nav-button:hover {
    background: var(--color-bg-hover);
    color: var(--color-text-primary);
    border-color: var(--color-border-hover);
  }

  .nav-indicator {
    font-family: var(--font-mono);
    font-size: var(--font-size-xs);
    color: var(--color-text-secondary);
    min-width: 4em;
    text-align: center;
  }

  .sync-toggle {
    display: flex;
    align-items: center;
    gap: var(--spacing-xs);
    margin-left: auto;
    padding: var(--spacing-xs) var(--spacing-sm);
    border-radius: var(--radius-sm);
    font-size: var(--font-size-xs);
    color: var(--color-text-muted);
    background: transparent;
    border: 1px solid var(--color-border);
    transition: all var(--transition-fast);
  }

  .sync-toggle:hover {
    background: var(--color-bg-hover);
    color: var(--color-text-secondary);
  }

  .sync-toggle.active {
    background: var(--color-bg-tertiary);
    color: var(--color-accent-primary);
    border-color: var(--color-accent-primary);
  }
</style>
