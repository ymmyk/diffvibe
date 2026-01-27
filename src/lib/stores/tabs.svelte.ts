/**
 * Tab management store
 */
import { recentStore } from './recent.svelte';
import type { AlignedScanResult } from '$lib/types';

export interface Tab {
  id: string;
  type: 'home' | 'compare' | 'merge' | 'directory';
  title: string;
  // For compare/merge tabs
  leftPath?: string;
  rightPath?: string;
  basePath?: string;
  mode?: 'file' | 'directory' | 'merge';
  // For merge mode - output file path
  outputPath?: string;
  // Dirty state
  dirty?: boolean;
  // Parent tab ID (for back navigation from file diff to directory)
  parentTabId?: string;
  // Directory comparison cached data
  scanResult?: AlignedScanResult | null;
  dirState?: DirectoryTabState;
}

export interface DirectoryTabState {
  expanded: Record<string, boolean>;
  scrollTop: number;
  leftSelected: string | null;
  rightSelected: string | null;
  filter: 'all' | 'changed' | 'identical';
  showIgnored: boolean;
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

    openCompare(leftPath: string, rightPath: string, mode: 'file' | 'directory' | 'merge' = 'file', basePath?: string, parentTabId?: string) {
      const leftName = getFileName(leftPath);
      const rightName = getFileName(rightPath);
      const title = `${leftName} ↔ ${rightName}`;

      const tabType = mode === 'directory' ? 'directory' : 'compare';

      // Check if tab already exists for this comparison
      const existing = tabs.find(
        (t) => t.type === tabType && t.leftPath === leftPath && t.rightPath === rightPath
      );

      if (existing) {
        activeTabId = existing.id;
        return existing.id;
      }

      const id = generateId();
      const newTab: Tab = {
        id,
        type: tabType,
        title,
        leftPath,
        rightPath,
        basePath,
        mode,
        parentTabId,
      };

      tabs = [...tabs, newTab];
      activeTabId = id;

      // Add to recent (only for standalone comparisons, not those from directory view)
      if (!parentTabId) {
        recentStore.add({ left: leftPath, right: rightPath, mode, base: basePath });
      }

      return id;
    },

    openMerge(basePath: string, localPath: string, remotePath: string, outputPath?: string) {
      const baseName = getFileName(basePath);
      const title = `Merge: ${baseName}`;

      // Check if tab already exists for this merge
      const existing = tabs.find(
        (t) => t.type === 'merge' && t.basePath === basePath && t.leftPath === localPath && t.rightPath === remotePath
      );

      if (existing) {
        activeTabId = existing.id;
        return existing.id;
      }

      const id = generateId();
      const newTab: Tab = {
        id,
        type: 'merge',
        title,
        leftPath: localPath,
        rightPath: remotePath,
        basePath,
        mode: 'merge',
        outputPath,
      };

      tabs = [...tabs, newTab];
      activeTabId = id;

      // Add to recent
      recentStore.add({ left: localPath, right: remotePath, mode: 'merge', base: basePath });

      return id;
    },

    setActive(id: string) {
      if (tabs.some((t) => t.id === id)) {
        activeTabId = id;
      }
    },

    setDirty(id: string, dirty: boolean) {
      const idx = tabs.findIndex((t) => t.id === id);
      if (idx !== -1) {
        tabs[idx] = { ...tabs[idx], dirty };
      }
    },

    getTab(id: string) {
      return tabs.find((t) => t.id === id);
    },

    close(id: string, force = false) {
      // Don't close the last tab or home tab
      if (tabs.length === 1) return false;
      if (id === 'home' && tabs.length > 1) {
        // Allow closing home if other tabs exist
      }

      const idx = tabs.findIndex((t) => t.id === id);
      if (idx === -1) return false;

      const tab = tabs[idx];

      // Check if tab has unsaved changes
      if (!force && tab.dirty) {
        return false; // Caller should show confirmation
      }

      tabs = tabs.filter((t) => t.id !== id);

      // If we closed the active tab, switch to adjacent tab
      if (activeTabId === id) {
        const newIdx = Math.min(idx, tabs.length - 1);
        activeTabId = tabs[newIdx].id;
      }

      return true;
    },

    forceClose(id: string) {
      this.close(id, true);
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

    // Directory tab state management
    setScanResult(id: string, result: AlignedScanResult | null) {
      const idx = tabs.findIndex((t) => t.id === id);
      if (idx !== -1) {
        tabs[idx] = { ...tabs[idx], scanResult: result };
      }
    },

    setDirectoryState(id: string, state: Partial<DirectoryTabState>) {
      const idx = tabs.findIndex((t) => t.id === id);
      if (idx !== -1) {
        const currentState = tabs[idx].dirState || {
          expanded: {},
          scrollTop: 0,
          leftSelected: null,
          rightSelected: null,
          filter: 'all',
          showIgnored: false,
        };
        tabs[idx] = { 
          ...tabs[idx], 
          dirState: { ...currentState, ...state } 
        };
      }
    },

    getDirectoryState(id: string): DirectoryTabState {
      const tab = tabs.find((t) => t.id === id);
      return tab?.dirState || {
        expanded: {},
        scrollTop: 0,
        leftSelected: null,
        rightSelected: null,
        filter: 'all',
        showIgnored: false,
      };
    },
  };
}

export const tabStore = createTabStore();
