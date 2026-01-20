<script lang="ts">
  import { untrack } from 'svelte';
  import { invoke } from '@tauri-apps/api/core';
  import type { FileContent, MergeResult, DiffResult } from '$lib/types';
  import DiffPane from './DiffPane.svelte';

  interface Props {
    base: FileContent;
    local: FileContent;
    remote: FileContent;
    outputPath?: string;
  }

  let { base, local, remote, outputPath }: Props = $props();

  // Save state
  let saving = $state(false);
  let saveError = $state<string | null>(null);

  // Diff results - base→local and base→remote
  let baseToLocalDiff = $state<DiffResult | null>(null);
  let baseToRemoteDiff = $state<DiffResult | null>(null);
  let mergeResult = $state<MergeResult | null>(null);
  let loadedFileKey = $state('');
  let isLoading = $state(false);


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
      loadedFileKey = key;
    } catch (e) {
      console.error('Failed to compute diffs:', e);
    } finally {
      isLoading = false;
    }
  }

  // Scroll sync
  let localScrollRef: HTMLDivElement | null = $state(null);
  let baseScrollRef: HTMLDivElement | null = $state(null);
  let remoteScrollRef: HTMLDivElement | null = $state(null);
  let syncScroll = $state(true);
  let isScrolling = false;

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

  // Count conflicts
  const conflictCount = $derived(mergeResult?.conflict_count ?? 0);

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

  // Save merged content to output file
  async function saveMerged() {
    if (!outputPath || !mergeResult) return;

    saving = true;
    saveError = null;
    try {
      await invoke('write_file', {
        path: outputPath,
        content: mergeResult.merged_content,
        encoding: base.encoding,
      });
      // Exit with code 0 (success) after saving
      await invoke('exit_app', { code: 0 });
    } catch (e) {
      saveError = e instanceof Error ? e.message : String(e);
    } finally {
      saving = false;
    }
  }

  // Check if save is possible (no conflicts for clean save)
  const canSave = $derived(outputPath && mergeResult && conflictCount === 0);
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

  {:else}
  <div class="loading">Loading...</div>
  {/if}

  <!-- Status bar -->
  <div class="status-bar">
    <span class="stat">Local: <span class="additions">+{baseToLocalDiff?.stats.additions ?? 0}</span> <span class="deletions">-{baseToLocalDiff?.stats.deletions ?? 0}</span></span>
    <span class="stat">Remote: <span class="additions">+{baseToRemoteDiff?.stats.additions ?? 0}</span> <span class="deletions">-{baseToRemoteDiff?.stats.deletions ?? 0}</span></span>
    {#if conflictCount > 0}
      <span class="conflict-badge">{conflictCount} conflict{conflictCount > 1 ? 's' : ''}</span>
    {:else if mergeResult}
      <span class="no-conflicts">No conflicts</span>
    {/if}

    {#if saveError}
      <span class="save-error">{saveError}</span>
    {/if}

    {#if outputPath}
      <button
        class="save-btn"
        onclick={saveMerged}
        disabled={!canSave || saving}
        title={conflictCount > 0 ? 'Resolve all conflicts before saving' : `Save to ${outputPath}`}
      >
        {#if saving}
          Saving...
        {:else}
          Save Merged
        {/if}
      </button>
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

  .conflict-badge {
    background: rgba(255, 180, 0, 0.2);
    color: #b58900;
    padding: 2px 8px;
    border-radius: var(--radius-sm);
    font-size: var(--font-size-xs);
    font-weight: 500;
  }

  .no-conflicts {
    background: var(--color-diff-insert-bg);
    color: var(--color-diff-insert-text);
    padding: 2px 8px;
    border-radius: var(--radius-sm);
    font-size: var(--font-size-xs);
    font-weight: 500;
  }

  .save-error {
    color: var(--color-diff-delete-text);
    font-size: var(--font-size-xs);
  }

  .save-btn {
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

  .save-btn:not(:disabled):hover {
    filter: brightness(1.1);
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
