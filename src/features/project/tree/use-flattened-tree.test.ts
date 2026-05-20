// use-flattened-tree.test.ts — unit tests for the flatten + filter logic.
//
// The hook is memoized via useMemo, but the underlying algorithm is deterministic.
// We test via renderHook to exercise the actual hook in a jsdom environment.

import { renderHook } from "@testing-library/react";
import { describe, expect, test } from "vitest";
import type { AssetTree } from "../../../lib/types";
import { useFlattenedTree } from "./use-flattened-tree";

// ---------------------------------------------------------------------------
// Fixture helpers
// ---------------------------------------------------------------------------

/**
 * Build a minimal AssetTree.
 *
 * Structure:
 *   root (folder)
 *   ├── child-a (texture)
 *   │   ├── grandchild-a1 (texture)
 *   │   └── grandchild-a2 (audio)
 *   └── child-b (script)
 */
function makeTree(): AssetTree {
  return {
    root: { "0": "root" },
    nodes: {
      root: {
        id: { "0": "root" },
        parent: null,
        name: "MyGame_Data",
        kind: "folder",
        size: 0,
        source_path: "/",
        metadata: {},
      },
      "child-a": {
        id: { "0": "child-a" },
        parent: { "0": "root" },
        name: "Textures",
        kind: "folder",
        size: 0,
        source_path: "/Textures",
        metadata: {},
      },
      "grandchild-a1": {
        id: { "0": "grandchild-a1" },
        parent: { "0": "child-a" },
        name: "hero.png",
        kind: "texture",
        size: 1024,
        source_path: "/Textures/hero.png",
        metadata: {},
      },
      "grandchild-a2": {
        id: { "0": "grandchild-a2" },
        parent: { "0": "child-a" },
        name: "theme.ogg",
        kind: "audio",
        size: 2048,
        source_path: "/Textures/theme.ogg",
        metadata: {},
      },
      "child-b": {
        id: { "0": "child-b" },
        parent: { "0": "root" },
        name: "Player.cs",
        kind: "script",
        size: 512,
        source_path: "/Player.cs",
        metadata: {},
      },
    },
  };
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

describe("useFlattenedTree", () => {
  test("null tree returns empty array", () => {
    const { result } = renderHook(() =>
      useFlattenedTree(null, new Set(), ""),
    );
    expect(result.current).toHaveLength(0);
  });

  test("all collapsed — only root visible", () => {
    const tree = makeTree();
    const { result } = renderHook(() =>
      useFlattenedTree(tree, new Set(), ""),
    );
    expect(result.current).toHaveLength(1);
    expect(result.current[0].nodeId).toBe("root");
  });

  test("expand root — root + direct children visible", () => {
    const tree = makeTree();
    const expanded = new Set(["root"]);
    const { result } = renderHook(() =>
      useFlattenedTree(tree, expanded, ""),
    );
    const ids = result.current.map((r) => r.nodeId);
    expect(ids).toContain("root");
    expect(ids).toContain("child-a");
    expect(ids).toContain("child-b");
    // Grandchildren not yet visible
    expect(ids).not.toContain("grandchild-a1");
  });

  test("expand root + child-a — grandchildren appear", () => {
    const tree = makeTree();
    const expanded = new Set(["root", "child-a"]);
    const { result } = renderHook(() =>
      useFlattenedTree(tree, expanded, ""),
    );
    const ids = result.current.map((r) => r.nodeId);
    expect(ids).toContain("grandchild-a1");
    expect(ids).toContain("grandchild-a2");
  });

  test("filter narrows visible rows — matching nodes shown with ancestors", () => {
    const tree = makeTree();
    // "hero" matches grandchild-a1 only
    const { result } = renderHook(() =>
      useFlattenedTree(tree, new Set(), "hero"),
    );
    const ids = result.current.map((r) => r.nodeId);
    // Root and child-a appear because they are ancestors of the match
    expect(ids).toContain("root");
    expect(ids).toContain("child-a");
    expect(ids).toContain("grandchild-a1");
    // Unrelated nodes hidden
    expect(ids).not.toContain("child-b");
    expect(ids).not.toContain("grandchild-a2");
  });

  test("filter is case-insensitive", () => {
    const tree = makeTree();
    const { result } = renderHook(() =>
      useFlattenedTree(tree, new Set(), "HERO"),
    );
    const ids = result.current.map((r) => r.nodeId);
    expect(ids).toContain("grandchild-a1");
  });

  test("filter that matches nothing returns empty array", () => {
    const tree = makeTree();
    const { result } = renderHook(() =>
      useFlattenedTree(tree, new Set(), "zzznomatch"),
    );
    expect(result.current).toHaveLength(0);
  });

  test("depth values are correct for nested expand", () => {
    const tree = makeTree();
    const expanded = new Set(["root", "child-a"]);
    const { result } = renderHook(() =>
      useFlattenedTree(tree, expanded, ""),
    );
    const byId = Object.fromEntries(result.current.map((r) => [r.nodeId, r]));
    expect(byId["root"].depth).toBe(0);
    expect(byId["child-a"].depth).toBe(1);
    expect(byId["grandchild-a1"].depth).toBe(2);
  });

  test("hasChildren flag reflects presence of children", () => {
    const tree = makeTree();
    const expanded = new Set(["root"]);
    const { result } = renderHook(() =>
      useFlattenedTree(tree, expanded, ""),
    );
    const byId = Object.fromEntries(result.current.map((r) => [r.nodeId, r]));
    expect(byId["root"].hasChildren).toBe(true);
    expect(byId["child-a"].hasChildren).toBe(true);
    // child-b is a leaf
    expect(byId["child-b"].hasChildren).toBe(false);
  });

  test("deterministic order on multiple renders with same inputs", () => {
    const tree = makeTree();
    const expanded = new Set(["root", "child-a"]);
    const { result, rerender } = renderHook(() =>
      useFlattenedTree(tree, expanded, ""),
    );
    const first = result.current.map((r) => r.nodeId);
    rerender();
    const second = result.current.map((r) => r.nodeId);
    expect(first).toEqual(second);
  });
});
