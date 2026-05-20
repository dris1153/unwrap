// features/command-palette/use-palette-store.ts
// Zustand store for command palette open state, query, and recent searches.

import { create } from "zustand";
import { persist } from "zustand/middleware";

const MAX_RECENTS = 10;

type PaletteState = {
  open: boolean;
  query: string;
  recents: string[];

  toggle: () => void;
  openPalette: (initialQuery?: string) => void;
  close: () => void;
  setQuery: (q: string) => void;
  pushRecent: (label: string) => void;
  clearRecents: () => void;
};

export const usePaletteStore = create<PaletteState>()(
  persist(
    (set, get) => ({
      open: false,
      query: "",
      recents: [],

      toggle: () => {
        const { open } = get();
        if (open) {
          set({ open: false, query: "" });
        } else {
          set({ open: true, query: "" });
        }
      },

      openPalette: (initialQuery = "") => {
        set({ open: true, query: initialQuery });
      },

      close: () => set({ open: false, query: "" }),

      setQuery: (q) => set({ query: q }),

      pushRecent: (label) => {
        const { recents } = get();
        // Deduplicate — move to front if already exists
        const filtered = recents.filter((r) => r !== label);
        set({ recents: [label, ...filtered].slice(0, MAX_RECENTS) });
      },

      clearRecents: () => set({ recents: [] }),
    }),
    {
      name: "unwrap-palette",
      // Only persist recents — open/query are transient
      partialize: (s) => ({ recents: s.recents }),
    },
  ),
);
