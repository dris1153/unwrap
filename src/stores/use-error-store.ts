import { create } from "zustand";

/**
 * Globally-visible error info surfaced via `<ErrorToast />` in __root.
 *
 * Only one error is shown at a time — newer errors evict the previous one.
 * `id` increments on every `show()` call so the toast component can reset
 * its auto-dismiss timer when the error changes even if other fields match.
 */
export interface AppErrorView {
  id: number;
  title: string;
  message: string;
  /** Optional hint shown beneath the message — e.g. log directory path. */
  logHint?: string;
}

interface ErrorState {
  current: AppErrorView | null;
  show: (err: Omit<AppErrorView, "id">) => void;
  dismiss: () => void;
}

let counter = 0;

export const useErrorStore = create<ErrorState>((set) => ({
  current: null,
  show: (err) => set({ current: { ...err, id: ++counter } }),
  dismiss: () => set({ current: null }),
}));
