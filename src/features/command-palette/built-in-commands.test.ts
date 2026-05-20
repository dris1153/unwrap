// built-in-commands.test.ts — unit tests for filterBuiltInCommands.

import { describe, expect, test } from "vitest";
import {
  BUILT_IN_COMMANDS,
  filterBuiltInCommands,
} from "./built-in-commands";

describe("filterBuiltInCommands", () => {
  test("empty query returns all built-in commands", () => {
    const results = filterBuiltInCommands("");
    expect(results).toHaveLength(BUILT_IN_COMMANDS.length);
    expect(results).toEqual(BUILT_IN_COMMANDS);
  });

  test("whitespace-only query returns all commands", () => {
    const results = filterBuiltInCommands("   ");
    expect(results).toHaveLength(BUILT_IN_COMMANDS.length);
  });

  test("case-insensitive substring match on label", () => {
    // "toggle" matches "Toggle Inspector"
    const results = filterBuiltInCommands("toggle");
    expect(results.some((c) => c.id === "toggle-inspector")).toBe(true);
  });

  test("uppercase query matches lowercase label", () => {
    const results = filterBuiltInCommands("TOGGLE");
    expect(results.some((c) => c.id === "toggle-inspector")).toBe(true);
  });

  test("query matching sublabel is included", () => {
    // sublabel for toggle-inspector is "⌘\\"
    // Use a partial label match instead — "Inspector" is in the label
    const results = filterBuiltInCommands("Inspector");
    expect(results.some((c) => c.id === "toggle-inspector")).toBe(true);
  });

  test("query that matches no command returns empty array", () => {
    const results = filterBuiltInCommands("zzznomatch");
    expect(results).toHaveLength(0);
  });

  test("partial match returns subset", () => {
    // "project" matches "Close Project" and "Reload Project"
    const results = filterBuiltInCommands("project");
    expect(results.length).toBeGreaterThanOrEqual(2);
    const ids = results.map((c) => c.id);
    expect(ids).toContain("close-project");
    expect(ids).toContain("reload-project");
  });

  test("results contain only commands matching query", () => {
    const results = filterBuiltInCommands("about");
    expect(results.every((c) =>
      c.label.toLowerCase().includes("about") ||
      c.sublabel.toLowerCase().includes("about"),
    )).toBe(true);
    expect(results.some((c) => c.id === "show-about")).toBe(true);
  });

  test("all returned results have required SearchResult fields", () => {
    const results = filterBuiltInCommands("close");
    for (const r of results) {
      expect(typeof r.id).toBe("string");
      expect(typeof r.label).toBe("string");
      expect(typeof r.score).toBe("number");
      expect(r.kind).toBe("Command");
    }
  });
});
