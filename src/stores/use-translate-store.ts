import { create } from "zustand";

export type TranslateFilter = "all" | "translated" | "pending" | "review";

type TranslateState = {
  sourceLocale: string;
  targetLocale: string;
  filter: TranslateFilter;
  /** Optimistic drafts keyed by row id — override DB value until flush. */
  dirty: Record<string, string>;

  setSourceLocale: (l: string) => void;
  setTargetLocale: (l: string) => void;
  setFilter: (f: TranslateFilter) => void;
  updateDraft: (rowId: string, text: string) => void;
  clearDraft: (rowId: string) => void;
};

export const useTranslateStore = create<TranslateState>()((set) => ({
  sourceLocale: "en",
  targetLocale: "vi",
  filter: "all",
  dirty: {},

  setSourceLocale: (l) => set({ sourceLocale: l }),
  setTargetLocale: (l) => set({ targetLocale: l }),
  setFilter: (f) => set({ filter: f }),

  updateDraft: (rowId, text) =>
    set((s) => ({ dirty: { ...s.dirty, [rowId]: text } })),

  clearDraft: (rowId) =>
    set((s) => {
      const next = { ...s.dirty };
      delete next[rowId];
      return { dirty: next };
    }),
}));
