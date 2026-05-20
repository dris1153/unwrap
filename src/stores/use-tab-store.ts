import { create } from "zustand";
import type { AssetKind, AssetNode } from "../lib/types";

export type Tab = {
  id: string;
  projectId: string;
  nodeId: string;
  kind: AssetKind;
  /** Display label shown in the tab — relative path from root */
  pathLabel: string;
  name: string;
};

type TabState = {
  tabs: Tab[];
  activeTabId: string | null;

  openTab: (projectId: string, node: AssetNode, pathLabel?: string) => void;
  closeTab: (tabId: string) => void;
  setActive: (tabId: string) => void;
  closeAllForProject: (projectId: string) => void;
};

export const useTabStore = create<TabState>()((set, get) => ({
  tabs: [],
  activeTabId: null,

  openTab: (projectId, node, pathLabel) => {
    const { tabs } = get();
    const nodeId = node.id["0"];
    // Reuse existing tab if already open for this project+node
    const existing = tabs.find(
      (t) => t.projectId === projectId && t.nodeId === nodeId,
    );
    if (existing) {
      set({ activeTabId: existing.id });
      return;
    }
    const tab: Tab = {
      id: `${projectId}:${nodeId}`,
      projectId,
      nodeId,
      kind: node.kind,
      pathLabel: pathLabel ?? node.name,
      name: node.name,
    };
    set((s) => ({
      tabs: [...s.tabs, tab],
      activeTabId: tab.id,
    }));
  },

  closeTab: (tabId) => {
    const { tabs, activeTabId } = get();
    const idx = tabs.findIndex((t) => t.id === tabId);
    const next = tabs.filter((t) => t.id !== tabId);
    let nextActive = activeTabId;
    if (activeTabId === tabId) {
      // Activate adjacent tab: prefer right, fallback left
      if (next.length === 0) {
        nextActive = null;
      } else if (idx < next.length) {
        nextActive = next[idx].id;
      } else {
        nextActive = next[next.length - 1].id;
      }
    }
    set({ tabs: next, activeTabId: nextActive });
  },

  setActive: (tabId) => set({ activeTabId: tabId }),

  closeAllForProject: (projectId) => {
    const { tabs, activeTabId } = get();
    const next = tabs.filter((t) => t.projectId !== projectId);
    const activeStillExists = next.some((t) => t.id === activeTabId);
    set({
      tabs: next,
      activeTabId: activeStillExists ? activeTabId : (next[0]?.id ?? null),
    });
  },
}));
