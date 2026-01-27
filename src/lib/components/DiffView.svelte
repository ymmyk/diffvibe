<script lang="ts">
  import { onMount, untrack } from 'svelte';
  import { invoke } from '@tauri-apps/api/core';
  import type { FileDiffResult, DiffLine, DiffResult } from '$lib/types';
  import DiffPane from './DiffPane.svelte';
  import DiffGutter from './DiffGutter.svelte';
  import { createHistory, push, undo, redo, canUndo, canRedo, reset, type History } from '$lib/utils/history';
  import { highlightLines } from '$lib/utils/syntax';
  import { syntaxThemeStore } from '$lib/stores';

  interface Props {
    result: FileDiffResult;
    onDirtyChange?: (leftDirty: boolean, rightDirty: boolean) => void;
    onSaveLeft?: (content: string) => Promise<void>;
    onSaveRight?: (content: string) => Promise<void>;
  }

  let { result, onDirtyChange, onSaveLeft, onSaveRight }: Props = $props();

  // Editable content state - starts from file content, can diverge on edit
  let leftContent = $state('');
  let rightContent = $state('');

  // Local diff result that updates when content changes
  let localDiff = $state<DiffResult | null>(null);
  let diffDebounceTimer: ReturnType<typeof setTimeout> | null = null;

  // Track original content to detect dirty state
  let originalLeftContent = $state('');
  let originalRightContent = $state('');

  // Track which result we've initialized from to detect changes
  let lastResultId = $state('');

  // Undo/redo history per pane
  let leftHistory = $state<History<string>>(createHistory(''));
  let rightHistory = $state<History<string>>(createHistory(''));

  // Debounce timer for coalescing rapid edits into single history entry
  let leftHistoryTimer: ReturnType<typeof setTimeout> | null = null;
  let rightHistoryTimer: ReturnType<typeof setTimeout> | null = null;
  const HISTORY_DEBOUNCE_MS = 500;

  // Dirty state
  const leftDirty = $derived(leftContent !== originalLeftContent);
  const rightDirty = $derived(rightContent !== originalRightContent);

  // Notify parent of dirty state changes
  // Track previous values to only notify on actual changes
  let prevLeftDirty = false;
  let prevRightDirty = false;

  $effect(() => {
    const left = leftDirty;
    const right = rightDirty;
    if (left !== prevLeftDirty || right !== prevRightDirty) {
      prevLeftDirty = left;
      prevRightDirty = right;
      untrack(() => onDirtyChange?.(left, right));
    }
  });

  // Reset content when result changes (new file loaded)
  // Use a unique ID based on file paths to detect changes
  $effect(() => {
    const resultId = `${result.left.path}:${result.right.path}`;
    if (resultId !== lastResultId) {
      lastResultId = resultId;
      leftContent = result.left.content;
      rightContent = result.right.content;
      originalLeftContent = result.left.content;
      originalRightContent = result.right.content;
      localDiff = result.diff;
      // Reset history for new files
      leftHistory = createHistory(result.left.content);
      rightHistory = createHistory(result.right.content);
    }
  });

  // Save functions
  async function saveLeft() {
    if (onSaveLeft && leftDirty) {
      await onSaveLeft(leftContent);
      originalLeftContent = leftContent;
    }
  }

  async function saveRight() {
    if (onSaveRight && rightDirty) {
      await onSaveRight(rightContent);
      originalRightContent = rightContent;
    }
  }

  // Save both if dirty
  async function saveAll() {
    if (leftDirty) await saveLeft();
    if (rightDirty) await saveRight();
  }

  // Recompute diff when content changes (debounced)
  async function recomputeDiff() {
    const perfStart = performance.now();
    const leftLines = leftContent.split('\n').length;
    const rightLines = rightContent.split('\n').length;
    console.log(`[DiffView] recomputeDiff starting: left=${leftLines} lines, right=${rightLines} lines`);
    
    try {
      const invokeStart = performance.now();
      const newDiff = await invoke<DiffResult>('compute_diff', {
        left: leftContent,
        right: rightContent
      });
      const invokeTime = performance.now() - invokeStart;
      console.log(`[DiffView] Rust compute_diff took ${invokeTime.toFixed(1)}ms`);
      
      const updateStart = performance.now();
      localDiff = newDiff;
      const updateTime = performance.now() - updateStart;
      console.log(`[DiffView] UI update took ${updateTime.toFixed(1)}ms`);
      
      const totalTime = performance.now() - perfStart;
      console.log(`[DiffView] recomputeDiff complete: ${totalTime.toFixed(1)}ms total (${newDiff.lines.length} diff lines)`);
    } catch (e) {
      console.error('Failed to recompute diff:', e);
    }
  }

  function handleLeftContentChange(newContent: string) {
    leftContent = newContent;
    if (diffDebounceTimer) clearTimeout(diffDebounceTimer);
    diffDebounceTimer = setTimeout(recomputeDiff, 300);

    // Debounced history push to coalesce rapid typing
    if (leftHistoryTimer) clearTimeout(leftHistoryTimer);
    leftHistoryTimer = setTimeout(() => {
      leftHistory = push(leftHistory, newContent);
    }, HISTORY_DEBOUNCE_MS);
  }

  function handleRightContentChange(newContent: string) {
    rightContent = newContent;
    if (diffDebounceTimer) clearTimeout(diffDebounceTimer);
    diffDebounceTimer = setTimeout(recomputeDiff, 300);

    // Debounced history push to coalesce rapid typing
    if (rightHistoryTimer) clearTimeout(rightHistoryTimer);
    rightHistoryTimer = setTimeout(() => {
      rightHistory = push(rightHistory, newContent);
    }, HISTORY_DEBOUNCE_MS);
  }

  // Undo/redo functions
  function undoLeft() {
    if (!canUndo(leftHistory)) return;
    leftHistory = undo(leftHistory);
    leftContent = leftHistory.present;
    recomputeDiff();
  }

  function redoLeft() {
    if (!canRedo(leftHistory)) return;
    leftHistory = redo(leftHistory);
    leftContent = leftHistory.present;
    recomputeDiff();
  }

  function undoRight() {
    if (!canUndo(rightHistory)) return;
    rightHistory = undo(rightHistory);
    rightContent = rightHistory.present;
    recomputeDiff();
  }

  function redoRight() {
    if (!canRedo(rightHistory)) return;
    rightHistory = redo(rightHistory);
    rightContent = rightHistory.present;
    recomputeDiff();
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
    highlightedHtml?: string; // Optional syntax-highlighted HTML
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

  const paneLines = $derived(localDiff ? buildPaneLines(localDiff.lines) : { left: [], right: [] });
  
  // Syntax highlighting state
  let highlightedLeft: string[] = $state([]);
  let highlightedRight: string[] = $state([]);
  let isHighlighting = $state(false);
  
  // Apply syntax highlighting when pane lines or theme changes
  $effect(() => {
    // Watch the store value to trigger re-highlighting on theme change
    const currentTheme = syntaxThemeStore.value;
    const leftPath = result.left.path;
    const rightPath = result.right.path;
    const leftLines = paneLines.left.map(l => l.content.replace(/\n$/, ''));
    const rightLines = paneLines.right.map(l => l.content.replace(/\n$/, ''));
    
    console.log('[DiffView] Starting syntax highlighting:', { leftPath, rightPath, lineCount: leftLines.length, theme: currentTheme });
    
    // Reset highlighting when lines change
    highlightedLeft = [];
    highlightedRight = [];
    isHighlighting = true;
    
    // Run highlighting asynchronously
    Promise.all([
      highlightLines(leftLines, leftPath),
      highlightLines(rightLines, rightPath)
    ]).then(([left, right]) => {
      console.log('[DiffView] Highlighting complete:', { leftLines: left.length, rightLines: right.length, sample: left[0]?.substring(0, 100) });
      highlightedLeft = left;
      highlightedRight = right;
      isHighlighting = false;
    }).catch(err => {
      console.error('[DiffView] Syntax highlighting failed:', err);
      isHighlighting = false;
    });
  });
  
  // Attach highlighted HTML to pane lines
  const paneLinesWithHighlighting = $derived.by(() => {
    const left = paneLines.left.map((line, i) => ({
      ...line,
      highlightedHtml: highlightedLeft[i] || undefined
    }));
    const right = paneLines.right.map((line, i) => ({
      ...line,
      highlightedHtml: highlightedRight[i] || undefined
    }));
    return { left, right };
  });

  // Compute hunk ranges (start/end row indices for each change block)
  interface HunkRange {
    start: number;
    end: number; // exclusive
  }

  const hunkRanges = $derived.by(() => {
    const ranges: HunkRange[] = [];
    let inChange = false;
    let start = 0;

    for (let i = 0; i < paneLinesWithHighlighting.left.length; i++) {
      const leftTag = paneLinesWithHighlighting.left[i].tag;
      const rightTag = paneLinesWithHighlighting.right[i].tag;
      const isChange = leftTag !== 'equal' || rightTag !== 'equal';

      if (isChange && !inChange) {
        start = i;
        inChange = true;
      } else if (!isChange && inChange) {
        ranges.push({ start, end: i });
        inChange = false;
      }
    }

    // Handle hunk at end of file
    if (inChange) {
      ranges.push({ start, end: paneLinesWithHighlighting.left.length });
    }

    return ranges;
  });

  // For backwards compat with navigation
  const hunkPositions = $derived(hunkRanges.map(r => r.start));

  // Copy hunk from left to right (replace right content with left content for this hunk)
  function copyHunkToRight(hunkIndex: number) {
    const hunk = hunkRanges[hunkIndex];
    if (!hunk) return;

    // Get the left side content for this hunk (non-empty lines)
    const leftLines: string[] = [];
    for (let i = hunk.start; i < hunk.end; i++) {
      const line = paneLinesWithHighlighting.left[i];
      if (line.tag !== 'empty') {
        leftLines.push(line.content);
      }
    }

    // Build new right content by replacing lines in the hunk range
    const rightLines = rightContent.split(/(?<=\n)/); // split keeping newlines

    // Find which right line indices correspond to this hunk
    let rightStartLine = 0;
    let rightEndLine = 0;
    for (let i = 0; i < hunk.start; i++) {
      if (paneLinesWithHighlighting.right[i].lineNum !== null) rightStartLine++;
    }
    for (let i = 0; i < hunk.end; i++) {
      if (paneLinesWithHighlighting.right[i].lineNum !== null) rightEndLine++;
    }

    // Replace lines
    const newRightLines = [
      ...rightLines.slice(0, rightStartLine),
      ...leftLines,
      ...rightLines.slice(rightEndLine)
    ];

    const newContent = newRightLines.join('');
    rightContent = newContent;
    // Push to history immediately (no debounce for copy operations)
    rightHistory = push(rightHistory, newContent);
    recomputeDiff();
  }

  // Copy hunk from right to left
  function copyHunkToLeft(hunkIndex: number) {
    const hunk = hunkRanges[hunkIndex];
    if (!hunk) return;

    // Get the right side content for this hunk (non-empty lines)
    const rightLines: string[] = [];
    for (let i = hunk.start; i < hunk.end; i++) {
      const line = paneLinesWithHighlighting.right[i];
      if (line.tag !== 'empty') {
        rightLines.push(line.content);
      }
    }

    // Build new left content by replacing lines in the hunk range
    const leftLines = leftContent.split(/(?<=\n)/);

    // Find which left line indices correspond to this hunk
    let leftStartLine = 0;
    let leftEndLine = 0;
    for (let i = 0; i < hunk.start; i++) {
      if (paneLinesWithHighlighting.left[i].lineNum !== null) leftStartLine++;
    }
    for (let i = 0; i < hunk.end; i++) {
      if (paneLinesWithHighlighting.left[i].lineNum !== null) leftEndLine++;
    }

    // Replace lines
    const newLeftLines = [
      ...leftLines.slice(0, leftStartLine),
      ...rightLines,
      ...leftLines.slice(leftEndLine)
    ];

    const newContent = newLeftLines.join('');
    leftContent = newContent;
    // Push to history immediately (no debounce for copy operations)
    leftHistory = push(leftHistory, newContent);
    recomputeDiff();
  }

  // Copy all hunks in one direction
  function copyAllToRight() {
    rightContent = leftContent;
    // Push to history immediately
    rightHistory = push(rightHistory, leftContent);
    recomputeDiff();
  }

  function copyAllToLeft() {
    leftContent = rightContent;
    // Push to history immediately
    leftHistory = push(leftHistory, rightContent);
    recomputeDiff();
  }

  // Track scroll position for gutter
  let gutterScrollTop = $state(0);

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

    for (let i = 0; i < paneLinesWithHighlighting.left.length; i++) {
      const leftContent = paneLinesWithHighlighting.left[i].content.toLowerCase();
      const rightContent = paneLinesWithHighlighting.right[i].content.toLowerCase();
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

  // Track which pane has focus for save
  let focusedPane: 'left' | 'right' | null = $state(null);

  // Keyboard navigation
  onMount(() => {
    function handleKeydown(e: KeyboardEvent) {
      // Cmd/Ctrl+Z to undo, Cmd/Ctrl+Shift+Z to redo
      if ((e.metaKey || e.ctrlKey) && e.key === 'z') {
        e.preventDefault();
        if (e.shiftKey) {
          // Redo
          if (focusedPane === 'left') {
            redoLeft();
          } else if (focusedPane === 'right') {
            redoRight();
          }
        } else {
          // Undo
          if (focusedPane === 'left') {
            undoLeft();
          } else if (focusedPane === 'right') {
            undoRight();
          }
        }
        return;
      }

      // Cmd/Ctrl+S to save
      if ((e.metaKey || e.ctrlKey) && e.key === 's' && !e.shiftKey) {
        e.preventDefault();
        // Save focused pane, or both if none focused
        if (focusedPane === 'left' && leftDirty) {
          saveLeft();
        } else if (focusedPane === 'right' && rightDirty) {
          saveRight();
        } else {
          saveAll();
        }
        return;
      }

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

      // Skip hunk navigation if in input or editable
      if (e.target instanceof HTMLInputElement || e.target instanceof HTMLTextAreaElement) return;
      if ((e.target as HTMLElement)?.isContentEditable) return;

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
    const sourceEl = source === 'left' ? leftScrollRef : rightScrollRef;
    if (sourceEl) {
      gutterScrollTop = sourceEl.scrollTop;
    }

    if (!syncScroll || isScrolling) return;

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

  // Get line height for positioning
  const lineHeight = 21; // Match CSS line-height * font-size
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

  {#if localDiff}
  <div class="diff-container">
    <DiffPane
      file={result.left}
      lines={paneLinesWithHighlighting.left}
      side="left"
      bind:scrollRef={leftScrollRef}
      onscroll={() => handleScroll('left')}
      searchQuery={searchQuery}
      currentMatchRow={currentMatchIndex >= 0 ? searchMatches[currentMatchIndex] : -1}
      content={leftContent}
      onContentChange={handleLeftContentChange}
      onFocus={() => focusedPane = 'left'}
      dirty={leftDirty}
    />
    <DiffGutter
      hunkRanges={hunkRanges}
      lineHeight={lineHeight}
      scrollTop={gutterScrollTop}
      onCopyToRight={copyHunkToRight}
      onCopyToLeft={copyHunkToLeft}
    />
    <DiffPane
      file={result.right}
      lines={paneLinesWithHighlighting.right}
      side="right"
      bind:scrollRef={rightScrollRef}
      onscroll={() => handleScroll('right')}
      searchQuery={searchQuery}
      currentMatchRow={currentMatchIndex >= 0 ? searchMatches[currentMatchIndex] : -1}
      content={rightContent}
      onContentChange={handleRightContentChange}
      onFocus={() => focusedPane = 'right'}
      dirty={rightDirty}
    />
  </div>
  {/if}

  <div class="diff-stats">
    <span class="stat additions">+{localDiff?.stats.additions ?? 0}</span>
    <span class="stat deletions">-{localDiff?.stats.deletions ?? 0}</span>
    <span class="stat unchanged">{localDiff?.stats.unchanged ?? 0} unchanged</span>

    {#if hunkRanges.length > 0}
      <div class="copy-all-controls">
        <button class="copy-all-btn" onclick={copyAllToRight} title="Copy all changes to right">
          All
          <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <polyline points="9 18 15 12 9 6"></polyline>
          </svg>
        </button>
        <button class="copy-all-btn" onclick={copyAllToLeft} title="Copy all changes to left">
          <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <polyline points="15 18 9 12 15 6"></polyline>
          </svg>
          All
        </button>
      </div>
    {/if}

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

  .copy-all-controls {
    display: flex;
    align-items: center;
    gap: var(--spacing-xs);
    margin-left: var(--spacing-md);
  }

  .copy-all-btn {
    display: flex;
    align-items: center;
    gap: 2px;
    padding: var(--spacing-xs) var(--spacing-sm);
    border-radius: var(--radius-sm);
    font-size: var(--font-size-xs);
    color: var(--color-text-muted);
    background: transparent;
    border: 1px solid var(--color-border);
    transition: all var(--transition-fast);
  }

  .copy-all-btn:hover {
    background: var(--color-bg-hover);
    color: var(--color-text-primary);
    border-color: var(--color-border-hover);
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
