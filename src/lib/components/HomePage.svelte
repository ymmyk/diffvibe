<script lang="ts">
  import { open } from '@tauri-apps/plugin-dialog';
  import { onMount } from 'svelte';
  import { tabStore } from '$lib/stores/tabs.svelte';

  // Use persisted state from tab store
  let mode = $derived(tabStore.homeState.mode);
  let leftPath = $derived(tabStore.homeState.leftPath);
  let rightPath = $derived(tabStore.homeState.rightPath);
  let basePath = $derived(tabStore.homeState.basePath);

  function setMode(newMode: 'file' | 'directory' | 'merge') {
    tabStore.setHomeState({ mode: newMode });
  }

  onMount(() => {
    function handleKeydown(e: KeyboardEvent) {
      if ((e.ctrlKey || e.metaKey) && e.key === 'o') {
        e.preventDefault();
        if (!leftPath) selectPath('left');
        else if (!rightPath) selectPath('right');
        else selectPath('left');
      }
    }
    window.addEventListener('keydown', handleKeydown);
    return () => window.removeEventListener('keydown', handleKeydown);
  });

  async function selectPath(target: 'left' | 'right' | 'base') {
    const selected = await open({
      multiple: false,
      directory: mode === 'directory',
    });
    if (selected) {
      if (target === 'left') tabStore.setHomeState({ leftPath: selected as string });
      else if (target === 'right') tabStore.setHomeState({ rightPath: selected as string });
      else tabStore.setHomeState({ basePath: selected as string });
    }
  }

  function canCompare(): boolean {
    if (mode === 'merge') {
      return !!leftPath && !!rightPath && !!basePath;
    }
    return !!leftPath && !!rightPath;
  }

  function startComparison() {
    if (!canCompare()) return;
    tabStore.openCompare(leftPath, rightPath, mode, basePath || undefined);
  }

  function swapPaths() {
    tabStore.setHomeState({ leftPath: rightPath, rightPath: leftPath });
  }
</script>

<div class="home">
  <section class="hero">
    <h1 class="hero-title">Compare Files & Directories</h1>
    <p class="hero-subtitle">Visual diff and merge tool for developers</p>
  </section>

  <section class="mode-selector">
    <button
      class="mode-button"
      class:active={mode === 'file'}
      onclick={() => setMode('file')}
    >
      <svg class="mode-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
        <path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z"></path>
        <polyline points="14 2 14 8 20 8"></polyline>
      </svg>
      <span class="mode-label">File Comparison</span>
      <span class="mode-desc">Compare two files side by side</span>
    </button>

    <button
      class="mode-button"
      class:active={mode === 'directory'}
      onclick={() => setMode('directory')}
    >
      <svg class="mode-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
        <path d="M22 19a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h5l2 3h9a2 2 0 0 1 2 2z"></path>
      </svg>
      <span class="mode-label">Directory Comparison</span>
      <span class="mode-desc">Compare folder contents</span>
    </button>

    <button
      class="mode-button"
      class:active={mode === 'merge'}
      onclick={() => setMode('merge')}
    >
      <svg class="mode-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
        <circle cx="18" cy="18" r="3"></circle>
        <circle cx="6" cy="6" r="3"></circle>
        <path d="M6 21V9a9 9 0 0 0 9 9"></path>
      </svg>
      <span class="mode-label">Three-Way Merge</span>
      <span class="mode-desc">Merge with common ancestor</span>
    </button>
  </section>

  <section class="file-selector">
    <div class="file-input">
      <label class="input-label">
        {mode === 'directory' ? 'Left Directory' : 'Left File'}
      </label>
      <div class="input-row">
        <input
          type="text"
          class="path-input"
          placeholder={mode === 'directory' ? 'Select a directory...' : 'Select a file...'}
          value={leftPath}
          readonly
        />
        <button class="browse-button" onclick={() => selectPath('left')}>
          Browse
        </button>
      </div>
    </div>

    <button
      class="swap-button"
      onclick={swapPaths}
      disabled={!leftPath && !rightPath}
      title="Swap left and right"
    >
      <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
        <polyline points="17 1 21 5 17 9"></polyline>
        <path d="M3 11V9a4 4 0 0 1 4-4h14"></path>
        <polyline points="7 23 3 19 7 15"></polyline>
        <path d="M21 13v2a4 4 0 0 1-4 4H3"></path>
      </svg>
    </button>

    <div class="file-input">
      <label class="input-label">
        {mode === 'directory' ? 'Right Directory' : 'Right File'}
      </label>
      <div class="input-row">
        <input
          type="text"
          class="path-input"
          placeholder={mode === 'directory' ? 'Select a directory...' : 'Select a file...'}
          value={rightPath}
          readonly
        />
        <button class="browse-button" onclick={() => selectPath('right')}>
          Browse
        </button>
      </div>
    </div>

    {#if mode === 'merge'}
      <div class="file-input">
        <label class="input-label">Base File (Common Ancestor)</label>
        <div class="input-row">
          <input
            type="text"
            class="path-input"
            placeholder="Select base file..."
            value={basePath}
            readonly
          />
          <button class="browse-button" onclick={() => selectPath('base')}>
            Browse
          </button>
        </div>
      </div>
    {/if}
  </section>

  <section class="actions">
    <button
      class="compare-button"
      onclick={startComparison}
      disabled={!canCompare()}
    >
      <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
        <polyline points="16 3 21 3 21 8"></polyline>
        <line x1="4" y1="20" x2="21" y2="3"></line>
        <polyline points="21 16 21 21 16 21"></polyline>
        <line x1="15" y1="15" x2="21" y2="21"></line>
        <line x1="4" y1="4" x2="9" y2="9"></line>
      </svg>
      Compare
    </button>
  </section>

  <section class="recent">
    <h2 class="section-title">Recent Comparisons</h2>
    <div class="empty-state">
      <svg width="48" height="48" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
        <circle cx="12" cy="12" r="10"></circle>
        <polyline points="12 6 12 12 16 14"></polyline>
      </svg>
      <p>No recent comparisons</p>
    </div>
  </section>
</div>

<style>
  .home {
    flex: 1;
    display: flex;
    flex-direction: column;
    padding: var(--spacing-xl);
    max-width: 900px;
    margin: 0 auto;
    width: 100%;
  }

  .hero {
    text-align: center;
    padding: var(--spacing-xl) 0;
  }

  .hero-title {
    font-size: var(--font-size-3xl);
    font-weight: 700;
    background: var(--color-gradient-accent);
    -webkit-background-clip: text;
    -webkit-text-fill-color: transparent;
    background-clip: text;
    margin-bottom: var(--spacing-sm);
  }

  .hero-subtitle {
    color: var(--color-text-muted);
    font-size: var(--font-size-lg);
  }

  .mode-selector {
    display: grid;
    grid-template-columns: repeat(3, 1fr);
    gap: var(--spacing-md);
    margin-bottom: var(--spacing-xl);
  }

  .mode-button {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: var(--spacing-sm);
    padding: var(--spacing-lg);
    background: var(--color-bg-secondary);
    border: 2px solid var(--color-border);
    border-radius: var(--radius-lg);
    color: var(--color-text-muted);
    transition: all var(--transition-normal);
  }

  .mode-button:hover {
    border-color: var(--color-border-hover);
    color: var(--color-text-secondary);
  }

  .mode-button.active {
    border-color: var(--color-border-active);
    color: var(--color-text-primary);
    background: var(--color-bg-tertiary);
  }

  .mode-icon {
    width: 32px;
    height: 32px;
  }

  .mode-label {
    font-weight: 600;
    font-size: var(--font-size-md);
  }

  .mode-desc {
    font-size: var(--font-size-xs);
    text-align: center;
  }

  .file-selector {
    display: flex;
    flex-direction: column;
    gap: var(--spacing-lg);
    margin-bottom: var(--spacing-xl);
  }

  .file-input {
    display: flex;
    flex-direction: column;
    gap: var(--spacing-sm);
  }

  .input-label {
    font-weight: 500;
    color: var(--color-text-secondary);
    font-size: var(--font-size-sm);
  }

  .input-row {
    display: flex;
    gap: var(--spacing-sm);
  }

  .path-input {
    flex: 1;
    padding: var(--spacing-sm) var(--spacing-md);
    background: var(--color-bg-secondary);
    border: 1px solid var(--color-border);
    border-radius: var(--radius-md);
    font-size: var(--font-size-sm);
    font-family: var(--font-mono);
  }

  .path-input:focus {
    outline: none;
    border-color: var(--color-border-active);
  }

  .path-input::placeholder {
    color: var(--color-text-disabled);
    font-family: inherit;
  }

  .browse-button {
    padding: var(--spacing-sm) var(--spacing-lg);
    background: var(--color-bg-hover);
    border: 1px solid var(--color-border-hover);
    border-radius: var(--radius-md);
    font-size: var(--font-size-sm);
    font-weight: 500;
    transition: all var(--transition-fast);
  }

  .browse-button:hover {
    background: var(--color-border-hover);
  }

  .swap-button {
    align-self: center;
    padding: var(--spacing-sm);
    background: var(--color-bg-secondary);
    border: 1px solid var(--color-border);
    border-radius: var(--radius-md);
    color: var(--color-text-muted);
    transition: all var(--transition-fast);
  }

  .swap-button:hover:not(:disabled) {
    background: var(--color-bg-hover);
    color: var(--color-text-primary);
    border-color: var(--color-border-hover);
  }

  .swap-button:disabled {
    opacity: 0.3;
    cursor: not-allowed;
  }

  .actions {
    display: flex;
    justify-content: center;
    margin-bottom: var(--spacing-2xl);
  }

  .compare-button {
    display: flex;
    align-items: center;
    gap: var(--spacing-sm);
    padding: var(--spacing-md) var(--spacing-2xl);
    background: var(--color-gradient-accent);
    border-radius: var(--radius-md);
    color: var(--color-bg-primary);
    font-size: var(--font-size-lg);
    font-weight: 600;
    transition: all var(--transition-normal);
  }

  .compare-button:hover:not(:disabled) {
    transform: translateY(-2px);
    box-shadow: var(--shadow-glow);
  }

  .compare-button:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .recent {
    flex: 1;
  }

  .section-title {
    font-size: var(--font-size-lg);
    font-weight: 600;
    color: var(--color-text-secondary);
    margin-bottom: var(--spacing-md);
  }

  .empty-state {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: var(--spacing-md);
    padding: var(--spacing-2xl);
    background: var(--color-bg-secondary);
    border: 1px dashed var(--color-border);
    border-radius: var(--radius-lg);
    color: var(--color-text-disabled);
  }
</style>
