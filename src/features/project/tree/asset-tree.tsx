import React, { useCallback, useRef, useState, useEffect } from "react";
import { useVirtualizer } from "@tanstack/react-virtual";
import type { AssetTree } from "../../../lib/types";
import { useTreeStore } from "../../../stores/use-tree-store";
import { useTabStore } from "../../../stores/use-tab-store";
import { useFlattenedTree } from "./use-flattened-tree";
import { TreeRow } from "./tree-row";
import { TreeFilter } from "./tree-filter";
import { buildPathLabel } from "./build-path-label";

type AssetTreeProps = {
  projectId: string;
  tree: AssetTree;
};

/**
 * Virtualized asset tree using TanStack Virtual.
 * Row height: 24px fixed. Overscan: 10.
 */
export function AssetTree({ projectId, tree }: AssetTreeProps) {
  const [filter, setFilter] = useState("");
  const scrollRef = useRef<HTMLDivElement>(null);

  const expandedIds =
    useTreeStore((s) => s.expandedIds[projectId]) ?? new Set<string>();
  const selectedId = useTreeStore((s) => s.selectedId[projectId]) ?? null;
  const { toggleExpanded, setSelected } = useTreeStore();
  const { openTab } = useTabStore();

  const visibleRows = useFlattenedTree(tree, expandedIds, filter);

  const rowVirtualizer = useVirtualizer({
    count: visibleRows.length,
    getScrollElement: () => scrollRef.current,
    estimateSize: () => 24,
    overscan: 10,
  });

  const totalCount = Object.keys(tree.nodes).length;

  // Keyboard navigation
  const handleKeyDown = useCallback(
    (e: React.KeyboardEvent) => {
      const currentIdx = visibleRows.findIndex((r) => r.nodeId === selectedId);
      if (e.key === "ArrowDown") {
        e.preventDefault();
        const nextIdx = Math.min(currentIdx + 1, visibleRows.length - 1);
        const next = visibleRows[nextIdx];
        if (next) {
          setSelected(projectId, next.nodeId, next.node);
          rowVirtualizer.scrollToIndex(nextIdx, { align: "auto" });
        }
      } else if (e.key === "ArrowUp") {
        e.preventDefault();
        const prevIdx = Math.max(currentIdx - 1, 0);
        const prev = visibleRows[prevIdx];
        if (prev) {
          setSelected(projectId, prev.nodeId, prev.node);
          rowVirtualizer.scrollToIndex(prevIdx, { align: "auto" });
        }
      } else if (e.key === "ArrowRight") {
        if (selectedId) toggleExpanded(projectId, selectedId);
      } else if (e.key === "ArrowLeft") {
        if (selectedId) toggleExpanded(projectId, selectedId);
      } else if (e.key === "Enter") {
        const cur = visibleRows[currentIdx];
        if (cur && cur.node.kind !== "folder") {
          openTab(projectId, cur.node, buildPathLabel(tree, cur.nodeId));
        }
      }
    },
    [visibleRows, selectedId, projectId, setSelected, toggleExpanded, openTab, tree, rowVirtualizer],
  );

  // Auto-expand root on load
  useEffect(() => {
    const rootId = tree.root["0"];
    if (!expandedIds.has(rootId)) {
      useTreeStore.getState().setExpanded(projectId, rootId, true);
    }
  // Only run when tree root changes
  // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [tree.root["0"], projectId]);

  return (
    <div className="flex flex-col min-h-0 flex-1">
      <TreeFilter
        value={filter}
        onChange={setFilter}
        resultCount={filter ? visibleRows.length : undefined}
      />

      {/* Scrollable tree body */}
      <div
        ref={scrollRef}
        role="tree"
        aria-label="Asset tree"
        tabIndex={0}
        onKeyDown={handleKeyDown}
        className="flex-1 overflow-y-auto scroll-thin py-1 outline-none"
      >
        <div
          style={{
            height: rowVirtualizer.getTotalSize(),
            position: "relative",
          }}
        >
          {rowVirtualizer.getVirtualItems().map((vItem) => {
            const row = visibleRows[vItem.index];
            if (!row) return null;
            const isSelected = row.nodeId === selectedId;
            const isExpanded = expandedIds.has(row.nodeId);

            return (
              <div
                key={row.nodeId}
                style={{
                  position: "absolute",
                  top: vItem.start,
                  left: 0,
                  right: 0,
                  height: vItem.size,
                }}
              >
                <TreeRow
                  row={row}
                  isSelected={isSelected}
                  isExpanded={isExpanded}
                  onSelect={() => {
                    setSelected(projectId, row.nodeId, row.node);
                    if (row.node.kind !== "folder") {
                      openTab(
                        projectId,
                        row.node,
                        buildPathLabel(tree, row.nodeId),
                      );
                    }
                  }}
                  onToggleExpand={() => toggleExpanded(projectId, row.nodeId)}
                />
              </div>
            );
          })}
        </div>
      </div>

      {/* Footer mini-stats */}
      <div
        className="flex items-center justify-between px-3 border-t border-border-default font-mono text-[10px] text-tertiary flex-shrink-0"
        style={{ height: 24 }}
      >
        <span className="flex items-center gap-1.5">
          <span className="w-[5px] h-[5px] rounded-full bg-accent flex-shrink-0" />
          indexed
        </span>
        <span>
          {filter
            ? `${visibleRows.length} of ${totalCount}`
            : `${totalCount.toLocaleString()}`}
        </span>
      </div>
    </div>
  );
}
