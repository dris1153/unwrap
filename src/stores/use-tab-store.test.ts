// use-tab-store.test.ts — unit tests for useTabStore Zustand actions.

import { beforeEach, describe, expect, test } from "vitest";
import type { AssetNode } from "../lib/types";
import { useTabStore } from "./use-tab-store";

beforeEach(() => {
  useTabStore.setState({ tabs: [], activeTabId: null });
});

function makeNode(id: string, name = `Node ${id}`): AssetNode {
  return {
    id: { "0": id },
    parent: null,
    name,
    kind: "texture",
    size: 0,
    source_path: `/${id}`,
    metadata: {},
  };
}

describe("useTabStore — openTab", () => {
  test("openTab adds a new tab and sets it active", () => {
    useTabStore.getState().openTab("proj1", makeNode("n1"));
    const { tabs, activeTabId } = useTabStore.getState();
    expect(tabs).toHaveLength(1);
    expect(tabs[0].nodeId).toBe("n1");
    expect(activeTabId).toBe("proj1:n1");
  });

  test("openTab for same nodeId does not duplicate — becomes active", () => {
    const node = makeNode("n1");
    useTabStore.getState().openTab("proj1", node);
    useTabStore.getState().openTab("proj1", node);
    expect(useTabStore.getState().tabs).toHaveLength(1);
    expect(useTabStore.getState().activeTabId).toBe("proj1:n1");
  });

  test("openTab sets pathLabel from argument when provided", () => {
    useTabStore.getState().openTab("proj1", makeNode("n1"), "Folder/n1");
    expect(useTabStore.getState().tabs[0].pathLabel).toBe("Folder/n1");
  });

  test("openTab defaults pathLabel to node.name when not provided", () => {
    useTabStore.getState().openTab("proj1", makeNode("n1", "hero.png"));
    expect(useTabStore.getState().tabs[0].pathLabel).toBe("hero.png");
  });

  test("opening a second different tab appends and activates it", () => {
    useTabStore.getState().openTab("proj1", makeNode("n1"));
    useTabStore.getState().openTab("proj1", makeNode("n2"));
    expect(useTabStore.getState().tabs).toHaveLength(2);
    expect(useTabStore.getState().activeTabId).toBe("proj1:n2");
  });
});

describe("useTabStore — closeTab", () => {
  test("closeTab removes the tab", () => {
    useTabStore.getState().openTab("proj1", makeNode("n1"));
    useTabStore.getState().closeTab("proj1:n1");
    expect(useTabStore.getState().tabs).toHaveLength(0);
    expect(useTabStore.getState().activeTabId).toBeNull();
  });

  test("closeTab active tab reassigns active to right neighbor", () => {
    // tabs: n1 (active), n2, n3
    useTabStore.getState().openTab("proj1", makeNode("n1"));
    useTabStore.getState().openTab("proj1", makeNode("n2"));
    useTabStore.getState().openTab("proj1", makeNode("n3"));
    useTabStore.getState().setActive("proj1:n1");

    useTabStore.getState().closeTab("proj1:n1");
    // n2 is now at index 0 — it was the right neighbor
    expect(useTabStore.getState().activeTabId).toBe("proj1:n2");
  });

  test("closeTab last tab sets activeTabId to null", () => {
    useTabStore.getState().openTab("proj1", makeNode("n1"));
    useTabStore.getState().closeTab("proj1:n1");
    expect(useTabStore.getState().activeTabId).toBeNull();
  });

  test("closeTab non-active tab does not change activeTabId", () => {
    useTabStore.getState().openTab("proj1", makeNode("n1"));
    useTabStore.getState().openTab("proj1", makeNode("n2"));
    // active is n2; close n1
    useTabStore.getState().closeTab("proj1:n1");
    expect(useTabStore.getState().activeTabId).toBe("proj1:n2");
  });

  test("closeTab rightmost active tab falls back to left neighbor", () => {
    useTabStore.getState().openTab("proj1", makeNode("n1"));
    useTabStore.getState().openTab("proj1", makeNode("n2"));
    // n2 is active (last opened); close it
    useTabStore.getState().closeTab("proj1:n2");
    expect(useTabStore.getState().activeTabId).toBe("proj1:n1");
  });
});

describe("useTabStore — closeAllForProject", () => {
  test("removes all tabs belonging to a project", () => {
    useTabStore.getState().openTab("proj1", makeNode("n1"));
    useTabStore.getState().openTab("proj1", makeNode("n2"));
    useTabStore.getState().openTab("proj2", makeNode("n3"));
    useTabStore.getState().closeAllForProject("proj1");

    const { tabs } = useTabStore.getState();
    expect(tabs.every((t) => t.projectId !== "proj1")).toBe(true);
    expect(tabs).toHaveLength(1);
  });
});
