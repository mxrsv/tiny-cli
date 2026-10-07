import { create } from "zustand";
import type { Progress } from "../types/ipc";
export type Page =
  "overview" | "cleanup" | "space-lens" | "monitor" | "safety" | "settings";
interface UiState {
  engineError: string | null;
  setEngineError: (message: string | null) => void;
  page: Page;
  selectedPaths: string[];
  progress: Progress | null;
  setPage: (page: Page) => void;
  selectPaths: (paths: string[]) => void;
  togglePath: (path: string) => void;
  setProgress: (progress: Progress | null) => void;
}
export const useUi = create<UiState>((set) => ({
  engineError: null,
  setEngineError: (engineError) => set({ engineError }),
  page: "overview",
  selectedPaths: [],
  progress: null,
  setPage: (page) => set({ page }),
  selectPaths: (selectedPaths) =>
    set({ selectedPaths: [...new Set(selectedPaths)] }),
  togglePath: (path) =>
    set((state) => ({
      selectedPaths: state.selectedPaths.includes(path)
        ? state.selectedPaths.filter((p) => p !== path)
        : [...state.selectedPaths, path],
    })),
  setProgress: (progress) => set({ progress }),
}));
