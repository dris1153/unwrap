// features/command-palette/built-in-commands.ts
// Static list of built-in app commands shown in the palette.

import type { SearchResult } from "../../lib/types";

export const BUILT_IN_COMMANDS: SearchResult[] = [
  {
    kind: "Command",
    id: "toggle-inspector",
    label: "Toggle Inspector",
    sublabel: "⌘\\",
    icon: "sidebar-simple-right",
    project_id: null,
    action: { type: "Command", command_id: "toggle-inspector" },
    score: 1.5,
  },
  {
    kind: "Command",
    id: "focus-filter",
    label: "Focus Tree Filter",
    sublabel: "⌘/",
    icon: "magnifying-glass",
    project_id: null,
    action: { type: "Command", command_id: "focus-filter" },
    score: 1.5,
  },
  {
    kind: "Command",
    id: "close-project",
    label: "Close Project",
    sublabel: "returns to welcome",
    icon: "x-circle",
    project_id: null,
    action: { type: "Command", command_id: "close-project" },
    score: 1.5,
  },
  {
    kind: "Command",
    id: "reload-project",
    label: "Reload Project",
    sublabel: "rebuild tree",
    icon: "arrows-clockwise",
    project_id: null,
    action: { type: "Command", command_id: "reload-project" },
    score: 1.5,
  },
  {
    kind: "Command",
    id: "show-about",
    label: "About Unwrap",
    sublabel: "version + licenses",
    icon: "info",
    project_id: null,
    action: { type: "Command", command_id: "show-about" },
    score: 1.5,
  },
];

/** Filter built-in commands by query string (case-insensitive includes). */
export function filterBuiltInCommands(query: string): SearchResult[] {
  if (!query.trim()) return BUILT_IN_COMMANDS;
  const lower = query.toLowerCase();
  return BUILT_IN_COMMANDS.filter(
    (c) =>
      c.label.toLowerCase().includes(lower) ||
      c.sublabel.toLowerCase().includes(lower),
  );
}
