<script lang="ts">
  interface HunkRange {
    start: number;
    end: number;
  }

  interface Props {
    hunkRanges: HunkRange[];
    lineHeight: number;
    onCopyToRight: (hunkIndex: number) => void;
    onCopyToLeft: (hunkIndex: number) => void;
    scrollTop?: number;
  }

  let { hunkRanges, lineHeight, onCopyToRight, onCopyToLeft, scrollTop = 0 }: Props = $props();
</script>

<div class="diff-gutter">
  {#each hunkRanges as hunk, i (i)}
    {@const top = hunk.start * lineHeight - scrollTop}
    {@const height = (hunk.end - hunk.start) * lineHeight}
    {@const midY = top + height / 2 - 10}
    {#if midY > -20 && midY < 2000}
      <div class="hunk-buttons" style="top: {midY}px">
        <button
          class="copy-btn copy-right"
          onclick={() => onCopyToRight(i)}
          title="Copy to right"
        >
          <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <polyline points="9 18 15 12 9 6"></polyline>
          </svg>
        </button>
        <button
          class="copy-btn copy-left"
          onclick={() => onCopyToLeft(i)}
          title="Copy to left"
        >
          <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <polyline points="15 18 9 12 15 6"></polyline>
          </svg>
        </button>
      </div>
    {/if}
  {/each}
</div>

<style>
  .diff-gutter {
    width: 32px;
    background: var(--color-bg-secondary);
    border-left: 1px solid var(--color-border);
    border-right: 1px solid var(--color-border);
    flex-shrink: 0;
    position: relative;
    overflow: hidden;
  }

  .hunk-buttons {
    position: absolute;
    left: 50%;
    transform: translateX(-50%);
    display: flex;
    flex-direction: column;
    gap: 2px;
  }

  .copy-btn {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 20px;
    height: 20px;
    border-radius: var(--radius-sm);
    background: var(--color-bg-primary);
    border: 1px solid var(--color-border);
    color: var(--color-text-muted);
    cursor: pointer;
    transition: all var(--transition-fast);
  }

  .copy-btn:hover {
    background: var(--color-accent-primary);
    border-color: var(--color-accent-primary);
    color: white;
  }

  .copy-right:hover {
    background: var(--color-diff-insert-text);
    border-color: var(--color-diff-insert-text);
  }

  .copy-left:hover {
    background: var(--color-diff-delete-text);
    border-color: var(--color-diff-delete-text);
  }
</style>
