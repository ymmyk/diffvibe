<script lang="ts">
  import { untrack } from 'svelte';
  import { invoke } from '@tauri-apps/api/core';
  import type { FileContent, MergeResult, DiffResult } from '$lib/types';
  import DiffPane from './DiffPane.svelte';

  interface Props {
    base: FileContent;
    local: FileContent;
    remote: FileContent;
    onDirtyChange?: (dirty: boolean) => void;
    onSaveMerged?: (content: string) => Promise<void>;
  }

  let { base, local, remote, onDirtyChange, onSaveMerged }: Props = $props();

  // Diff results - base→local and base→remote
  let baseToLocalDiff = $state<DiffResult | null>(null);
  let baseToRemoteDiff = $state<DiffResult | null>(null);
  let mergeResult = $state<MergeResult | null>(null);
  let loadedFileKey = $state('');
  let isLoading = $state(false);

  // Editable merged content
  let mergedContent = $state('');
  let originalMergedContent = $state('');

  // Reactive file key to detect changes
  const fileKey = $derived(`${base.path}:${local.path}:${remote.path}`);

  // Load on mount and when file key changes
  $effect(() => {
    const key = fileKey;
    untrack(() => {
      if (key !== loadedFileKey && !isLoading) {
        loadDiffs(key);
      }
    });
  });

  async function loadDiffs(key: string) {
    isLoading = true;
    try {
      const [blDiff, brDiff, merge] = await Promise.all([
        invoke<DiffResult>('compute_diff', { left: base.content, right: local.content }),
        invoke<DiffResult>('compute_diff', { left: base.content, right: remote.content }),
        invoke<MergeResult>('compute_three_way_diff', {
          base: base.content,
          local: local.content,
          remote: remote.content,
        })
      ]);
      baseToLocalDiff = blDiff;
      baseToRemoteDiff = brDiff;
      mergeResult = merge;
      mergedContent = merge.merged_content;
      originalMergedContent = merge.merged_content;
      loadedFileKey = key;
      onDirtyChange?.(false);
    } catch (e) {
      console.error('Failed to compute diffs:', e);
    } finally {
      isLoading = false;
    }
  }

  // Dirty state
  const isDirty = $derived(mergedContent !== originalMergedContent);

  // Notify parent of dirty state
  let prevDirty = false;
  $effect(() => {
    const dirty = isDirty;
    if (dirty !== prevDirty) {
      prevDirty = dirty;
      untrack(() => onDirtyChange?.(dirty));
    }
  });

  // Scroll sync
  let localScrollRef: HTMLDivElement | null = $state(null);
  let baseScrollRef: HTMLDivElement | null = $state(null);
  let remoteScrollRef: HTMLDivElement | null = $state(null);
  let syncScroll = $state(true);
  let isScrolling = false;

  // Navigation
  let currentConflictIndex = $state(-1);

  // Build pane lines for display (same pattern as DiffView)
  interface PaneLine {
    lineNum: number | null;
    content: string;
    tag: 'equal' | 'insert' | 'delete' | 'empty';
  }

  function buildPaneLines(diff: DiffResult | null): { left: PaneLine[]; right: PaneLine[] } {
    if (!diff) return { left: [], right: [] };

    const left: PaneLine[] = [];
    const right: PaneLine[] = [];

    for (const line of diff.lines) {
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

  // Build display lines for each pane
  // For local: show base on left (with deletions), local on right (with insertions)
  // For remote: show base on left (with deletions), remote on right (with insertions)
  const localPaneLines = $derived(buildPaneLines(baseToLocalDiff));
  const remotePaneLines = $derived(buildPaneLines(baseToRemoteDiff));

  // Count conflicts in merged content
  const conflictCount = $derived(mergeResult?.conflict_count ?? 0);

  // Find conflict positions in merged content
  const conflictPositions = $derived.by(() => {
    const positions: number[] = [];
    const lines = mergedContent.split('\n');
    for (let i = 0; i < lines.length; i++) {
      if (lines[i] === '<<<<<<< LOCAL') {
        positions.push(i);
      }
    }
    return positions;
  });

  // Resolve a conflict by choosing local or remote
  function resolveConflict(index: number, choice: 'local' | 'remote') {
    const lines = mergedContent.split('\n');
    let conflictNum = 0;
    let i = 0;

    while (i < lines.length) {
      if (lines[i] === '<<<<<<< LOCAL') {
        if (conflictNum === index) {
          // Find bounds
          let sepIdx = i + 1;
          while (sepIdx < lines.length && lines[sepIdx] !== '=======') sepIdx++;
          let endIdx = sepIdx + 1;
          while (endIdx < lines.length && lines[endIdx] !== '>>>>>>> REMOTE') endIdx++;

          const localContent = lines.slice(i + 1, sepIdx);
          const remoteContent = lines.slice(sepIdx + 1, endIdx);
          const chosen = choice === 'local' ? localContent : remoteContent;

          lines.splice(i, endIdx - i + 1, ...chosen);
          mergedContent = lines.join('\n');
          return;
        }
        conflictNum++;
      }
      i++;
    }
  }

  // Navigation
  function scrollToConflict(lineIndex: number) {
    // Scroll merged pane to conflict
    const mergedEl = document.querySelector('.merged-content');
    if (mergedEl) {
      const lineHeight = 21;
      mergedEl.scrollTop = lineIndex * lineHeight;
    }
  }

  function goToConflict(index: number) {
    if (index < 0 || index >= conflictPositions.length) return;
    currentConflictIndex = index;
    scrollToConflict(conflictPositions[index]);
  }

  function nextConflict() {
    if (conflictPositions.length === 0) return;
    const next = currentConflictIndex < conflictPositions.length - 1 ? currentConflictIndex + 1 : 0;
    goToConflict(next);
  }

  function prevConflict() {
    if (conflictPositions.length === 0) return;
    const prev = currentConflictIndex > 0 ? currentConflictIndex - 1 : conflictPositions.length - 1;
    goToConflict(prev);
  }

  // Scroll sync handler
  function handleScroll(source: 'local' | 'base' | 'remote') {
    if (!syncScroll || isScrolling) return;

    const sourceEl = source === 'local' ? localScrollRef : source === 'base' ? baseScrollRef : remoteScrollRef;
    if (!sourceEl) return;

    isScrolling = true;
    const scrollTop = sourceEl.scrollTop;
    const scrollLeft = sourceEl.scrollLeft;

    if (source !== 'local' && localScrollRef) {
      localScrollRef.scrollTop = scrollTop;
      localScrollRef.scrollLeft = scrollLeft;
    }
    if (source !== 'base' && baseScrollRef) {
      baseScrollRef.scrollTop = scrollTop;
      baseScrollRef.scrollLeft = scrollLeft;
    }
    if (source !== 'remote' && remoteScrollRef) {
      remoteScrollRef.scrollTop = scrollTop;
      remoteScrollRef.scrollLeft = scrollLeft;
    }

    requestAnimationFrame(() => {
      isScrolling = false;
    });
  }

  // Save merged content
  async function saveMerged() {
    if (onSaveMerged && conflictPositions.length === 0) {
      await onSaveMerged(mergedContent);
      originalMergedContent = mergedContent;
    }
  }

  // Build merged content lines with conflict info
  interface MergedLine {
    lineNum: number;
    content: string;
    type: 'normal' | 'conflict-start' | 'conflict-local' | 'conflict-sep' | 'conflict-remote' | 'conflict-end';
    conflictIndex?: number;
  }

  const mergedLines = $derived.by(() => {
    const lines = mergedContent.split('\n');
    const result: MergedLine[] = [];
    let conflictIdx = -1;
    let inConflict: 'local' | 'remote' | null = null;

    for (let i = 0; i < lines.length; i++) {
      const line = lines[i];

      if (line === '<<<<<<< LOCAL') {
        conflictIdx++;
        inConflict = 'local';
        result.push({ lineNum: i + 1, content: line, type: 'conflict-start', conflictIndex: conflictIdx });
      } else if (line === '=======' && inConflict === 'local') {
        inConflict = 'remote';
        result.push({ lineNum: i + 1, content: line, type: 'conflict-sep', conflictIndex: conflictIdx });
      } else if (line === '>>>>>>> REMOTE' && inConflict === 'remote') {
        inConflict = null;
        result.push({ lineNum: i + 1, content: line, type: 'conflict-end', conflictIndex: conflictIdx });
      } else if (inConflict === 'local') {
        result.push({ lineNum: i + 1, content: line, type: 'conflict-local', conflictIndex: conflictIdx });
      } else if (inConflict === 'remote') {
        result.push({ lineNum: i + 1, content: line, type: 'conflict-remote', conflictIndex: conflictIdx });
      } else {
        result.push({ lineNum: i + 1, content: line, type: 'normal' });
      }
    }

    return result;
  });
</script>

<div class="three-way-view">
  {#if baseToLocalDiff && baseToRemoteDiff}
  <!-- Top: Three-way comparison -->
  <div class="diff-container">
    <DiffPane
      file={local}
      lines={localPaneLines.right}
      side="left"
      bind:scrollRef={localScrollRef}
      onscroll={() => handleScroll('local')}
    />
    <div class="pane-divider"></div>
    <DiffPane
      file={base}
      lines={localPaneLines.left}
      side="left"
      bind:scrollRef={baseScrollRef}
      onscroll={() => handleScroll('base')}
    />
    <div class="pane-divider"></div>
    <DiffPane
      file={remote}
      lines={remotePaneLines.right}
      side="right"
      bind:scrollRef={remoteScrollRef}
      onscroll={() => handleScroll('remote')}
    />
  </div>

  <!-- Bottom: Merged result with conflict resolution -->
  <div class="merged-container">
    <div class="merged-header">
      <span class="merged-title">Merged Result</span>
      {#if conflictPositions.length > 0}
        <span class="conflict-badge">{conflictPositions.length} conflict{conflictPositions.length > 1 ? 's' : ''}</span>
      {:else if conflictCount > 0}
        <span class="resolved-badge">All conflicts resolved</span>
      {:else}
        <span class="resolved-badge">No conflicts</span>
      {/if}
      {#if onSaveMerged}
        <button
          class="save-btn"
          onclick={saveMerged}
          disabled={conflictPositions.length > 0 || !isDirty}
        >
          Save Merged
        </button>
      {/if}
    </div>
    <div class="merged-content">
      {#each mergedLines as line, i (i)}
        <div
          class="merged-line"
          class:conflict-start={line.type === 'conflict-start'}
          class:conflict-local={line.type === 'conflict-local'}
          class:conflict-sep={line.type === 'conflict-sep'}
          class:conflict-remote={line.type === 'conflict-remote'}
          class:conflict-end={line.type === 'conflict-end'}
        >
          <span class="line-num">{line.lineNum}</span>
          <span class="line-content">{line.content}</span>
          {#if line.type === 'conflict-start' && line.conflictIndex !== undefined}
            <div class="conflict-actions">
              <button
                class="resolve-btn use-local"
                onclick={() => resolveConflict(line.conflictIndex!, 'local')}
                title="Use Local"
              >
                ← Local
              </button>
              <button
                class="resolve-btn use-remote"
                onclick={() => resolveConflict(line.conflictIndex!, 'remote')}
                title="Use Remote"
              >
                Remote →
              </button>
            </div>
          {/if}
        </div>
      {/each}
    </div>
  </div>
  {:else}
  <div class="loading">Loading...</div>
  {/if}

  <!-- Status bar -->
  <div class="status-bar">
    <span class="stat">Local: <span class="additions">+{baseToLocalDiff?.stats.additions ?? 0}</span> <span class="deletions">-{baseToLocalDiff?.stats.deletions ?? 0}</span></span>
    <span class="stat">Remote: <span class="additions">+{baseToRemoteDiff?.stats.additions ?? 0}</span> <span class="deletions">-{baseToRemoteDiff?.stats.deletions ?? 0}</span></span>

    {#if conflictPositions.length > 0}
      <div class="nav-controls">
        <button class="nav-button" onclick={prevConflict} title="Previous conflict (P)">
          <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <polyline points="15 18 9 12 15 6"></polyline>
          </svg>
        </button>
        <span class="nav-indicator">
          {currentConflictIndex >= 0 ? currentConflictIndex + 1 : '-'} / {conflictPositions.length}
        </span>
        <button class="nav-button" onclick={nextConflict} title="Next conflict (N)">
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
    gap: var(--spacing-sm);
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

  .merged-container {
    flex: 0 0 200px;
    display: flex;
    flex-direction: column;
    border: 1px solid var(--color-border);
    border-radius: var(--radius-md);
    overflow: hidden;
  }

  .merged-header {
    display: flex;
    align-items: center;
    gap: var(--spacing-md);
    padding: var(--spacing-sm) var(--spacing-md);
    background: var(--color-bg-secondary);
    border-bottom: 1px solid var(--color-border);
  }

  .merged-title {
    font-weight: 600;
    font-size: var(--font-size-sm);
  }

  .conflict-badge {
    background: rgba(255, 180, 0, 0.2);
    color: #b58900;
    padding: 2px 8px;
    border-radius: var(--radius-sm);
    font-size: var(--font-size-xs);
    font-weight: 500;
  }

  .resolved-badge {
    background: var(--color-diff-insert-bg);
    color: var(--color-diff-insert-text);
    padding: 2px 8px;
    border-radius: var(--radius-sm);
    font-size: var(--font-size-xs);
    font-weight: 500;
  }

  .save-btn {
    margin-left: auto;
    padding: var(--spacing-xs) var(--spacing-md);
    border-radius: var(--radius-sm);
    background: var(--color-accent-primary);
    color: white;
    border: none;
    font-size: var(--font-size-sm);
    cursor: pointer;
  }

  .save-btn:disabled {
    background: var(--color-bg-tertiary);
    color: var(--color-text-muted);
    cursor: not-allowed;
  }

  .merged-content {
    flex: 1;
    overflow: auto;
    font-family: var(--font-mono);
    font-size: var(--font-size-sm);
  }

  .merged-line {
    display: flex;
    align-items: center;
    min-height: 21px;
    line-height: 21px;
  }

  .merged-line .line-num {
    width: 50px;
    padding: 0 var(--spacing-xs);
    text-align: right;
    color: var(--color-text-muted);
    background: var(--color-bg-secondary);
    border-right: 1px solid var(--color-border);
    user-select: none;
    flex-shrink: 0;
  }

  .merged-line .line-content {
    flex: 1;
    padding: 0 var(--spacing-sm);
    white-space: pre;
  }

  .merged-line.conflict-start,
  .merged-line.conflict-sep,
  .merged-line.conflict-end {
    background: rgba(255, 180, 0, 0.3);
    color: #b58900;
    font-weight: 600;
  }

  .merged-line.conflict-local {
    background: var(--color-diff-insert-bg);
  }

  .merged-line.conflict-remote {
    background: var(--color-diff-delete-bg);
  }

  .conflict-actions {
    display: flex;
    gap: var(--spacing-xs);
    padding-right: var(--spacing-sm);
  }

  .resolve-btn {
    padding: 2px 8px;
    border-radius: var(--radius-sm);
    font-size: var(--font-size-xs);
    font-weight: 500;
    cursor: pointer;
    border: 1px solid var(--color-border);
    background: var(--color-bg-primary);
    transition: all 0.15s ease;
  }

  .resolve-btn.use-local {
    color: var(--color-diff-insert-text);
  }

  .resolve-btn.use-local:hover {
    background: var(--color-diff-insert-bg);
    border-color: var(--color-diff-insert-text);
  }

  .resolve-btn.use-remote {
    color: var(--color-diff-delete-text);
  }

  .resolve-btn.use-remote:hover {
    background: var(--color-diff-delete-bg);
    border-color: var(--color-diff-delete-text);
  }

  .loading {
    flex: 1;
    display: flex;
    align-items: center;
    justify-content: center;
    color: var(--color-text-muted);
  }

  .status-bar {
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
  }

  .nav-button:hover {
    background: var(--color-bg-hover);
    color: var(--color-text-primary);
  }

  .nav-indicator {
    font-family: var(--font-mono);
    font-size: var(--font-size-xs);
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
  }

  .sync-toggle:hover {
    background: var(--color-bg-hover);
  }

  .sync-toggle.active {
    background: var(--color-bg-tertiary);
    color: var(--color-accent-primary);
    border-color: var(--color-accent-primary);
  }
</style>
