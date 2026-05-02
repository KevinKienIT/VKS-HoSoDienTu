/**
 * VKS ECMS — useModuleConfig hook
 *
 * Hook tien loi de lay config cua 1 module cu the.
 * Derive truc tiep tu configs array de tranh re-render loop.
 */

import { useMemo } from "react";
import { useModuleStore } from "../store/moduleStore";

export interface ModuleConfigResult {
  enabled: boolean;
  version: string;
  settings: Record<string, unknown>;
  loading: boolean;
}

export function useModuleConfig(moduleId: string): ModuleConfigResult {
  const loading = useModuleStore((s) => s.loading);
  const configs = useModuleStore((s) => s.configs);

  return useMemo(() => {
    const config = configs.find((c) => c.module_id === moduleId);
    let settings: Record<string, unknown> = {};
    try {
      settings = JSON.parse(config?.settings ?? "{}");
    } catch {
      settings = {};
    }
    return {
      enabled: config?.enabled ?? false,
      version: config?.selected_version ?? "v1",
      settings,
      loading,
    };
  }, [configs, moduleId, loading]);
}
