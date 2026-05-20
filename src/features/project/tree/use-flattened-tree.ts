import { useMemo } from "react";
import type { AssetNode, AssetTree } from "../../../lib/types";

export type FlatRow = {
  node: AssetNode;
  nodeId: string;
  depth: number;
  hasChildren: boolean;
};

/**
 * Flatten AssetTree into a visible-rows array respecting expandedIds and filter.
 * Memoized — only recomputes on tree/expandedIds/filter change.
 */
export function useFlattenedTree(
  tree: AssetTree | null | undefined,
  expandedIds: Set<string>,
  filter: string,
): FlatRow[] {
  return useMemo(() => {
    if (!tree) return [];

    const filterLow = filter.toLowerCase();
    const out: FlatRow[] = [];

    /** Returns true if this node or any descendant matches filter */
    const matchesOrHasMatch = (id: string): boolean => {
      const node = tree.nodes[id];
      if (!node) return false;
      if (!filterLow) return true;
      if (node.name.toLowerCase().includes(filterLow)) return true;
      const children = getChildren(id);
      return children.some((cid) => matchesOrHasMatch(cid));
    };

    const getChildren = (id: string): string[] => {
      // children are all nodes whose parent is this id
      return Object.keys(tree.nodes).filter(
        (nid) => tree.nodes[nid]?.parent?.["0"] === id,
      );
    };

    const walk = (id: string, depth: number) => {
      const node = tree.nodes[id];
      if (!node) return;

      const children = getChildren(id);
      const hasChildren = children.length > 0;

      // Filter: skip if neither self nor any descendant matches
      if (filterLow && !matchesOrHasMatch(id)) return;

      out.push({ node, nodeId: id, depth, hasChildren });

      if (expandedIds.has(id) || (filterLow && hasChildren)) {
        for (const cid of children) {
          walk(cid, depth + 1);
        }
      }
    };

    walk(tree.root["0"], 0);
    return out;
  }, [tree, expandedIds, filter]);
}

/** Count all descendants (recursive) of a node */
export function countDescendants(
  tree: AssetTree,
  nodeId: string,
  cache: Map<string, number> = new Map(),
): number {
  if (cache.has(nodeId)) return cache.get(nodeId)!;
  const children = Object.keys(tree.nodes).filter(
    (nid) => tree.nodes[nid]?.parent?.["0"] === nodeId,
  );
  if (children.length === 0) {
    cache.set(nodeId, 0);
    return 0;
  }
  const count =
    children.length +
    children.reduce((sum, cid) => sum + countDescendants(tree, cid, cache), 0);
  cache.set(nodeId, count);
  return count;
}
