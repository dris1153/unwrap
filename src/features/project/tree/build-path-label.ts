import type { AssetTree, AssetNode } from "../../../lib/types";

/**
 * Build a relative path label for a node by walking up the tree.
 * Returns e.g. "characters/Mira/sprites/idle_01.png"
 */
export function buildPathLabel(tree: AssetTree, nodeId: string): string {
  const parts: string[] = [];
  let current: string | null = nodeId;

  while (current !== null) {
    const node: AssetNode | undefined = tree.nodes[current];
    if (!node) break;
    parts.unshift(node.name);
    current = node.parent?.["0"] ?? null;
  }

  // Drop the root node name (Assets) from label to keep it compact
  if (parts.length > 1) parts.shift();

  return parts.join("/");
}
