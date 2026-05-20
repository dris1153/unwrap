// use-tree-store.test.ts — unit tests for useTreeStore Zustand actions.

import { beforeEach, describe, expect, test } from "vitest";
import type { AssetNode } from "../lib/types";
import { useTreeStore } from "./use-tree-store";

// Reset Zustand store state between tests
beforeEach(() => {
  useTreeStore.setState({
    expandedIds: {},
    selectedId: {},
    selectedNode: {},
  });
});

function makeNode(id: string): AssetNode {
  return {
    id: { "0": id },
    parent: null,
    name: `Node ${id}`,
    kind: "folder",
    size: 0,
    source_path: `/${id}`,
    metadata: {},
  };
}

describe("useTreeStore — toggleExpanded", () => {
  test("toggles a node into the expanded set", () => {
    useTreeStore.getState().toggleExpanded("proj1", "node-a");
    const set = useTreeStore.getState().expandedIds["proj1"];
    expect(set.has("node-a")).toBe(true);
  });

  test("toggles a node back out of the expanded set", () => {
    useTreeStore.getState().toggleExpanded("proj1", "node-a");
    useTreeStore.getState().toggleExpanded("proj1", "node-a");
    const set = useTreeStore.getState().expandedIds["proj1"];
    expect(set.has("node-a")).toBe(false);
  });

  test("toggling one project does not affect another", () => {
    useTreeStore.getState().toggleExpanded("proj1", "node-a");
    const setP2 = useTreeStore.getState().expandedIds["proj2"];
    expect(setP2).toBeUndefined();
  });
});

describe("useTreeStore — setExpanded", () => {
  test("setExpanded(true) adds to set", () => {
    useTreeStore.getState().setExpanded("proj1", "node-b", true);
    expect(useTreeStore.getState().expandedIds["proj1"].has("node-b")).toBe(true);
  });

  test("setExpanded(false) removes from set", () => {
    useTreeStore.getState().setExpanded("proj1", "node-b", true);
    useTreeStore.getState().setExpanded("proj1", "node-b", false);
    expect(useTreeStore.getState().expandedIds["proj1"].has("node-b")).toBe(false);
  });
});

describe("useTreeStore — setSelected", () => {
  test("setSelected stores nodeId and node data per project", () => {
    const node = makeNode("abc");
    useTreeStore.getState().setSelected("proj1", "abc", node);
    expect(useTreeStore.getState().selectedId["proj1"]).toBe("abc");
    expect(useTreeStore.getState().selectedNode["proj1"]).toEqual(node);
  });

  test("setSelected updates per project independently", () => {
    const nodeA = makeNode("aaa");
    const nodeB = makeNode("bbb");
    useTreeStore.getState().setSelected("proj1", "aaa", nodeA);
    useTreeStore.getState().setSelected("proj2", "bbb", nodeB);
    expect(useTreeStore.getState().selectedId["proj1"]).toBe("aaa");
    expect(useTreeStore.getState().selectedId["proj2"]).toBe("bbb");
  });
});

describe("useTreeStore — expanded state isolation between projects", () => {
  test("expanded ids do not bleed between projects", () => {
    useTreeStore.getState().toggleExpanded("proj1", "shared-node");
    useTreeStore.getState().toggleExpanded("proj2", "other-node");

    const set1 = useTreeStore.getState().expandedIds["proj1"];
    const set2 = useTreeStore.getState().expandedIds["proj2"];

    expect(set1.has("shared-node")).toBe(true);
    expect(set1.has("other-node")).toBe(false);
    expect(set2.has("other-node")).toBe(true);
    expect(set2.has("shared-node")).toBe(false);
  });
});

describe("useTreeStore — clearProject", () => {
  test("clearProject removes all state for that project", () => {
    const node = makeNode("x");
    useTreeStore.getState().toggleExpanded("proj1", "x");
    useTreeStore.getState().setSelected("proj1", "x", node);
    useTreeStore.getState().clearProject("proj1");

    expect(useTreeStore.getState().expandedIds["proj1"]).toBeUndefined();
    expect(useTreeStore.getState().selectedId["proj1"]).toBeUndefined();
    expect(useTreeStore.getState().selectedNode["proj1"]).toBeUndefined();
  });

  test("clearProject does not affect other projects", () => {
    const node = makeNode("y");
    useTreeStore.getState().setSelected("proj1", "x", makeNode("x"));
    useTreeStore.getState().setSelected("proj2", "y", node);
    useTreeStore.getState().clearProject("proj1");

    expect(useTreeStore.getState().selectedId["proj2"]).toBe("y");
  });
});
