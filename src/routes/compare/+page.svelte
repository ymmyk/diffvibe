<script lang="ts">
  import { page } from '$app/stores';

  const leftPath = $derived($page.url.searchParams.get('left') || '');
  const rightPath = $derived($page.url.searchParams.get('right') || '');
  const mode = $derived($page.url.searchParams.get('mode') || 'file');
</script>

<div class="compare-page">
  <header class="compare-header">
    <a href="/" class="back-link">&larr; Back</a>
    <h1>Compare</h1>
  </header>

  <div class="file-info">
    <div class="file-path">
      <span class="label">Left:</span>
      <code>{leftPath}</code>
    </div>
    <div class="file-path">
      <span class="label">Right:</span>
      <code>{rightPath}</code>
    </div>
  </div>

  <div class="placeholder">
    <p>Diff view coming in milestone 1b</p>
    <p class="mode">Mode: {mode}</p>
  </div>
</div>

<style>
  .compare-page {
    flex: 1;
    display: flex;
    flex-direction: column;
    padding: var(--spacing-lg);
  }

  .compare-header {
    display: flex;
    align-items: center;
    gap: var(--spacing-lg);
    margin-bottom: var(--spacing-lg);
  }

  .back-link {
    color: var(--color-text-muted);
    text-decoration: none;
    font-size: var(--font-size-sm);
  }

  .back-link:hover {
    color: var(--color-text-primary);
  }

  h1 {
    font-size: var(--font-size-xl);
    font-weight: 600;
  }

  .file-info {
    display: flex;
    flex-direction: column;
    gap: var(--spacing-sm);
    padding: var(--spacing-md);
    background: var(--color-bg-secondary);
    border-radius: var(--radius-md);
    margin-bottom: var(--spacing-lg);
  }

  .file-path {
    display: flex;
    gap: var(--spacing-sm);
    font-size: var(--font-size-sm);
  }

  .label {
    color: var(--color-text-muted);
    min-width: 50px;
  }

  code {
    font-family: var(--font-mono);
    color: var(--color-text-secondary);
    word-break: break-all;
  }

  .placeholder {
    flex: 1;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    color: var(--color-text-muted);
    gap: var(--spacing-sm);
  }

  .mode {
    font-size: var(--font-size-sm);
    color: var(--color-text-disabled);
  }
</style>
