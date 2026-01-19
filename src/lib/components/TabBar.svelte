<script lang="ts">
  import { tabStore } from '$lib/stores/tabs.svelte';

  function handleClose(e: MouseEvent, id: string) {
    e.stopPropagation();
    tabStore.close(id);
  }

  function handleMiddleClick(e: MouseEvent, id: string) {
    if (e.button === 1) {
      e.preventDefault();
      tabStore.close(id);
    }
  }
</script>

<div class="tab-bar">
  <div class="tabs">
    {#each tabStore.tabs as tab (tab.id)}
      <div
        class="tab"
        class:active={tab.id === tabStore.activeTabId}
        onclick={() => tabStore.setActive(tab.id)}
        onmousedown={(e) => handleMiddleClick(e, tab.id)}
        title={tab.type === 'compare' ? `${tab.leftPath} ↔ ${tab.rightPath}` : tab.title}
        role="tab"
        tabindex="0"
        onkeydown={(e) => e.key === 'Enter' && tabStore.setActive(tab.id)}
      >
        <span class="tab-icon">
          {#if tab.type === 'home'}
            <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <path d="M3 9l9-7 9 7v11a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2z"></path>
              <polyline points="9 22 9 12 15 12 15 22"></polyline>
            </svg>
          {:else}
            <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z"></path>
              <polyline points="14 2 14 8 20 8"></polyline>
            </svg>
          {/if}
        </span>
        <span class="tab-title">{tab.title}</span>
        {#if tabStore.tabs.length > 1}
          <button
            class="tab-close"
            onclick={(e) => handleClose(e, tab.id)}
            title="Close tab"
          >
            <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <line x1="18" y1="6" x2="6" y2="18"></line>
              <line x1="6" y1="6" x2="18" y2="18"></line>
            </svg>
          </button>
        {/if}
      </div>
    {/each}
  </div>
</div>

<style>
  .tab-bar {
    background: var(--color-bg-secondary);
    border-bottom: 1px solid var(--color-border);
    flex-shrink: 0;
  }

  .tabs {
    display: flex;
    overflow-x: auto;
    scrollbar-width: none;
  }

  .tabs::-webkit-scrollbar {
    display: none;
  }

  .tab {
    display: flex;
    align-items: center;
    gap: var(--spacing-xs);
    padding: var(--spacing-sm) var(--spacing-md);
    border-right: 1px solid var(--color-border);
    background: transparent;
    color: var(--color-text-muted);
    font-size: var(--font-size-sm);
    white-space: nowrap;
    max-width: 200px;
    transition: all var(--transition-fast);
    cursor: pointer;
  }

  .tab:hover {
    background: var(--color-bg-hover);
    color: var(--color-text-secondary);
  }

  .tab.active {
    background: var(--color-bg-primary);
    color: var(--color-text-primary);
    border-bottom: 2px solid var(--color-accent-primary);
    margin-bottom: -1px;
  }

  .tab-icon {
    display: flex;
    align-items: center;
    flex-shrink: 0;
  }

  .tab-title {
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .tab-close {
    display: flex;
    align-items: center;
    justify-content: center;
    padding: 2px;
    border-radius: var(--radius-sm);
    opacity: 0.5;
    flex-shrink: 0;
    margin-left: var(--spacing-xs);
  }

  .tab-close:hover {
    opacity: 1;
    background: var(--color-bg-tertiary);
  }

  .tab.active .tab-close {
    opacity: 0.7;
  }
</style>
