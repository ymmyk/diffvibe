/**
 * Syntax theme store for managing code highlighting themes
 */

import type { BundledTheme } from 'shiki';
import { setTheme } from '$lib/utils/syntax';

export type SyntaxTheme = BundledTheme;

const STORAGE_KEY = 'diffvibe-syntax-theme';
const DEFAULT_DARK_THEME: SyntaxTheme = 'github-dark';
const DEFAULT_LIGHT_THEME: SyntaxTheme = 'github-light';

function getInitialTheme(): SyntaxTheme {
  if (typeof window === 'undefined') return DEFAULT_DARK_THEME;

  const stored = localStorage.getItem(STORAGE_KEY);
  if (stored) {
    return stored as SyntaxTheme;
  }

  // Default based on system preference
  if (window.matchMedia('(prefers-color-scheme: light)').matches) {
    return DEFAULT_LIGHT_THEME;
  }

  return DEFAULT_DARK_THEME;
}

function createSyntaxThemeStore() {
  let theme = $state<SyntaxTheme>(DEFAULT_DARK_THEME);
  let initialized = false;

  return {
    get value() {
      return theme;
    },

    init() {
      if (initialized) return;
      initialized = true;
      theme = getInitialTheme();
      setTheme(theme);
    },

    set(newTheme: SyntaxTheme) {
      theme = newTheme;
      localStorage.setItem(STORAGE_KEY, theme);
      setTheme(theme);
    },
  };
}

export const syntaxThemeStore = createSyntaxThemeStore();
