<script lang="ts">
  import '../app.css';
  import { onMount } from 'svelte';
  import { themeStore } from '$lib/stores';

  interface Props {
    children: import('svelte').Snippet;
  }

  let { children }: Props = $props();

  onMount(() => {
    themeStore.init();
  });
</script>

<div class="app-container">
  <header class="app-header">
    <div class="header-left">
      <a href="/" class="logo">
        <span class="logo-text">DiffVibe</span>
      </a>
    </div>

    <nav class="header-nav">
      <a href="/" class="nav-link">Home</a>
    </nav>

    <div class="header-right">
      <button
        class="icon-button theme-toggle"
        title={themeStore.value === 'dark' ? 'Switch to light mode' : 'Switch to dark mode'}
        onclick={() => themeStore.toggle()}
      >
        {#if themeStore.value === 'dark'}
          <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <circle cx="12" cy="12" r="5"></circle>
            <line x1="12" y1="1" x2="12" y2="3"></line>
            <line x1="12" y1="21" x2="12" y2="23"></line>
            <line x1="4.22" y1="4.22" x2="5.64" y2="5.64"></line>
            <line x1="18.36" y1="18.36" x2="19.78" y2="19.78"></line>
            <line x1="1" y1="12" x2="3" y2="12"></line>
            <line x1="21" y1="12" x2="23" y2="12"></line>
            <line x1="4.22" y1="19.78" x2="5.64" y2="18.36"></line>
            <line x1="18.36" y1="5.64" x2="19.78" y2="4.22"></line>
          </svg>
        {:else}
          <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <path d="M21 12.79A9 9 0 1 1 11.21 3 7 7 0 0 0 21 12.79z"></path>
          </svg>
        {/if}
      </button>

      <button class="icon-button" title="Settings">
        <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <circle cx="12" cy="12" r="3"></circle>
          <path d="M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 0 1 0 2.83 2 2 0 0 1-2.83 0l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 0 1-2 2 2 2 0 0 1-2-2v-.09A1.65 1.65 0 0 0 9 19.4a1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 0 1-2.83 0 2 2 0 0 1 0-2.83l.06-.06a1.65 1.65 0 0 0 .33-1.82 1.65 1.65 0 0 0-1.51-1H3a2 2 0 0 1-2-2 2 2 0 0 1 2-2h.09A1.65 1.65 0 0 0 4.6 9a1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 0 1 0-2.83 2 2 0 0 1 2.83 0l.06.06a1.65 1.65 0 0 0 1.82.33H9a1.65 1.65 0 0 0 1-1.51V3a2 2 0 0 1 2-2 2 2 0 0 1 2 2v.09a1.65 1.65 0 0 0 1 1.51 1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 0 1 2.83 0 2 2 0 0 1 0 2.83l-.06.06a1.65 1.65 0 0 0-.33 1.82V9a1.65 1.65 0 0 0 1.51 1H21a2 2 0 0 1 2 2 2 2 0 0 1-2 2h-.09a1.65 1.65 0 0 0-1.51 1z"></path>
        </svg>
      </button>
    </div>
  </header>

  <main class="app-main">
    {@render children()}
  </main>
</div>

<style>
  .app-container {
    display: flex;
    flex-direction: column;
    min-height: 100vh;
  }

  .app-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    height: var(--header-height);
    padding: 0 var(--spacing-md);
    background: var(--color-bg-secondary);
    border-bottom: 1px solid var(--color-border);
    flex-shrink: 0;
  }

  .header-left {
    display: flex;
    align-items: center;
    gap: var(--spacing-md);
  }

  .logo {
    display: flex;
    align-items: center;
    gap: var(--spacing-sm);
  }

  .logo-text {
    font-size: var(--font-size-lg);
    font-weight: 700;
    background: var(--color-gradient-accent);
    -webkit-background-clip: text;
    -webkit-text-fill-color: transparent;
    background-clip: text;
  }

  .header-nav {
    display: flex;
    align-items: center;
    gap: var(--spacing-sm);
  }

  .nav-link {
    padding: var(--spacing-xs) var(--spacing-md);
    border-radius: var(--radius-md);
    color: var(--color-text-secondary);
    font-size: var(--font-size-sm);
    transition: all var(--transition-fast);
  }

  .nav-link:hover {
    color: var(--color-text-primary);
    background: var(--color-bg-hover);
  }

  .header-right {
    display: flex;
    align-items: center;
    gap: var(--spacing-sm);
  }

  .icon-button {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 32px;
    height: 32px;
    border-radius: var(--radius-md);
    color: var(--color-text-secondary);
    transition: all var(--transition-fast);
  }

  .icon-button:hover {
    color: var(--color-text-primary);
    background: var(--color-bg-hover);
  }

  .theme-toggle:hover {
    color: var(--color-accent-primary);
  }

  .app-main {
    flex: 1;
    display: flex;
    flex-direction: column;
  }
</style>
