<script lang="ts">
  import { syntaxThemeStore } from '$lib/stores';
  import { AVAILABLE_THEMES } from '$lib/utils/syntax';
  
  let showDropdown = $state(false);
  let dropdownRef: HTMLDivElement | null = $state(null);
  
  function toggleDropdown() {
    showDropdown = !showDropdown;
  }
  
  function selectTheme(themeId: string) {
    syntaxThemeStore.set(themeId as any);
    showDropdown = false;
  }
  
  function closeDropdown() {
    showDropdown = false;
  }
  
  // Close dropdown when clicking outside
  $effect(() => {
    if (!showDropdown) return;
    
    function handleClickOutside(e: MouseEvent) {
      if (dropdownRef && !dropdownRef.contains(e.target as Node)) {
        closeDropdown();
      }
    }
    
    document.addEventListener('click', handleClickOutside);
    return () => document.removeEventListener('click', handleClickOutside);
  });
  
  function getCurrentThemeName(): string {
    const currentId = syntaxThemeStore.value;
    const allThemes = [...AVAILABLE_THEMES.dark, ...AVAILABLE_THEMES.light];
    return allThemes.find(t => t.id === currentId)?.name || 'Theme';
  }
</script>

<div class="theme-selector" bind:this={dropdownRef}>
  <button class="theme-button" onclick={toggleDropdown} title="Select syntax theme">
    <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
      <path d="M12 2.69l5.66 5.66a8 8 0 1 1-11.31 0z"></path>
    </svg>
    <span class="theme-label">{getCurrentThemeName()}</span>
    <svg class="chevron" class:open={showDropdown} width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
      <polyline points="6 9 12 15 18 9"></polyline>
    </svg>
  </button>
  
  {#if showDropdown}
    <div class="theme-dropdown">
      <div class="theme-group">
        <div class="theme-group-header">Dark Themes</div>
        {#each AVAILABLE_THEMES.dark as theme}
          <button
            class="theme-option"
            class:active={syntaxThemeStore.value === theme.id}
            onclick={() => selectTheme(theme.id)}
          >
            {theme.name}
            {#if syntaxThemeStore.value === theme.id}
              <svg class="check-icon" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                <polyline points="20 6 9 17 4 12"></polyline>
              </svg>
            {/if}
          </button>
        {/each}
      </div>
      
      <div class="theme-group">
        <div class="theme-group-header">Light Themes</div>
        {#each AVAILABLE_THEMES.light as theme}
          <button
            class="theme-option"
            class:active={syntaxThemeStore.value === theme.id}
            onclick={() => selectTheme(theme.id)}
          >
            {theme.name}
            {#if syntaxThemeStore.value === theme.id}
              <svg class="check-icon" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                <polyline points="20 6 9 17 4 12"></polyline>
              </svg>
            {/if}
          </button>
        {/each}
      </div>
    </div>
  {/if}
</div>

<style>
  .theme-selector {
    position: relative;
  }
  
  .theme-button {
    display: flex;
    align-items: center;
    gap: var(--spacing-xs);
    padding: var(--spacing-xs) var(--spacing-sm);
    border-radius: var(--radius-md);
    font-size: var(--font-size-sm);
    color: var(--color-text-secondary);
    background: transparent;
    border: 1px solid var(--color-border);
    cursor: pointer;
    transition: all var(--transition-fast);
  }
  
  .theme-button:hover {
    background: var(--color-bg-hover);
    color: var(--color-text-primary);
    border-color: var(--color-border-hover);
  }
  
  .theme-label {
    font-size: var(--font-size-xs);
    white-space: nowrap;
  }
  
  .chevron {
    transition: transform var(--transition-fast);
    opacity: 0.6;
  }
  
  .chevron.open {
    transform: rotate(180deg);
  }
  
  .theme-dropdown {
    position: absolute;
    top: calc(100% + 4px);
    right: 0;
    min-width: 200px;
    background: var(--color-bg-secondary);
    border: 1px solid var(--color-border);
    border-radius: var(--radius-md);
    box-shadow: 0 4px 12px rgba(0, 0, 0, 0.3);
    z-index: 1000;
    overflow: hidden;
  }
  
  .theme-group {
    padding: var(--spacing-xs) 0;
  }
  
  .theme-group + .theme-group {
    border-top: 1px solid var(--color-border);
  }
  
  .theme-group-header {
    padding: var(--spacing-xs) var(--spacing-sm);
    font-size: var(--font-size-xs);
    font-weight: 600;
    color: var(--color-text-muted);
    text-transform: uppercase;
    letter-spacing: 0.05em;
  }
  
  .theme-option {
    display: flex;
    align-items: center;
    justify-content: space-between;
    width: 100%;
    padding: var(--spacing-xs) var(--spacing-sm);
    font-size: var(--font-size-sm);
    color: var(--color-text-primary);
    background: transparent;
    border: none;
    cursor: pointer;
    text-align: left;
    transition: background var(--transition-fast);
  }
  
  .theme-option:hover {
    background: var(--color-bg-hover);
  }
  
  .theme-option.active {
    color: var(--color-accent-primary);
    font-weight: 500;
  }
  
  .check-icon {
    color: var(--color-accent-primary);
    flex-shrink: 0;
  }
</style>
