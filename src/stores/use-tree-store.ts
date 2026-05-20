import { create } from "zustand";
import type { AssetNode } from "../lib/types";

type ProjectId = string;
type NodeId = string;

type TreeState = {
  /** Set of expanded node IDs per project */
  expandedIds: Record<ProjectId, Set<NodeId>>;
  /** Selected node ID per project */
  selectedId: Record<ProjectId, NodeId | null>;
  /** Selected node data cache per project — set alongside selectedId */
  selectedNode: Record<ProjectId, AssetNode | null>;

  toggleExpanded: (projectId: ProjectId, nodeId: NodeId) => void;
  setExpanded: (projectId: ProjectId, nodeId: NodeId, open: boolean) => void;
  setSelected: (projectId: ProjectId, nodeId: NodeId, node: AssetNode) => void;
  clearProject: (projectId: ProjectId) => void;
};

export const useTreeStore = create<TreeState>()((set, get) => ({
  expandedIds: {},
  selectedId: {},
  selectedNode: {},

  toggleExpanded: (projectId, nodeId) => {
    const cur = get().expandedIds[projectId] ?? new Set<NodeId>();
    const next = new Set(cur);
    if (next.has(nodeId)) {
      next.delete(nodeId);
    } else {
      next.add(nodeId);
    }
    set((s) => ({ expandedIds: { ...s.expandedIds, [projectId]: next } }));
  },

  setExpanded: (projectId, nodeId, open) => {
    const cur = get().expandedIds[projectId] ?? new Set<NodeId>();
    const next = new Set(cur);
    if (open) {
      next.add(nodeId);
    } else {
      next.delete(nodeId);
    }
    set((s) => ({ expandedIds: { ...s.expandedIds, [projectId]: next } }));
  },

  setSelected: (projectId, nodeId, node) => {
    set((s) => ({
      selectedId: { ...s.selectedId, [projectId]: nodeId },
      selectedNode: { ...s.selectedNode, [projectId]: node },
    }));
  },

  clearProject: (projectId) => {
    const { expandedIds, selectedId, selectedNode } = get();
    const { [projectId]: _e, ...restE } = expandedIds;
    const { [projectId]: _s, ...restS } = selectedId;
    const { [projectId]: _n, ...restN } = selectedNode;
    set({ expandedIds: restE, selectedId: restS, selectedNode: restN });
  },
}));
