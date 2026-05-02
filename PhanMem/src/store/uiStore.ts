import { create } from 'zustand';
import type { PipelineFooterStatus, PipelineProgressStatus } from '../services/scanService';
import { getPipelineFooterStatus, getPipelineProgressStatus } from '../services/scanService';

type UiState = {
  online: boolean;
  statusText: string;
  sidebarCollapsed: boolean;
  pipelineFooter: PipelineFooterStatus;
  pipelineProgress: PipelineProgressStatus | null;
  setOnline: (online: boolean) => void;
  setStatusText: (statusText: string) => void;
  toggleSidebar: () => void;
  refreshPipelineStatus: () => Promise<void>;
};

export const useUiStore = create<UiState>((set) => ({
  online: false,
  statusText: 'Checking Tauri runtime',
  sidebarCollapsed: false,
  pipelineFooter: {
    active_jobs: 0,
    paused_jobs: 0,
    failed_jobs: 0,
    completed_jobs: 0,
    cancelled_jobs: 0,
  },
  pipelineProgress: null,
  setOnline: (online) => set({ online }),
  setStatusText: (statusText) => set({ statusText }),
  toggleSidebar: () =>
    set((state) => ({
      sidebarCollapsed: !state.sidebarCollapsed,
    })),
  refreshPipelineStatus: async () => {
    const [footer, progress] = await Promise.all([
      getPipelineFooterStatus(),
      getPipelineProgressStatus(),
    ]);
    set({
      pipelineFooter: footer,
      pipelineProgress: progress,
    });
  },
}));
