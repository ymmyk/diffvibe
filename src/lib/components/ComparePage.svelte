<script lang="ts">
  import { invoke } from '@tauri-apps/api/core';
  import { onMount } from 'svelte';
  import DiffView from './DiffView.svelte';
  import type { Tab } from '$lib/stores/tabs.svelte';
  import type { FileDiffResult } from '$lib/types';

  interface Props {
    tab: Tab;
  }

  let { tab }: Props = $props();

  let diffResult: FileDiffResult | null = $state(null);
  let error: string | null = $state(null);
  let loading = $state(true);

  async function loadDiff() {
    if (!tab.leftPath || !tab.rightPath) {
      error = 'Missing file paths';
      loading = false;
      return;
    }

    try {
      loading = true;
      error = null;
      diffResult = await invoke<FileDiffResult>('compute_diff_files', {
        leftPath: tab.leftPath,
        rightPath: tab.rightPath,
      });
    } catch (e) {
      error = e instanceof Error ? e.message : String(e);
      diffResult = null;
    } finally {
      loading = false;
    }
  }

  onMount(() => {
    loadDiff();
  });

  // Reload when tab changes
  $effect(() => {
    if (tab.leftPath && tab.rightPath) {
      loadDiff();
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
      <DiffView result={diffResult} />
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
