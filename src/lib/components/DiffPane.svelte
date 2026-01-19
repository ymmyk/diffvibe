<script lang="ts">
  import type { FileContent } from '$lib/types';

  interface PaneLine {
    lineNum: number | null;
    content: string;
    tag: 'equal' | 'insert' | 'delete' | 'empty';
  }

  interface Props {
    file: FileContent;
    lines: PaneLine[];
    side: 'left' | 'right';
    scrollRef?: HTMLDivElement | null;
    onscroll?: () => void;
    searchQuery?: string;
    currentMatchRow?: number;
  }

  let { file, lines, side, scrollRef = $bindable(null), onscroll, searchQuery = '', currentMatchRow = -1 }: Props = $props();

  function formatSize(bytes: number): string {
    if (bytes < 1024) return `${bytes} B`;
    if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
    return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
  }

  function getFileName(path: string): string {
    return path.split('/').pop() || path.split('\\').pop() || path;
  }

  function highlightText(text: string, query: string): { text: string; highlight: boolean }[] {
    if (!query.trim()) return [{ text, highlight: false }];

    const lowerText = text.toLowerCase();
    const lowerQuery = query.toLowerCase();
    const parts: { text: string; highlight: boolean }[] = [];
    let lastIndex = 0;

    let index = lowerText.indexOf(lowerQuery);
    while (index !== -1) {
      if (index > lastIndex) {
        parts.push({ text: text.slice(lastIndex, index), highlight: false });
      }
      parts.push({ text: text.slice(index, index + query.length), highlight: true });
      lastIndex = index + query.length;
      index = lowerText.indexOf(lowerQuery, lastIndex);
    }

    if (lastIndex < text.length) {
      parts.push({ text: text.slice(lastIndex), highlight: false });
    }

    return parts.length > 0 ? parts : [{ text, highlight: false }];
  }
</script>

<div class="diff-pane">
  <header class="pane-header">
    <span class="file-name" title={file.path}>{getFileName(file.path)}</span>
    <span class="file-meta">
      {formatSize(file.size)} | {file.line_count} lines | {file.encoding}
    </span>
  </header>

  <div class="pane-content" bind:this={scrollRef} onscroll={onscroll}>
    {#each lines as line, i (i)}
      <div class="line" class:line-equal={line.tag === 'equal'} class:line-insert={line.tag === 'insert'} class:line-delete={line.tag === 'delete'} class:line-empty={line.tag === 'empty'} class:current-match={i === currentMatchRow}>
        <span class="line-num">{line.lineNum ?? ''}</span>
        <span class="line-content">{#each highlightText(line.content, searchQuery) as part}{#if part.highlight}<mark class="search-highlight">{part.text}</mark>{:else}{part.text}{/if}{/each}</span>
      </div>
    {/each}
  </div>
</div>

<style>
  .diff-pane {
    flex: 1;
    display: flex;
    flex-direction: column;
    min-width: 0;
    overflow: hidden;
  }

  .pane-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: var(--spacing-sm) var(--spacing-md);
    background: var(--color-bg-secondary);
    border-bottom: 1px solid var(--color-border);
    flex-shrink: 0;
  }

  .file-name {
    font-weight: 600;
    font-size: var(--font-size-sm);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .file-meta {
    font-size: var(--font-size-xs);
    color: var(--color-text-muted);
    flex-shrink: 0;
    margin-left: var(--spacing-md);
  }

  .pane-content {
    flex: 1;
    overflow: auto;
    font-family: var(--font-mono);
    font-size: var(--font-size-sm);
    line-height: 1.5;
  }

  .line {
    display: flex;
    min-height: 1.5em;
  }

  .line-num {
    width: 4em;
    padding: 0 var(--spacing-sm);
    text-align: right;
    color: var(--color-text-disabled);
    background: var(--color-bg-secondary);
    flex-shrink: 0;
    user-select: none;
    border-right: 1px solid var(--color-border);
  }

  .line-content {
    flex: 1;
    padding: 0 var(--spacing-sm);
    white-space: pre;
    overflow-x: auto;
  }

  .line-equal {
    background: transparent;
  }

  .line-insert {
    background: var(--color-diff-insert-bg);
  }

  .line-insert .line-num {
    background: var(--color-diff-insert-bg);
    color: var(--color-diff-insert-text);
  }

  .line-insert .line-content {
    color: var(--color-diff-insert-text);
  }

  .line-delete {
    background: var(--color-diff-delete-bg);
  }

  .line-delete .line-num {
    background: var(--color-diff-delete-bg);
    color: var(--color-diff-delete-text);
  }

  .line-delete .line-content {
    color: var(--color-diff-delete-text);
  }

  .line-empty {
    background: var(--color-bg-tertiary);
  }

  .line-empty .line-num {
    background: var(--color-bg-tertiary);
  }

  .search-highlight {
    background: var(--color-accent-secondary);
    color: var(--color-bg-primary);
    border-radius: 2px;
    padding: 0 1px;
  }

  .current-match {
    outline: 2px solid var(--color-accent-primary);
    outline-offset: -2px;
  }

  .current-match .search-highlight {
    background: var(--color-accent-primary);
    font-weight: 600;
  }
</style>
