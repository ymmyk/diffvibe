<script lang="ts">
  import { onMount, untrack } from 'svelte';
  import { invoke } from '@tauri-apps/api/core';
  import type { FileDiffResult, DiffLine, DiffResult } from '$lib/types';
  import DiffPane from './DiffPane.svelte';

  interface Props {
    result: FileDiffResult;
  }

  let { result }: Props = $props();

  // Editable content state - starts from file content, can diverge on edit
  let leftContent = $state(result.left.content);
  let rightContent = $state(result.right.content);

  // Local diff result that updates when content changes
  let localDiff = $state<DiffResult>(result.diff);
  let diffDebounceTimer: ReturnType<typeof setTimeout> | null = null;

  // Reset content when result changes (new file loaded)
  $effect(() => {
    leftContent = result.left.content;
    rightContent = result.right.content;
    localDiff = result.diff;
  });

  // Recompute diff when content changes (debounced)
  async function recomputeDiff() {
    try {
      const newDiff = await invoke<DiffResult>('compute_diff', {
        left: leftContent,
        right: rightContent
      });
      localDiff = newDiff;
    } catch (e) {
      console.error('Failed to recompute diff:', e);
    }
  }

  function handleLeftContentChange(newContent: string) {
    leftContent = newContent;
    if (diffDebounceTimer) clearTimeout(diffDebounceTimer);
    diffDebounceTimer = setTimeout(recomputeDiff, 300);
  }

  function handleRightContentChange(newContent: string) {
    rightContent = newContent;
    if (diffDebounceTimer) clearTimeout(diffDebounceTimer);
    diffDebounceTimer = setTimeout(recomputeDiff, 300);
  }

  // Scroll sync state
  let leftScrollRef: HTMLDivElement | null = $state(null);
  let rightScrollRef: HTMLDivElement | null = $state(null);
  let syncScroll = $state(true);
  let isScrolling = false;

  // Navigation state
  let currentHunkIndex = $state(-1);

  // Search state
  let showSearch = $state(false);
  let searchQuery = $state('');
  let currentMatchIndex = $state(-1);
  let searchInputRef: HTMLInputElement | null = $state(null);

  // Split diff lines into left (old) and right (new) for side-by-side display
  interface PaneLine {
    lineNum: number | null;
    content: string;
    tag: 'equal' | 'insert' | 'delete' | 'empty';
  }

  function buildPaneLines(lines: DiffLine[]): { left: PaneLine[]; right: PaneLine[] } {
    const left: PaneLine[] = [];
    const right: PaneLine[] = [];

    for (const line of lines) {
      if (line.tag === 'equal') {
        left.push({ lineNum: line.old_index !== null ? line.old_index + 1 : null, content: line.value, tag: 'equal' });
        right.push({ lineNum: line.new_index !== null ? line.new_index + 1 : null, content: line.value, tag: 'equal' });
      } else if (line.tag === 'delete') {
        left.push({ lineNum: line.old_index !== null ? line.old_index + 1 : null, content: line.value, tag: 'delete' });
        right.push({ lineNum: null, content: '', tag: 'empty' });
      } else if (line.tag === 'insert') {
        left.push({ lineNum: null, content: '', tag: 'empty' });
        right.push({ lineNum: line.new_index !== null ? line.new_index + 1 : null, content: line.value, tag: 'insert' });
      }
    }

    return { left, right };
  }

  const paneLines = $derived(buildPaneLines(localDiff.lines));

  // Compute hunk start positions (row index in paneLines where a change block begins)
  const hunkPositions = $derived.by(() => {
    const positions: number[] = [];
    let inChange = false;

    for (let i = 0; i < paneLines.left.length; i++) {
      const leftTag = paneLines.left[i].tag;
      const rightTag = paneLines.right[i].tag;
      const isChange = leftTag !== 'equal' || rightTag !== 'equal';

      if (isChange && !inChange) {
        positions.push(i);
        inChange = true;
      } else if (!isChange) {
        inChange = false;
      }
    }

    return positions;
  });

  function scrollToRow(rowIndex: number) {
    if (!leftScrollRef) return;
    // Get actual line height from first line element
    const firstLine = leftScrollRef.querySelector('.line') as HTMLElement | null;
    const lineHeight = firstLine?.offsetHeight || 21;
    const targetScroll = rowIndex * lineHeight;
    leftScrollRef.scrollTop = targetScroll;
    if (syncScroll && rightScrollRef) {
      rightScrollRef.scrollTop = targetScroll;
    }
  }

  function goToHunk(index: number) {
    if (index < 0 || index >= hunkPositions.length) return;
    currentHunkIndex = index;
    scrollToRow(hunkPositions[index]);
  }

  function nextHunk() {
    if (hunkPositions.length === 0) return;
    const next = currentHunkIndex < hunkPositions.length - 1 ? currentHunkIndex + 1 : 0;
    goToHunk(next);
  }

  function prevHunk() {
    if (hunkPositions.length === 0) return;
    const prev = currentHunkIndex > 0 ? currentHunkIndex - 1 : hunkPositions.length - 1;
    goToHunk(prev);
  }

  // Search: find row indices containing query (searches both panes)
  const searchMatches = $derived.by(() => {
    if (!searchQuery.trim()) return [];
    const query = searchQuery.toLowerCase();
    const matches: number[] = [];

    for (let i = 0; i < paneLines.left.length; i++) {
      const leftContent = paneLines.left[i].content.toLowerCase();
      const rightContent = paneLines.right[i].content.toLowerCase();
      if (leftContent.includes(query) || rightContent.includes(query)) {
        matches.push(i);
      }
    }

    return matches;
  });

  // Reset match index when query changes - use untrack to avoid re-running on matches access
  $effect(() => {
    searchQuery; // subscribe to query changes only
    untrack(() => {
      if (searchMatches.length > 0) {
        currentMatchIndex = 0;
        scrollToRow(searchMatches[0]);
      } else {
        currentMatchIndex = -1;
      }
    });
  });

  function goToMatch(index: number) {
    if (index < 0 || index >= searchMatches.length) return;
    currentMatchIndex = index;
    scrollToRow(searchMatches[index]);
  }

  function nextMatch() {
    if (searchMatches.length === 0) return;
    const next = currentMatchIndex < searchMatches.length - 1 ? currentMatchIndex + 1 : 0;
    goToMatch(next);
  }

  function prevMatch() {
    if (searchMatches.length === 0) return;
    const prev = currentMatchIndex > 0 ? currentMatchIndex - 1 : searchMatches.length - 1;
    goToMatch(prev);
  }

  function openSearch() {
    showSearch = true;
    // Focus input after it renders
    requestAnimationFrame(() => searchInputRef?.focus());
  }

  function closeSearch() {
    showSearch = false;
    searchQuery = '';
  }

  // Keyboard navigation
  onMount(() => {
    function handleKeydown(e: KeyboardEvent) {
      // Cmd/Ctrl+F to open search
      if ((e.metaKey || e.ctrlKey) && e.key === 'f') {
        e.preventDefault();
        openSearch();
        return;
      }

      // Escape to close search
      if (e.key === 'Escape' && showSearch) {
        e.preventDefault();
        closeSearch();
        return;
      }

      // Skip hunk navigation if in input
      if (e.target instanceof HTMLInputElement || e.target instanceof HTMLTextAreaElement) return;

      if (e.key === 'n' || e.key === 'N') {
        e.preventDefault();
        nextHunk();
      } else if (e.key === 'p' || e.key === 'P') {
        e.preventDefault();
        prevHunk();
      }
    }

    window.addEventListener('keydown', handleKeydown);
    return () => window.removeEventListener('keydown', handleKeydown);
  });

  function handleSearchKeydown(e: KeyboardEvent) {
    if (e.key === 'Enter') {
      e.preventDefault();
      if (e.shiftKey) {
        prevMatch();
      } else {
        nextMatch();
      }
    }
  }

  function handleScroll(source: 'left' | 'right') {
    if (!syncScroll || isScrolling) return;

    const sourceEl = source === 'left' ? leftScrollRef : rightScrollRef;
    const targetEl = source === 'left' ? rightScrollRef : leftScrollRef;

    if (!sourceEl || !targetEl) return;

    isScrolling = true;
    targetEl.scrollTop = sourceEl.scrollTop;
    targetEl.scrollLeft = sourceEl.scrollLeft;

    // Reset flag after scroll settles
    requestAnimationFrame(() => {
      isScrolling = false;
    });
  }
</script>

<div class="diff-view">
  {#if showSearch}
    <div class="search-bar">
      <div class="search-input-wrapper">
        <svg class="search-icon" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <circle cx="11" cy="11" r="8"></circle>
          <line x1="21" y1="21" x2="16.65" y2="16.65"></line>
        </svg>
        <input
          type="text"
          class="search-input"
          placeholder="Search..."
          bind:this={searchInputRef}
          bind:value={searchQuery}
          onkeydown={handleSearchKeydown}
        />
        {#if searchQuery}
          <span class="search-count">
            {searchMatches.length > 0 ? `${currentMatchIndex + 1}/${searchMatches.length}` : 'No results'}
          </span>
        {/if}
      </div>
      <button class="search-nav-btn" onclick={prevMatch} disabled={searchMatches.length === 0} title="Previous (Shift+Enter)">
        <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <polyline points="18 15 12 9 6 15"></polyline>
        </svg>
      </button>
      <button class="search-nav-btn" onclick={nextMatch} disabled={searchMatches.length === 0} title="Next (Enter)">
        <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <polyline points="6 9 12 15 18 9"></polyline>
        </svg>
      </button>
      <button class="search-close-btn" onclick={closeSearch} title="Close (Escape)">
        <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <line x1="18" y1="6" x2="6" y2="18"></line>
          <line x1="6" y1="6" x2="18" y2="18"></line>
        </svg>
      </button>
    </div>
  {/if}

  <div class="diff-container">
    <DiffPane
      file={result.left}
      lines={paneLines.left}
      side="left"
      bind:scrollRef={leftScrollRef}
      onscroll={() => handleScroll('left')}
      searchQuery={searchQuery}
      currentMatchRow={currentMatchIndex >= 0 ? searchMatches[currentMatchIndex] : -1}
      content={leftContent}
      onContentChange={handleLeftContentChange}
    />
    <div class="diff-gutter"></div>
    <DiffPane
      file={result.right}
      lines={paneLines.right}
      side="right"
      bind:scrollRef={rightScrollRef}
      onscroll={() => handleScroll('right')}
      searchQuery={searchQuery}
      currentMatchRow={currentMatchIndex >= 0 ? searchMatches[currentMatchIndex] : -1}
      content={rightContent}
      onContentChange={handleRightContentChange}
    />
  </div>

  <div class="diff-stats">
    <span class="stat additions">+{localDiff.stats.additions}</span>
    <span class="stat deletions">-{localDiff.stats.deletions}</span>
    <span class="stat unchanged">{localDiff.stats.unchanged} unchanged</span>

    {#if hunkPositions.length > 0}
      <div class="nav-controls">
        <button
          class="nav-button"
          onclick={prevHunk}
          title="Previous change (P)"
        >
          <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <polyline points="15 18 9 12 15 6"></polyline>
          </svg>
        </button>
        <span class="nav-indicator">
          {currentHunkIndex >= 0 ? currentHunkIndex + 1 : '-'} / {hunkPositions.length}
        </span>
        <button
          class="nav-button"
          onclick={nextHunk}
          title="Next change (N)"
        >
          <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <polyline points="9 18 15 12 9 6"></polyline>
          </svg>
        </button>
      </div>
    {/if}

    <button
      class="sync-toggle"
      class:active={syncScroll}
      onclick={() => (syncScroll = !syncScroll)}
      title={syncScroll ? 'Disable sync scroll' : 'Enable sync scroll'}
    >
      <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
        <polyline points="17 1 21 5 17 9"></polyline>
        <path d="M3 11V9a4 4 0 0 1 4-4h14"></path>
        <polyline points="7 23 3 19 7 15"></polyline>
        <path d="M21 13v2a4 4 0 0 1-4 4H3"></path>
      </svg>
      Sync
    </button>
  </div>
</div>

<style>
  .diff-view {
    display: flex;
    flex-direction: column;
    flex: 1;
    min-height: 0;
  }

  .search-bar {
    display: flex;
    align-items: center;
    gap: var(--spacing-sm);
    padding: var(--spacing-sm) var(--spacing-md);
    background: var(--color-bg-secondary);
    border-bottom: 1px solid var(--color-border);
  }

  .search-input-wrapper {
    display: flex;
    align-items: center;
    flex: 1;
    max-width: 400px;
    background: var(--color-bg-primary);
    border: 1px solid var(--color-border);
    border-radius: var(--radius-md);
    padding: 0 var(--spacing-sm);
  }

  .search-input-wrapper:focus-within {
    border-color: var(--color-accent-primary);
  }

  .search-icon {
    color: var(--color-text-muted);
    flex-shrink: 0;
  }

  .search-input {
    flex: 1;
    padding: var(--spacing-xs) var(--spacing-sm);
    background: transparent;
    border: none;
    font-size: var(--font-size-sm);
    outline: none;
  }

  .search-count {
    font-size: var(--font-size-xs);
    color: var(--color-text-muted);
    white-space: nowrap;
    padding-right: var(--spacing-xs);
  }

  .search-nav-btn,
  .search-close-btn {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 28px;
    height: 28px;
    border-radius: var(--radius-sm);
    color: var(--color-text-muted);
    background: transparent;
    border: 1px solid var(--color-border);
    transition: all var(--transition-fast);
  }

  .search-nav-btn:hover:not(:disabled),
  .search-close-btn:hover {
    background: var(--color-bg-hover);
    color: var(--color-text-primary);
  }

  .search-nav-btn:disabled {
    opacity: 0.4;
    cursor: not-allowed;
  }

  .diff-container {
    display: flex;
    flex: 1;
    min-height: 0;
    border: 1px solid var(--color-border);
    border-radius: var(--radius-md);
    overflow: hidden;
  }

  .diff-gutter {
    width: 4px;
    background: var(--color-border);
    flex-shrink: 0;
  }

  .diff-stats {
    display: flex;
    gap: var(--spacing-lg);
    padding: var(--spacing-sm) var(--spacing-md);
    font-size: var(--font-size-sm);
    color: var(--color-text-muted);
  }

  .stat {
    font-family: var(--font-mono);
  }

  .stat.additions {
    color: var(--color-diff-insert-text);
  }

  .stat.deletions {
    color: var(--color-diff-delete-text);
  }

  .nav-controls {
    display: flex;
    align-items: center;
    gap: var(--spacing-xs);
    margin-left: var(--spacing-lg);
  }

  .nav-button {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 24px;
    height: 24px;
    border-radius: var(--radius-sm);
    color: var(--color-text-muted);
    background: transparent;
    border: 1px solid var(--color-border);
    transition: all var(--transition-fast);
  }

  .nav-button:hover {
    background: var(--color-bg-hover);
    color: var(--color-text-primary);
    border-color: var(--color-border-hover);
  }

  .nav-indicator {
    font-family: var(--font-mono);
    font-size: var(--font-size-xs);
    color: var(--color-text-secondary);
    min-width: 4em;
    text-align: center;
  }

  .sync-toggle {
    display: flex;
    align-items: center;
    gap: var(--spacing-xs);
    margin-left: auto;
    padding: var(--spacing-xs) var(--spacing-sm);
    border-radius: var(--radius-sm);
    font-size: var(--font-size-xs);
    color: var(--color-text-muted);
    background: transparent;
    border: 1px solid var(--color-border);
    transition: all var(--transition-fast);
  }

  .sync-toggle:hover {
    background: var(--color-bg-hover);
    color: var(--color-text-secondary);
  }

  .sync-toggle.active {
    background: var(--color-bg-tertiary);
    color: var(--color-accent-primary);
    border-color: var(--color-accent-primary);
  }
</style>
