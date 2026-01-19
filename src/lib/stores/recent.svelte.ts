import { load, Store } from '@tauri-apps/plugin-store';

export interface RecentComparison {
  left: string;
  right: string;
  mode: 'file' | 'directory' | 'merge';
  base?: string;
  timestamp: number;
}

const STORE_KEY = 'recent_comparisons';
const MAX_RECENT = 10;

function createRecentStore() {
  let items = $state<RecentComparison[]>([]);
  let store: Store | null = null;
  let initialized = false;

  async function init() {
    if (initialized) return;
    try {
      store = await load('recent.json', { autoSave: true });
      const saved = await store.get<RecentComparison[]>(STORE_KEY);
      if (saved) {
        items = saved;
      }
      initialized = true;
    } catch (e) {
      console.error('Failed to load recent store:', e);
    }
  }

  async function add(comparison: Omit<RecentComparison, 'timestamp'>) {
    // Remove existing entry with same paths
    items = items.filter(
      (item) => !(item.left === comparison.left && item.right === comparison.right)
    );

    // Add new entry at start
    items = [
      { ...comparison, timestamp: Date.now() },
      ...items.slice(0, MAX_RECENT - 1)
    ];

    await save();
  }

  async function remove(index: number) {
    items = items.filter((_, i) => i !== index);
    await save();
  }

  async function clear() {
    items = [];
    await save();
  }

  async function save() {
    if (store) {
      await store.set(STORE_KEY, items);
    }
  }

  return {
    get items() {
      return items;
    },
    init,
    add,
    remove,
    clear
  };
}

export const recentStore = createRecentStore();
