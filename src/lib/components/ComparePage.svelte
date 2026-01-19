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
