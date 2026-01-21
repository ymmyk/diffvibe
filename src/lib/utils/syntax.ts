import { createHighlighter, type Highlighter, type BundledLanguage, type BundledTheme } from 'shiki';

let highlighterInstance: Highlighter | null = null;
let initPromise: Promise<Highlighter> | null = null;

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

const DEFAULT_THEME: BundledTheme = 'github-dark';

/**
 * Initialize the syntax highlighter (lazy, called once)
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
    themes: [DEFAULT_THEME],
    langs: SUPPORTED_LANGUAGES,
  }).then(h => {
    highlighterInstance = h;
    initPromise = null;
    return h;
  });
  
  return initPromise;
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
    const highlighter = await getHighlighter();
    const lang = detectLanguage(filePath);
    
    if (lang === 'plaintext') {
      // No highlighting for plain text
      return lines.map(escapeHtml);
    }
    
    // Highlight the full content to maintain context across lines
    const fullContent = lines.join('\n');
    const tokens = highlighter.codeToTokens(fullContent, {
      lang,
      theme: DEFAULT_THEME,
    });
    
    // Convert tokens to HTML per line
    const htmlLines: string[] = [];
    for (const line of tokens.tokens) {
      const html = line.map(token => {
        const style = token.htmlStyle ? ` style="${token.htmlStyle}"` : '';
        return `<span${style}>${escapeHtml(token.content)}</span>`;
      }).join('');
      htmlLines.push(html);
    }
    
    return htmlLines;
  } catch (error) {
    console.error('Failed to highlight code:', error);
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
