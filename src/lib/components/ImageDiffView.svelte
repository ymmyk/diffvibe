<script lang="ts">
  import { invoke } from '@tauri-apps/api/core';

  interface Props {
    leftPath: string;
    rightPath: string;
  }

  interface ImageDiffMaskResult {
    width: number;
    height: number;
    left_data_url: string;
    right_data_url: string;
    mask_data_url: string;
  }

  let { leftPath, rightPath }: Props = $props();

  let leftUrl = $state('');
  let rightUrl = $state('');
  let maskUrl = $state('');
  let loading = $state(true);
  let error: string | null = $state(null);
  let loadToken = 0;

  async function loadImages() {
    const token = ++loadToken;
    loading = true;
    error = null;

    try {
      const result = await invoke<ImageDiffMaskResult>('compute_image_diff_mask', {
        leftPath,
        rightPath
      });
      if (token !== loadToken) return;
      leftUrl = result.left_data_url;
      rightUrl = result.right_data_url;
      maskUrl = result.mask_data_url;
    } catch (e) {
      if (token !== loadToken) return;
      error = e instanceof Error ? e.message : String(e);
    } finally {
      if (token === loadToken) {
        loading = false;
      }
    }
  }

  $effect(() => {
    if (leftPath && rightPath) {
      loadImages();
    }
  });
</script>

<div class="image-diff">
  {#if loading}
    <div class="state">Loading images...</div>
  {:else if error}
    <div class="state error">
      <p>Failed to compare images</p>
      <code>{error}</code>
    </div>
  {:else}
    <div class="pane-grid">
      <div class="pane">
        <div class="pane-label">Left</div>
        <div class="viewport">
          <img src={leftUrl} alt="Left" />
        </div>
      </div>

      <div class="pane">
        <div class="pane-label">Mask</div>
        <div class="viewport">
          <img src={maskUrl} alt="Difference mask" />
        </div>
      </div>

      <div class="pane">
        <div class="pane-label">Right</div>
        <div class="viewport">
          <img src={rightUrl} alt="Right" />
        </div>
      </div>
    </div>
  {/if}
</div>

<style>
  .image-diff {
    flex: 1;
    min-height: 0;
    display: flex;
  }

  .state {
    flex: 1;
    display: flex;
    align-items: center;
    justify-content: center;
    color: var(--color-text-muted);
  }

  .state.error {
    flex-direction: column;
    gap: var(--spacing-md);
    color: var(--color-diff-delete-text);
  }

  .state.error code {
    max-width: 100%;
    overflow: auto;
    padding: var(--spacing-sm) var(--spacing-md);
    border-radius: var(--radius-md);
    background: var(--color-bg-secondary);
    color: var(--color-text-secondary);
  }

  .pane-grid {
    flex: 1;
    min-height: 0;
    display: grid;
    grid-template-columns: 1fr 1fr 1fr;
    gap: var(--spacing-md);
  }

  .pane {
    min-width: 0;
    min-height: 0;
    display: flex;
    flex-direction: column;
    gap: var(--spacing-xs);
  }

  .pane-label {
    font-size: var(--font-size-xs);
    text-transform: uppercase;
    letter-spacing: 0.04em;
    color: var(--color-text-muted);
  }

  .viewport {
    flex: 1;
    min-height: 0;
    background: var(--color-bg-secondary);
    border: 1px solid var(--color-border);
    border-radius: var(--radius-md);
    overflow: auto;
    display: flex;
    align-items: flex-start;
    justify-content: flex-start;
  }

  .viewport img {
    display: block;
    max-width: none;
    max-height: none;
  }
</style>
