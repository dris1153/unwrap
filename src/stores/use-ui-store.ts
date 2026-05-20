import { create } from "zustand";
import { persist } from "zustand/middleware";

export type RailTab =
  | "files"
  | "search"
  | "xref"
  | "scripts"
  | "strings"
  | "bookmarks";

type UiState = {
  sidebarWidth: number;
  inspectorWidth: number;
  activeRail: RailTab;
  setSidebarWidth: (w: number) => void;
  setInspectorWidth: (w: number) => void;
  toggleInspector: () => void;
  setActiveRail: (r: RailTab) => void;
};

const SIDEBAR_DEFAULT = 280;
const INSPECTOR_DEFAULT = 320;
const INSPECTOR_COLLAPSED = 0;

export const useUiStore = create<UiState>()(
  persist(
    (set, get) => ({
      sidebarWidth: SIDEBAR_DEFAULT,
      inspectorWidth: INSPECTOR_DEFAULT,
      activeRail: "files",

      setSidebarWidth: (w) =>
        set({ sidebarWidth: Math.min(400, Math.max(200, w)) }),

      setInspectorWidth: (w) =>
        set({ inspectorWidth: w === 0 ? 0 : Math.min(480, Math.max(240, w)) }),

      toggleInspector: () => {
        const current = get().inspectorWidth;
        set({
          inspectorWidth:
            current === INSPECTOR_COLLAPSED ? INSPECTOR_DEFAULT : INSPECTOR_COLLAPSED,
        });
      },

      setActiveRail: (r) => set({ activeRail: r }),
    }),
    {
      name: "unwrap-ui",
      // Only persist layout prefs, not callbacks
      partialize: (s) => ({
        sidebarWidth: s.sidebarWidth,
        inspectorWidth: s.inspectorWidth,
        activeRail: s.activeRail,
      }),
    },
  ),
);
