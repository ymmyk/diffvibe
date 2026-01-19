<script lang="ts">
  import { open } from "@tauri-apps/plugin-dialog";
  import { invoke } from "@tauri-apps/api/core";

  type CompareMode = "file" | "directory" | "merge";

  let mode: CompareMode = $state("file");
  let leftPath = $state("");
  let rightPath = $state("");

  async function selectLeft() {
    const selected = await open({
      multiple: false,
      directory: mode === "directory",
    });
    if (selected) {
      leftPath = selected as string;
    }
  }

  async function selectRight() {
    const selected = await open({
      multiple: false,
      directory: mode === "directory",
    });
    if (selected) {
      rightPath = selected as string;
    }
  }

  function startComparison() {
    if (!leftPath || !rightPath) return;
    // TODO: Navigate to comparison view
    console.log("Comparing:", leftPath, "vs", rightPath);
  }
</script>

<div class="app">
  <header class="header">
    <h1>DiffVibe</h1>
    <p class="tagline">Visual Diff & Merge Tool</p>
  </header>

  <main class="main">
    <section class="mode-selector">
      <button
        class="mode-button"
        class:active={mode === "file"}
        onclick={() => (mode = "file")}
      >
        <span class="icon">&#128196;</span>
        <span>File Comparison</span>
      </button>
      <button
        class="mode-button"
        class:active={mode === "directory"}
        onclick={() => (mode = "directory")}
      >
        <span class="icon">&#128193;</span>
        <span>Directory Comparison</span>
      </button>
      <button
        class="mode-button"
        class:active={mode === "merge"}
        onclick={() => (mode = "merge")}
      >
        <span class="icon">&#128279;</span>
        <span>Three-Way Merge</span>
      </button>
    </section>

    <section class="file-selector">
      <div class="file-input">
        <label>
          {mode === "directory" ? "Left Directory" : "Left File"}
        </label>
        <div class="input-row">
          <input
            type="text"
            placeholder={mode === "directory"
              ? "Select a directory..."
              : "Select a file..."}
            bind:value={leftPath}
            readonly
          />
          <button onclick={selectLeft}>Browse...</button>
        </div>
      </div>

      <div class="file-input">
        <label>
          {mode === "directory" ? "Right Directory" : "Right File"}
        </label>
        <div class="input-row">
          <input
            type="text"
            placeholder={mode === "directory"
              ? "Select a directory..."
              : "Select a file..."}
            bind:value={rightPath}
            readonly
          />
          <button onclick={selectRight}>Browse...</button>
        </div>
      </div>

      {#if mode === "merge"}
        <div class="file-input">
          <label>Base File (Common Ancestor)</label>
          <div class="input-row">
            <input type="text" placeholder="Select base file..." readonly />
            <button>Browse...</button>
          </div>
        </div>
      {/if}
    </section>

    <section class="actions">
      <button
        class="compare-button"
        onclick={startComparison}
        disabled={!leftPath || !rightPath}
      >
        Compare
      </button>
    </section>

    <section class="recent">
      <h2>Recent Comparisons</h2>
      <p class="empty-state">No recent comparisons</p>
    </section>
  </main>
</div>

<style>
  :global(*) {
    box-sizing: border-box;
    margin: 0;
    padding: 0;
  }

  :global(body) {
    font-family:
      -apple-system,
      BlinkMacSystemFont,
      "Segoe UI",
      Roboto,
      Oxygen,
      Ubuntu,
      Cantarell,
      sans-serif;
    background-color: #1a1a2e;
    color: #eee;
  }

  .app {
    min-height: 100vh;
    display: flex;
    flex-direction: column;
  }

  .header {
    padding: 2rem;
    text-align: center;
    background: linear-gradient(135deg, #16213e 0%, #1a1a2e 100%);
    border-bottom: 1px solid #2a2a4a;
  }

  .header h1 {
    font-size: 2.5rem;
    font-weight: 700;
    background: linear-gradient(135deg, #00d9ff, #00ff88);
    -webkit-background-clip: text;
    -webkit-text-fill-color: transparent;
    background-clip: text;
  }

  .tagline {
    color: #888;
    margin-top: 0.5rem;
  }

  .main {
    flex: 1;
    padding: 2rem;
    max-width: 800px;
    margin: 0 auto;
    width: 100%;
  }

  .mode-selector {
    display: flex;
    gap: 1rem;
    margin-bottom: 2rem;
  }

  .mode-button {
    flex: 1;
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 0.5rem;
    padding: 1.5rem 1rem;
    background: #16213e;
    border: 2px solid #2a2a4a;
    border-radius: 12px;
    color: #888;
    cursor: pointer;
    transition: all 0.2s;
  }

  .mode-button:hover {
    border-color: #3a3a6a;
    color: #ccc;
  }

  .mode-button.active {
    border-color: #00d9ff;
    color: #fff;
    background: #1a2a4e;
  }

  .mode-button .icon {
    font-size: 2rem;
  }

  .file-selector {
    display: flex;
    flex-direction: column;
    gap: 1.5rem;
    margin-bottom: 2rem;
  }

  .file-input label {
    display: block;
    margin-bottom: 0.5rem;
    font-weight: 500;
    color: #aaa;
  }

  .input-row {
    display: flex;
    gap: 0.5rem;
  }

  .input-row input {
    flex: 1;
    padding: 0.75rem 1rem;
    background: #16213e;
    border: 1px solid #2a2a4a;
    border-radius: 8px;
    color: #eee;
    font-size: 0.9rem;
  }

  .input-row input:focus {
    outline: none;
    border-color: #00d9ff;
  }

  .input-row button {
    padding: 0.75rem 1.5rem;
    background: #2a2a4a;
    border: 1px solid #3a3a6a;
    border-radius: 8px;
    color: #eee;
    cursor: pointer;
    transition: all 0.2s;
  }

  .input-row button:hover {
    background: #3a3a6a;
  }

  .actions {
    display: flex;
    justify-content: center;
    margin-bottom: 3rem;
  }

  .compare-button {
    padding: 1rem 3rem;
    background: linear-gradient(135deg, #00d9ff, #00ff88);
    border: none;
    border-radius: 8px;
    color: #1a1a2e;
    font-size: 1.1rem;
    font-weight: 600;
    cursor: pointer;
    transition: all 0.2s;
  }

  .compare-button:hover:not(:disabled) {
    transform: translateY(-2px);
    box-shadow: 0 4px 20px rgba(0, 217, 255, 0.3);
  }

  .compare-button:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .recent h2 {
    font-size: 1.1rem;
    color: #888;
    margin-bottom: 1rem;
  }

  .empty-state {
    color: #555;
    text-align: center;
    padding: 2rem;
    background: #16213e;
    border-radius: 8px;
    border: 1px dashed #2a2a4a;
  }
</style>
