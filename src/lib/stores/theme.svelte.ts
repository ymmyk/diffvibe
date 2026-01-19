/**
 * Theme store for managing light/dark mode
 */

export type Theme = 'light' | 'dark';

const STORAGE_KEY = 'diffvibe-theme';

function getInitialTheme(): Theme {
  if (typeof window === 'undefined') return 'dark';

  const stored = localStorage.getItem(STORAGE_KEY);
  if (stored === 'light' || stored === 'dark') {
    return stored;
  }

  // Check system preference
  if (window.matchMedia('(prefers-color-scheme: light)').matches) {
    return 'light';
  }

  return 'dark';
}

function createThemeStore() {
  let theme = $state<Theme>('dark');
  let initialized = false;

  return {
    get value() {
      return theme;
    },

    init() {
      if (initialized) return;
      initialized = true;
      theme = getInitialTheme();
      this.apply();

      // Listen for system theme changes
      window.matchMedia('(prefers-color-scheme: light)').addEventListener('change', (e) => {
        if (!localStorage.getItem(STORAGE_KEY)) {
          theme = e.matches ? 'light' : 'dark';
          this.apply();
        }
      });
    },

    toggle() {
      theme = theme === 'dark' ? 'light' : 'dark';
      localStorage.setItem(STORAGE_KEY, theme);
      this.apply();
    },

    set(newTheme: Theme) {
      theme = newTheme;
      localStorage.setItem(STORAGE_KEY, theme);
      this.apply();
    },

    apply() {
      if (typeof document !== 'undefined') {
        document.documentElement.setAttribute('data-theme', theme);
      }
    },
  };
}

export const themeStore = createThemeStore();
