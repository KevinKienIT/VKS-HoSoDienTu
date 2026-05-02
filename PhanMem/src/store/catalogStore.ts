/**
 * VKS ECMS — Catalog Store (Zustand)
 *
 * State management cho catalog entries.
 * Theo pattern tu spec: 20260424_09_dac_ta_json_schema_va_state_management.md §2.2
 */

import { create } from "zustand";
import type { CatalogEntry, CatalogStats } from "../lib/catalog";
import * as catalogService from "../services/catalogService";

interface CatalogState {
  // State
  entries: CatalogEntry[];
  stats: CatalogStats | null;
  loading: boolean;
  scanning: boolean;
  error: string | null;

  // Actions
  fetchEntries: () => Promise<void>;
  fetchStats: () => Promise<void>;
  scanFolder: (folderPath: string) => Promise<number>;
  clearError: () => void;
}

export const useCatalogStore = create<CatalogState>((set) => ({
  entries: [],
  stats: null,
  loading: false,
  scanning: false,
  error: null,

  fetchEntries: async () => {
    set({ loading: true, error: null });
    try {
      const entries = await catalogService.getCatalogEntries();
      set({ entries, loading: false });
    } catch (e) {
      set({ error: (e as Error).message, loading: false });
    }
  },

  fetchStats: async () => {
    try {
      const stats = await catalogService.getCatalogStats();
      set({ stats });
    } catch (e) {
      set({ error: (e as Error).message });
    }
  },

  scanFolder: async (folderPath: string) => {
    set({ scanning: true, error: null });
    try {
      const count = await catalogService.scanFolder(folderPath);
      // Re-fetch entries after scan
      const entries = await catalogService.getCatalogEntries();
      const stats = await catalogService.getCatalogStats();
      set({ entries, stats, scanning: false });
      return count;
    } catch (e) {
      set({ error: (e as Error).message, scanning: false });
      return 0;
    }
  },

  clearError: () => set({ error: null }),
}));
