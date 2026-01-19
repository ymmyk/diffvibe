/**
 * Tab management store
 */
import { recentStore } from './recent.svelte';

export interface Tab {
  id: string;
  type: 'home' | 'compare';
  title: string;
  // For compare tabs
  leftPath?: string;
  rightPath?: string;
  basePath?: string;
  mode?: 'file' | 'directory' | 'merge';
}

export interface HomeState {
  mode: 'file' | 'directory' | 'merge';
  leftPath: string;
  rightPath: string;
  basePath: string;
}

function createTabStore() {
  let tabs = $state<Tab[]>([{ id: 'home', type: 'home', title: 'Home' }]);
  let activeTabId = $state('home');
  let homeState = $state<HomeState>({ mode: 'file', leftPath: '', rightPath: '', basePath: '' });

  function generateId(): string {
    return `tab-${Date.now()}-${Math.random().toString(36).slice(2, 7)}`;
  }

  function getFileName(path: string): string {
    return path.split('/').pop() || path.split('\\').pop() || path;
  }

  return {
    get tabs() {
      return tabs;
    },
    get activeTabId() {
      return activeTabId;
    },
    get activeTab() {
      return tabs.find((t) => t.id === activeTabId) || tabs[0];
    },
    get homeState() {
      return homeState;
    },
    setHomeState(state: Partial<HomeState>) {
      homeState = { ...homeState, ...state };
    },

    openCompare(leftPath: string, rightPath: string, mode: 'file' | 'directory' | 'merge' = 'file', basePath?: string) {
      const leftName = getFileName(leftPath);
      const rightName = getFileName(rightPath);
      const title = `${leftName} ↔ ${rightName}`;

      // Check if tab already exists for this comparison
      const existing = tabs.find(
        (t) => t.type === 'compare' && t.leftPath === leftPath && t.rightPath === rightPath
      );

      if (existing) {
        activeTabId = existing.id;
        return existing.id;
      }

      const id = generateId();
      const newTab: Tab = {
        id,
        type: 'compare',
        title,
        leftPath,
        rightPath,
        basePath,
        mode,
      };

      tabs = [...tabs, newTab];
      activeTabId = id;

      // Add to recent
      recentStore.add({ left: leftPath, right: rightPath, mode, base: basePath });

      return id;
    },

    setActive(id: string) {
      if (tabs.some((t) => t.id === id)) {
        activeTabId = id;
      }
    },

    close(id: string) {
      // Don't close the last tab or home tab
      if (tabs.length === 1) return;
      if (id === 'home' && tabs.length > 1) {
        // Allow closing home if other tabs exist
      }

      const idx = tabs.findIndex((t) => t.id === id);
      if (idx === -1) return;

      tabs = tabs.filter((t) => t.id !== id);

      // If we closed the active tab, switch to adjacent tab
      if (activeTabId === id) {
        const newIdx = Math.min(idx, tabs.length - 1);
        activeTabId = tabs[newIdx].id;
      }
    },

    closeOthers(id: string) {
      const keep = tabs.find((t) => t.id === id);
      if (keep) {
        tabs = [keep];
        activeTabId = id;
      }
    },

    closeAll() {
      tabs = [{ id: 'home', type: 'home', title: 'Home' }];
      activeTabId = 'home';
    },
  };
}

export const tabStore = createTabStore();
