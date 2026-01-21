<script lang="ts">
  import { invoke } from '@tauri-apps/api/core';
  import DiffView from './DiffView.svelte';
  import type { Tab } from '$lib/stores/tabs.svelte';
  import { tabStore } from '$lib/stores/tabs.svelte';
  import type { FileDiffResult } from '$lib/types';

  interface Props {
    tab: Tab;
  }

  let { tab }: Props = $props();

  // Get parent tab for back navigation
  let parentTab = $derived(tab.parentTabId ? tabStore.getTab(tab.parentTabId) : null);

  function goBack() {
    if (tab.parentTabId) {
      tabStore.setActive(tab.parentTabId);
    }
  }

  let diffResult: FileDiffResult | null = $state(null);
  let error: string | null = $state(null);
  let loading = $state(true);

  // Dirty state
  let leftDirty = $state(false);
  let rightDirty = $state(false);

  function handleDirtyChange(left: boolean, right: boolean) {
    leftDirty = left;
    rightDirty = right;

    // Update tab dirty state
    const isDirty = left || right;
    tabStore.setDirty(tab.id, isDirty);
  }

  async function handleSaveLeft(content: string) {
    if (!tab.leftPath || !diffResult) return;
    try {
      await invoke('write_file', {
        path: tab.leftPath,
        content,
        encoding: diffResult.left.encoding
      });
    } catch (e) {
      console.error('Failed to save left file:', e);
      alert(`Failed to save: ${e}`);
    }
  }

  async function handleSaveRight(content: string) {
    if (!tab.rightPath || !diffResult) return;
    try {
      await invoke('write_file', {
        path: tab.rightPath,
        content,
        encoding: diffResult.right.encoding
      });
    } catch (e) {
      console.error('Failed to save right file:', e);
      alert(`Failed to save: ${e}`);
    }
  }

  // Track which paths we've loaded to prevent re-fetching
  let loadedPaths = $state('');

  // Load diff when tab paths change
  $effect(() => {
    const left = tab.leftPath;
    const right = tab.rightPath;
    const pathKey = `${left}:${right}`;

    if (left && right && pathKey !== loadedPaths) {
      loadedPaths = pathKey;
      loading = true;
      error = null;

      invoke<FileDiffResult>('compute_diff_files', {
        leftPath: left,
        rightPath: right,
      }).then(result => {
        diffResult = result;
        loading = false;
      }).catch(e => {
        error = e instanceof Error ? e.message : String(e);
        loading = false;
      });
    }
  });
</script>

<div class="compare-page">
  {#if parentTab}
    <div class="breadcrumb">
      <button class="back-btn" onclick={goBack} title="Back to directory comparison">
        <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <polyline points="15 18 9 12 15 6"></polyline>
        </svg>
        <span>{parentTab.title}</span>
      </button>
    </div>
  {/if}

  {#if loading}
    <div class="loading">Loading diff...</div>
  {:else if error}
    <div class="error">
      <p>Error loading diff:</p>
      <code>{error}</code>
    </div>
  {:else if diffResult}
    {#if diffResult.left.is_binary || diffResult.right.is_binary}
      <div class="binary-warning">
        <p>Binary files cannot be compared</p>
        <p class="paths">
          <code>{tab.leftPath}</code>
          <span>vs</span>
          <code>{tab.rightPath}</code>
        </p>
      </div>
    {:else}
      <DiffView
        result={diffResult}
        onDirtyChange={handleDirtyChange}
        onSaveLeft={handleSaveLeft}
        onSaveRight={handleSaveRight}
      />
    {/if}
  {/if}
</div>

<style>
  .compare-page {
    flex: 1;
    display: flex;
    flex-direction: column;
    padding: var(--spacing-lg);
    min-height: 0;
  }

  .breadcrumb {
    margin-bottom: var(--spacing-sm);
    flex-shrink: 0;
  }

  .back-btn {
    display: inline-flex;
    align-items: center;
    gap: var(--spacing-xs);
    padding: var(--spacing-xs) var(--spacing-sm);
    color: var(--color-text-muted);
    font-size: var(--font-size-sm);
    border-radius: var(--radius-sm);
    transition: all var(--transition-fast);
  }

  .back-btn:hover {
    color: var(--color-text-primary);
    background: var(--color-bg-hover);
  }

  .back-btn svg {
    flex-shrink: 0;
  }

  .loading {
    flex: 1;
    display: flex;
    align-items: center;
    justify-content: center;
    color: var(--color-text-muted);
  }

  .error {
    flex: 1;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: var(--spacing-md);
    color: var(--color-diff-delete-text);
  }

  .error code {
    font-family: var(--font-mono);
    padding: var(--spacing-md);
    background: var(--color-bg-secondary);
    border-radius: var(--radius-md);
    max-width: 100%;
    overflow-x: auto;
  }

  .binary-warning {
    flex: 1;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: var(--spacing-md);
    color: var(--color-text-muted);
  }

  .binary-warning .paths {
    display: flex;
    align-items: center;
    gap: var(--spacing-md);
    font-size: var(--font-size-sm);
  }

  .binary-warning code {
    font-family: var(--font-mono);
    color: var(--color-text-secondary);
  }
</style>
