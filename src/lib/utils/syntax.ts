import { createHighlighter, type Highlighter, type BundledLanguage, type BundledTheme } from 'shiki';

let highlighterInstance: Highlighter | null = null;
let initPromise: Promise<Highlighter> | null = null;
let currentTheme: BundledTheme = 'github-dark';

const SUPPORTED_LANGUAGES: BundledLanguage[] = [
  'javascript',
  'typescript',
  'python',
  'rust',
  'go',
  'java',
  'c',
  'cpp',
  'csharp',
  'html',
  'css',
  'scss',
  'json',
  'yaml',
  'toml',
  'markdown',
  'shell',
  'bash',
  'svelte',
  'vue',
  'jsx',
  'tsx',
];

// Available themes grouped by type
export const AVAILABLE_THEMES = {
  dark: [
    { id: 'github-dark', name: 'GitHub Dark' },
    { id: 'github-dark-dimmed', name: 'GitHub Dark Dimmed' },
    { id: 'vitesse-dark', name: 'Vitesse Dark' },
    { id: 'dracula', name: 'Dracula' },
    { id: 'monokai', name: 'Monokai' },
    { id: 'nord', name: 'Nord' },
    { id: 'one-dark-pro', name: 'One Dark Pro' },
    { id: 'tokyo-night', name: 'Tokyo Night' },
  ],
  light: [
    { id: 'github-light', name: 'GitHub Light' },
    { id: 'vitesse-light', name: 'Vitesse Light' },
    { id: 'min-light', name: 'Min Light' },
    { id: 'one-light', name: 'One Light' },
  ],
} as const;

// All theme IDs for initialization
const ALL_THEME_IDS: BundledTheme[] = [
  ...AVAILABLE_THEMES.dark.map(t => t.id as BundledTheme),
  ...AVAILABLE_THEMES.light.map(t => t.id as BundledTheme),
];

/**
 * Initialize the syntax highlighter (lazy, called once)
 * Loads all themes at initialization for instant switching
 */
export async function getHighlighter(): Promise<Highlighter> {
  if (highlighterInstance) {
    return highlighterInstance;
  }
  
  // Avoid multiple concurrent initializations
  if (initPromise) {
    return initPromise;
  }
  
  initPromise = createHighlighter({
    themes: ALL_THEME_IDS,
    langs: SUPPORTED_LANGUAGES,
  }).then(h => {
    highlighterInstance = h;
    initPromise = null;
    return h;
  });
  
  return initPromise;
}

/**
 * Set the current syntax theme
 */
export function setTheme(theme: BundledTheme) {
  currentTheme = theme;
}

/**
 * Get the current syntax theme
 */
export function getTheme(): BundledTheme {
  return currentTheme;
}

/**
 * Detect language from file extension
 */
export function detectLanguage(filePath: string): BundledLanguage | 'plaintext' {
  const ext = filePath.split('.').pop()?.toLowerCase() || '';
  
  const langMap: Record<string, BundledLanguage> = {
    // JavaScript/TypeScript
    'js': 'javascript',
    'mjs': 'javascript',
    'cjs': 'javascript',
    'ts': 'typescript',
    'tsx': 'tsx',
    'jsx': 'jsx',
    'mts': 'typescript',
    'cts': 'typescript',
    
    // Web
    'html': 'html',
    'htm': 'html',
    'css': 'css',
    'scss': 'scss',
    'sass': 'scss',
    'svelte': 'svelte',
    'vue': 'vue',
    
    // Config
    'json': 'json',
    'yaml': 'yaml',
    'yml': 'yaml',
    'toml': 'toml',
    
    // Rust
    'rs': 'rust',
    
    // Python
    'py': 'python',
    'pyw': 'python',
    
    // Go
    'go': 'go',
    
    // C family
    'c': 'c',
    'h': 'c',
    'cpp': 'cpp',
    'cc': 'cpp',
    'cxx': 'cpp',
    'hpp': 'cpp',
    'hxx': 'cpp',
    'cs': 'csharp',
    
    // Java
    'java': 'java',
    
    // Shell
    'sh': 'shell',
    'bash': 'bash',
    'zsh': 'bash',
    
    // Markdown
    'md': 'markdown',
    'markdown': 'markdown',
  };
  
  return langMap[ext] || 'plaintext';
}

/**
 * Highlight lines of code and return array of HTML strings (one per line)
 * This preserves line structure for diff display
 */
export async function highlightLines(lines: string[], filePath: string): Promise<string[]> {
  try {
    console.log('[syntax] highlightLines called:', { filePath, lineCount: lines.length, currentTheme });
    const highlighter = await getHighlighter();
    console.log('[syntax] Highlighter loaded');
    const lang = detectLanguage(filePath);
    console.log('[syntax] Detected language:', lang);
    
    if (lang === 'plaintext') {
      // No highlighting for plain text
      console.log('[syntax] Using plaintext (no highlighting)');
      return lines.map(escapeHtml);
    }
    
    // Highlight the full content to maintain context across lines
    const fullContent = lines.join('\n');
    console.log('[syntax] Highlighting content, length:', fullContent.length);
    
    // Use codeToHtml and split by lines instead
    const html = highlighter.codeToHtml(fullContent, {
      lang,
      theme: currentTheme,
    });
    
    // Extract the code content from <pre><code>...</code></pre>
    const codeMatch = html.match(/<code[^>]*>([\s\S]*?)<\/code>/);
    if (!codeMatch) {
      console.log('[syntax] Failed to extract code from HTML');
      return lines.map(escapeHtml);
    }
    
    // Split by line breaks, preserving the HTML spans
    const codeHtml = codeMatch[1];
    const htmlLines = codeHtml.split('\n');
    
    console.log('[syntax] Generated HTML lines:', htmlLines.length, 'sample:', htmlLines[0]?.substring(0, 150));
    return htmlLines;
  } catch (error) {
    console.error('[syntax] Failed to highlight code:', error);
    return lines.map(escapeHtml);
  }
}

/**
 * Escape HTML entities
 */
function escapeHtml(text: string): string {
  const map: Record<string, string> = {
    '&': '&amp;',
    '<': '&lt;',
    '>': '&gt;',
    '"': '&quot;',
    "'": '&#039;',
  };
  return text.replace(/[&<>"']/g, (m) => map[m]);
}
