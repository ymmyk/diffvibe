<script lang="ts">
  import type { FileContent } from '$lib/types';

  interface PaneLine {
    lineNum: number | null;
    content: string;
    highlightedHtml?: string;
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
    content?: string;
    onContentChange?: (newContent: string) => void;
    onFocus?: () => void;
    dirty?: boolean;
  }

  let { file, lines, side, scrollRef = $bindable(null), onscroll, searchQuery = '', currentMatchRow = -1, content = '', onContentChange, onFocus, dirty = false }: Props = $props();

  // Copy to clipboard state
  let showCopied = $state(false);
  let copiedTimeout: ReturnType<typeof setTimeout> | null = null;

  async function copyPath() {
    try {
      await navigator.clipboard.writeText(file.path);
      showCopied = true;
      if (copiedTimeout) clearTimeout(copiedTimeout);
      copiedTimeout = setTimeout(() => {
        showCopied = false;
      }, 1500);
    } catch (e) {
      console.error('Failed to copy:', e);
    }
  }

  function formatSize(bytes: number): string {
    if (bytes < 1024) return `${bytes} B`;
    if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
    return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
  }

  function getFileName(path: string): string {
    return path.split('/').pop() || path.split('\\').pop() || path;
  }

  function getDirPath(path: string): string {
    const sep = path.includes('/') ? '/' : '\\';
    const parts = path.split(sep);
    if (parts.length <= 1) return '';
    return parts.slice(0, -1).join(sep) + sep;
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

  // Build content string from current lines (excluding empty placeholder lines)
  function rebuildContent(): string {
    const contentLines: string[] = [];
    for (const line of lines) {
      if (line.tag !== 'empty' && line.lineNum !== null) {
        contentLines.push(line.content);
      }
    }
    return contentLines.join('');
  }

  function handleInput(e: Event, lineIndex: number) {
    if (!onContentChange) return;

    const target = e.target as HTMLElement;
    const newLineContent = target.textContent || '';

    // Build new content by replacing this line
    const contentLines: string[] = [];
    for (let i = 0; i < lines.length; i++) {
      const line = lines[i];
      if (line.tag === 'empty' || line.lineNum === null) continue;

      if (i === lineIndex) {
        // Use the new content, preserve newline if original had one
        const hadNewline = line.content.endsWith('\n');
        contentLines.push(hadNewline ? newLineContent + '\n' : newLineContent);
      } else {
        contentLines.push(line.content);
      }
    }

    onContentChange(contentLines.join(''));
  }
</script>

<div class="diff-pane">
  <header class="pane-header">
    <button class="file-path" title="Click to copy path" onclick={copyPath}>
      <span class="dir-path">{getDirPath(file.path)}</span><span class="file-name">{getFileName(file.path)}</span>{#if !file.exists}<span class="new-file-indicator"> (new)</span>{/if}{#if dirty}<span class="dirty-indicator"> •</span>{/if}
      {#if showCopied}
        <span class="copied-toast">Copied!</span>
      {/if}
    </button>
    <span class="file-meta">
      {#if file.exists}
        {formatSize(file.size)} | {file.line_count} lines | {file.encoding}
      {:else}
        File does not exist yet
      {/if}
    </span>
  </header>

  <div class="pane-content" bind:this={scrollRef} onscroll={onscroll} onfocusin={onFocus}>
    {#if !file.exists && lines.length === 0}
      <div class="new-file-placeholder">
        <p>This file does not exist yet.</p>
        <p class="hint">Copy content from the other side or add content to create it.</p>
      </div>
    {:else}
      {#each lines as line, i (i)}
        <div class="line" class:line-equal={line.tag === 'equal'} class:line-insert={line.tag === 'insert'} class:line-delete={line.tag === 'delete'} class:line-empty={line.tag === 'empty'} class:current-match={i === currentMatchRow}>
          <span class="line-num">{line.lineNum ?? ''}</span>
          {#if line.tag !== 'empty' && onContentChange}
            <span
              class="line-content"
              contenteditable="true"
              oninput={(e) => handleInput(e, i)}
            >{line.content.replace(/\n$/, '')}</span>
          {:else if line.highlightedHtml && !searchQuery}
            <!-- Use syntax-highlighted HTML when available and no search active -->
            <span class="line-content">{@html line.highlightedHtml}</span>
          {:else}
            <!-- Fallback to plain text with search highlighting -->
            <span class="line-content">{#each highlightText(line.content, searchQuery) as part}{#if part.highlight}<mark class="search-highlight">{part.text}</mark>{:else}{part.text}{/if}{/each}</span>
          {/if}
        </div>
      {/each}
    {/if}
  </div>
</div>

<style>
  .diff-pane {
    flex: 1;
    display: flex;
    flex-direction: column;
    min-width: 0;
  }

  .pane-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: var(--spacing-sm) var(--spacing-md);
    background: var(--color-bg-secondary);
    border-bottom: 1px solid var(--color-border);
    flex-shrink: 0;
    overflow: visible;
    position: relative;
    z-index: 5;
  }

  .file-path {
    position: relative;
    display: flex;
    font-size: var(--font-size-sm);
    overflow: hidden;
    min-width: 0;
    background: none;
    border: none;
    padding: 0;
    cursor: pointer;
    text-align: left;
    color: inherit;
    border-radius: var(--radius-sm);
  }

  .file-path:hover {
    background: var(--color-bg-hover);
  }

  .dir-path {
    color: var(--color-text-muted);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    flex-shrink: 1;
    min-width: 20px;
  }

  .file-name {
    font-weight: 600;
    white-space: nowrap;
    flex-shrink: 0;
  }

  .copied-toast {
    margin-left: var(--spacing-sm);
    padding: 2px 6px;
    background: var(--color-diff-insert-bg);
    border-radius: var(--radius-sm);
    font-size: var(--font-size-xs);
    font-weight: 500;
    color: var(--color-diff-insert-text);
    white-space: nowrap;
    animation: fadeOut 1.5s ease-in-out forwards;
  }

  @keyframes fadeOut {
    0% { opacity: 1; }
    70% { opacity: 1; }
    100% { opacity: 0; }
  }

  .dirty-indicator {
    color: var(--color-accent-primary);
    font-weight: bold;
  }

  .new-file-indicator {
    color: var(--color-diff-insert-text);
    font-weight: 500;
    font-size: var(--font-size-xs);
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

  .new-file-placeholder {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    height: 100%;
    color: var(--color-text-muted);
    text-align: center;
    padding: var(--spacing-xl);
  }

  .new-file-placeholder .hint {
    font-size: var(--font-size-xs);
    margin-top: var(--spacing-sm);
    color: var(--color-text-disabled);
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
  }

  .line-content[contenteditable="true"] {
    outline: none;
    cursor: text;
  }

  .line-content[contenteditable="true"]:focus {
    background: rgba(255, 255, 255, 0.05);
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
