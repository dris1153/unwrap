// features/command-palette/command-palette.tsx
// Cmd+K command palette built on cmdk. Mounts once in root layout via portal.

import { useEffect, useRef, useCallback } from "react";
import { Command } from "cmdk";
import { motion, AnimatePresence } from "framer-motion";
import { useDebounce } from "use-debounce";
import { createPortal } from "react-dom";
import { useNavigate } from "@tanstack/react-router";

import { ipc } from "../../lib/ipc";
import type { SearchResult } from "../../lib/types";
import { useUiStore } from "../../stores/use-ui-store";
import { useTabStore } from "../../stores/use-tab-store";
import { useTreeStore } from "../../stores/use-tree-store";
import { usePaletteStore } from "./use-palette-store";
import {
  BUILT_IN_COMMANDS,
  filterBuiltInCommands,
} from "./built-in-commands";
import { useQueryClient } from "@tanstack/react-query";
import { useState } from "react";

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

function groupResults(results: SearchResult[]): {
  commands: SearchResult[];
  assets: SearchResult[];
  code: SearchResult[];
  strings: SearchResult[];
} {
  return {
    commands: results.filter((r) => r.kind === "Command"),
    assets: results.filter((r) => r.kind === "Asset"),
    code: results.filter((r) => r.kind === "Code"),
    strings: results.filter((r) => r.kind === "String"),
  };
}

// ---------------------------------------------------------------------------
// Result item
// ---------------------------------------------------------------------------

function ResultItem({
  result,
  onSelect,
}: {
  result: SearchResult;
  onSelect: (r: SearchResult) => void;
}) {
  return (
    <Command.Item
      key={result.id}
      value={`${result.kind}:${result.id}:${result.label}`}
      onSelect={() => onSelect(result)}
      className="flex items-center gap-3 px-3 py-2 rounded-lg cursor-pointer
                 text-[13px] text-primary
                 data-[selected=true]:bg-overlay
                 hover:bg-overlay transition-colors"
    >
      <span className="text-tertiary flex-shrink-0 w-4 text-center text-[12px]">
        {iconGlyph(result.icon)}
      </span>
      <span className="flex-1 min-w-0">
        <span className="block truncate font-medium">{result.label}</span>
        {result.sublabel && (
          <span className="block truncate text-[11px] text-tertiary font-mono mt-0.5">
            {result.sublabel}
          </span>
        )}
      </span>
      {result.kind === "Command" && result.sublabel && (
        <kbd className="ml-auto text-[10px] text-tertiary bg-surface border border-border-default rounded px-1.5 py-0.5 font-mono flex-shrink-0">
          {result.sublabel}
        </kbd>
      )}
    </Command.Item>
  );
}

/** Map phosphor icon names to a simple text glyph for palette display. */
function iconGlyph(icon: string): string {
  const map: Record<string, string> = {
    "folder": "📁",
    "image": "🖼",
    "speaker-simple-high": "🔊",
    "cube": "⬛",
    "file-text": "📄",
    "code": "</>",
    "film-slate": "🎬",
    "paint-bucket": "🪣",
    "sparkle": "✨",
    "film-strip": "🎞",
    "package": "📦",
    "file-binary": "⬛",
    "translate": "🌐",
    "file": "📄",
    "sidebar-simple-right": "▶",
    "magnifying-glass": "🔍",
    "x-circle": "✕",
    "arrows-clockwise": "↺",
    "info": "ℹ",
  };
  return map[icon] ?? "•";
}

// ---------------------------------------------------------------------------
// Main palette
// ---------------------------------------------------------------------------

export function CommandPalette() {
  const { open, query, close, setQuery, pushRecent, recents } =
    usePaletteStore();
  const [searchResults, setSearchResults] = useState<SearchResult[]>([]);
  const [isSearching, setIsSearching] = useState(false);

  const [debouncedQuery] = useDebounce(query, 120);

  const navigate = useNavigate();
  const qc = useQueryClient();
  const uiStore = useUiStore();
  const tabStore = useTabStore();
  const treeStore = useTreeStore();

  // Derive active project id from active tab
  const activeProjectId = useTabStore((s) => {
    const active = s.tabs.find((t) => t.id === s.activeTabId);
    return active?.projectId ?? null;
  });

  // FTS5 search on debounced query
  useEffect(() => {
    if (!debouncedQuery.trim()) {
      setSearchResults([]);
      return;
    }
    setIsSearching(true);
    ipc
      .search(debouncedQuery, activeProjectId ?? undefined)
      .then((results) => setSearchResults(results))
      .catch(() => setSearchResults([]))
      .finally(() => setIsSearching(false));
  }, [debouncedQuery, activeProjectId]);

  // Merge built-in commands (client-side filtered) with FTS results
  const builtIns = filterBuiltInCommands(query);
  const merged: SearchResult[] = query.trim()
    ? [...builtIns, ...searchResults]
    : BUILT_IN_COMMANDS;

  const groups = groupResults(merged);

  // ---------------------------------------------------------------------------
  // Action dispatch
  // ---------------------------------------------------------------------------

  const dispatch = useCallback(
    (result: SearchResult) => {
      close();
      pushRecent(result.label);

      const action = result.action;

      switch (action.type) {
        case "OpenAsset": {
          const { project_id, node_id } = action;
          // Load tree from query cache to get node data
          const tree = qc.getQueryData<{ nodes: Record<string, { id: { "0": string }; name: string; kind: string; size: number; source_path: string; metadata: Record<string, unknown>; parent: { "0": string } | null }> }>(["tree", project_id]);
          const nodeData = tree?.nodes?.[node_id];
          if (nodeData) {
            treeStore.setSelected(project_id, node_id, nodeData as Parameters<typeof treeStore.setSelected>[2]);
            tabStore.openTab(project_id, nodeData as Parameters<typeof tabStore.openTab>[1]);
          }
          break;
        }
        case "OpenCode": {
          const { project_id, node_id } = action;
          const tree = qc.getQueryData<{ nodes: Record<string, Parameters<typeof tabStore.openTab>[1]> }>(["tree", project_id]);
          const nodeData = tree?.nodes?.[node_id];
          if (nodeData) {
            treeStore.setSelected(project_id, node_id, nodeData as Parameters<typeof treeStore.setSelected>[2]);
            tabStore.openTab(project_id, nodeData as Parameters<typeof tabStore.openTab>[1]);
          }
          break;
        }
        case "OpenString": {
          uiStore.setActiveRail("strings");
          // Row scroll deferred — translate-list will handle via anchor
          break;
        }
        case "Command": {
          handleCommand(action.command_id);
          break;
        }
      }
    },
    [close, pushRecent, qc, treeStore, tabStore, uiStore, navigate, activeProjectId],
  );

  const handleCommand = useCallback(
    (commandId: string) => {
      switch (commandId) {
        case "toggle-inspector":
          uiStore.toggleInspector();
          break;
        case "focus-filter": {
          const el = document.querySelector<HTMLElement>("[data-tree-filter]");
          el?.focus();
          break;
        }
        case "close-project":
          tabStore.closeAllForProject(activeProjectId ?? "");
          navigate({ to: "/" });
          break;
        case "reload-project":
          if (activeProjectId) {
            qc.invalidateQueries({ queryKey: ["tree", activeProjectId] });
          }
          break;
        case "show-about":
          // Placeholder — about modal not yet implemented
          alert("Unwrap v0.1.0");
          break;
        default:
          break;
      }
    },
    [uiStore, tabStore, navigate, activeProjectId, qc],
  );

  // Close on backdrop click
  const backdropRef = useRef<HTMLDivElement>(null);

  const palette = (
    <AnimatePresence>
      {open && (
        <div
          ref={backdropRef}
          className="fixed inset-0 z-50 flex items-start justify-center pt-[15vh]"
          style={{
            backdropFilter: "blur(8px)",
            backgroundColor: "rgba(var(--color-base-rgb, 0 0 0) / 0.60)",
          }}
          onClick={(e) => {
            if (e.target === backdropRef.current) close();
          }}
        >
          <motion.div
            initial={{ scale: 0.96, opacity: 0 }}
            animate={{ scale: 1, opacity: 1 }}
            exit={{ scale: 0.96, opacity: 0 }}
            transition={{ type: "spring", stiffness: 200, damping: 25 }}
            className="w-full max-w-[640px] mx-4"
          >
            <Command
              className="bg-elevated border border-border-default rounded-xl shadow-lg overflow-hidden"
              shouldFilter={false}
            >
              {/* Search input */}
              <div className="flex items-center gap-2 px-3 border-b border-border-default">
                <span className="text-tertiary text-[13px]">🔍</span>
                <Command.Input
                  value={query}
                  onValueChange={setQuery}
                  placeholder="Search assets, scripts, strings, commands…"
                  className="flex-1 bg-transparent py-3 text-[13px] text-primary placeholder:text-tertiary outline-none font-mono"
                  autoFocus
                />
                {isSearching && (
                  <span className="text-[11px] text-tertiary animate-pulse">
                    searching…
                  </span>
                )}
                <kbd className="text-[10px] text-tertiary bg-surface border border-border-default rounded px-1.5 py-0.5 font-mono">
                  esc
                </kbd>
              </div>

              <Command.List className="max-h-[420px] overflow-y-auto p-1.5">
                {/* Empty state */}
                <Command.Empty className="py-8 text-center text-[13px] text-tertiary font-mono">
                  {query.trim()
                    ? "No results · try a different query"
                    : "Type to search assets, scripts, strings…"}
                </Command.Empty>

                {/* Recent searches (shown only when query empty and recents exist) */}
                {!query.trim() && recents.length > 0 && (
                  <Command.Group
                    heading={
                      <span className="px-3 py-1.5 text-[10px] font-semibold uppercase tracking-wider text-tertiary block">
                        Recent
                      </span>
                    }
                  >
                    {recents.map((label) => (
                      <Command.Item
                        key={`recent:${label}`}
                        value={`recent:${label}`}
                        onSelect={() => setQuery(label)}
                        className="flex items-center gap-3 px-3 py-2 rounded-lg cursor-pointer
                                   text-[13px] text-secondary
                                   data-[selected=true]:bg-overlay hover:bg-overlay transition-colors"
                      >
                        <span className="text-tertiary text-[12px]">↩</span>
                        <span className="truncate">{label}</span>
                      </Command.Item>
                    ))}
                  </Command.Group>
                )}

                {/* Commands group */}
                {groups.commands.length > 0 && (
                  <Command.Group
                    heading={
                      <span className="px-3 py-1.5 text-[10px] font-semibold uppercase tracking-wider text-tertiary block">
                        Commands
                      </span>
                    }
                  >
                    {groups.commands.map((r) => (
                      <ResultItem key={r.id} result={r} onSelect={dispatch} />
                    ))}
                  </Command.Group>
                )}

                {/* Assets group */}
                {groups.assets.length > 0 && (
                  <Command.Group
                    heading={
                      <span className="px-3 py-1.5 text-[10px] font-semibold uppercase tracking-wider text-tertiary block">
                        Assets
                      </span>
                    }
                  >
                    {groups.assets.map((r) => (
                      <ResultItem key={r.id} result={r} onSelect={dispatch} />
                    ))}
                  </Command.Group>
                )}

                {/* Code group */}
                {groups.code.length > 0 && (
                  <Command.Group
                    heading={
                      <span className="px-3 py-1.5 text-[10px] font-semibold uppercase tracking-wider text-tertiary block">
                        Code
                      </span>
                    }
                  >
                    {groups.code.map((r) => (
                      <ResultItem key={r.id} result={r} onSelect={dispatch} />
                    ))}
                  </Command.Group>
                )}

                {/* Strings group */}
                {groups.strings.length > 0 && (
                  <Command.Group
                    heading={
                      <span className="px-3 py-1.5 text-[10px] font-semibold uppercase tracking-wider text-tertiary block">
                        Strings
                      </span>
                    }
                  >
                    {groups.strings.map((r) => (
                      <ResultItem key={r.id} result={r} onSelect={dispatch} />
                    ))}
                  </Command.Group>
                )}
              </Command.List>
            </Command>
          </motion.div>
        </div>
      )}
    </AnimatePresence>
  );

  return createPortal(palette, document.body);
}
