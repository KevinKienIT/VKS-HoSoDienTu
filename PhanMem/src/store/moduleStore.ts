/**
 * VKS ECMS — Module Store (Zustand)
 *
 * State management cho module configs.
 * Load tu Tauri backend, luu xuong DB khi thay doi.
 */

import { create } from "zustand";
import { invoke } from "@tauri-apps/api/core";
import type { ModuleConfig } from "../modules/loidung/registry";
import { getDefaultConfigs } from "../modules/loidung/registry";

// Will be replaced with real Tauri invoke in Phase 1B
async function invokeGetModuleConfigs(): Promise<ModuleConfig[]> {
  try {
    return await invoke<ModuleConfig[]>("get_module_configs");
  } catch {
    console.warn("Tauri not available, using defaults");
    return getDefaultConfigs();
  }
}

async function invokeSetModuleConfig(config: {
  module_id: string;
  enabled: boolean;
  selected_version: string;
  settings: string;
}): Promise<void> {
  try {
    await invoke("set_module_config", {
      moduleId: config.module_id,
      enabled: config.enabled,
      selectedVersion: config.selected_version,
      settings: config.settings,
    });
  } catch {
    console.warn("Tauri not available, config not persisted");
  }
}

interface ModuleState {
  configs: ModuleConfig[];
  loading: boolean;
  error: string | null;

  loadConfigs: () => Promise<void>;
  isModuleEnabled: (moduleId: string) => boolean;
  getModuleVersion: (moduleId: string) => string;
  getModuleSettings: (moduleId: string) => Record<string, unknown>;
  toggleModule: (moduleId: string, enabled: boolean) => Promise<void>;
  setModuleVersion: (moduleId: string, version: string) => Promise<void>;
  setModuleSettings: (
    moduleId: string,
    settings: Record<string, unknown>
  ) => Promise<void>;
}

export const useModuleStore = create<ModuleState>((set, get) => ({
  configs: getDefaultConfigs(),
  loading: false,
  error: null,

  loadConfigs: async () => {
    set({ loading: true, error: null });
    try {
      const configs = await invokeGetModuleConfigs();
      if (configs.length > 0) {
        set({ configs, loading: false });
      } else {
        set({ configs: getDefaultConfigs(), loading: false });
      }
    } catch (e) {
      set({ error: (e as Error).message, loading: false });
    }
  },

  isModuleEnabled: (moduleId: string) => {
    const config = get().configs.find((c) => c.module_id === moduleId);
    return config?.enabled ?? false;
  },

  getModuleVersion: (moduleId: string) => {
    const config = get().configs.find((c) => c.module_id === moduleId);
    return config?.selected_version ?? "v1";
  },

  getModuleSettings: (moduleId: string) => {
    const config = get().configs.find((c) => c.module_id === moduleId);
    try {
      return JSON.parse(config?.settings ?? "{}");
    } catch {
      return {};
    }
  },

  toggleModule: async (moduleId: string, enabled: boolean) => {
    const configs = get().configs.map((c) =>
      c.module_id === moduleId ? { ...c, enabled } : c
    );
    set({ configs });

    const config = configs.find((c) => c.module_id === moduleId);
    if (config) {
      await invokeSetModuleConfig(config);
    }
  },

  setModuleVersion: async (moduleId: string, version: string) => {
    const configs = get().configs.map((c) =>
      c.module_id === moduleId ? { ...c, selected_version: version } : c
    );
    set({ configs });

    const config = configs.find((c) => c.module_id === moduleId);
    if (config) {
      await invokeSetModuleConfig(config);
    }
  },

  setModuleSettings: async (
    moduleId: string,
    settings: Record<string, unknown>
  ) => {
    const configs = get().configs.map((c) =>
      c.module_id === moduleId
        ? { ...c, settings: JSON.stringify(settings) }
        : c
    );
    set({ configs });

    const config = configs.find((c) => c.module_id === moduleId);
    if (config) {
      await invokeSetModuleConfig(config);
    }
  },
}));
