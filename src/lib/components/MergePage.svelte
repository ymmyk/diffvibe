<script lang="ts">
  import { invoke } from '@tauri-apps/api/core';
  import ThreeWayView from './ThreeWayView.svelte';
  import type { Tab } from '$lib/stores/tabs.svelte';
  import { tabStore } from '$lib/stores/tabs.svelte';
  import type { FileContent } from '$lib/types';

  interface Props {
    tab: Tab;
  }

  let { tab }: Props = $props();

  let baseFile: FileContent | null = $state(null);
  let localFile: FileContent | null = $state(null);
  let remoteFile: FileContent | null = $state(null);
  let error: string | null = $state(null);
  let loading = $state(true);

  // Track which paths we've loaded
  let loadedPaths = $state('');

  // Load files when tab paths change
  $effect(() => {
    const base = tab.basePath;
    const left = tab.leftPath;
    const right = tab.rightPath;
    const pathKey = `${base}:${left}:${right}`;

    if (base && left && right && pathKey !== loadedPaths) {
      loadedPaths = pathKey;
      loading = true;
      error = null;

      Promise.all([
        invoke<FileContent>('read_file', { path: base }),
        invoke<FileContent>('read_file', { path: left }),
        invoke<FileContent>('read_file', { path: right }),
      ]).then(([b, l, r]) => {
        baseFile = b;
        localFile = l;
        remoteFile = r;
        loading = false;
      }).catch(e => {
        error = e instanceof Error ? e.message : String(e);
        loading = false;
      });
    }
  });

  function handleDirtyChange(dirty: boolean) {
    tabStore.setDirty(tab.id, dirty);
  }
</script>

<div class="merge-page">
  {#if loading}
    <div class="loading">Loading files...</div>
  {:else if error}
    <div class="error">
      <p>Error loading files:</p>
      <code>{error}</code>
    </div>
  {:else if baseFile && localFile && remoteFile}
    {#if baseFile.is_binary || localFile.is_binary || remoteFile.is_binary}
      <div class="binary-warning">
        <p>Binary files cannot be merged</p>
      </div>
    {:else}
      <ThreeWayView
        base={baseFile}
        local={localFile}
        remote={remoteFile}
        onDirtyChange={handleDirtyChange}
      />
    {/if}
  {/if}
</div>

<style>
  .merge-page {
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
</style>
