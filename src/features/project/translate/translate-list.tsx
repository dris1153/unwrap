import { useRef } from "react";
import { useVirtualizer } from "@tanstack/react-virtual";
import type { TranslatableRowWithDraft } from "../../../lib/types";
import type { TranslateFilter } from "../../../stores/use-translate-store";
import { TranslateRow } from "./translate-row";

type Props = {
  rows: TranslatableRowWithDraft[];
  targetLocale: string;
  filter: TranslateFilter;
};

/** Virtualized list of translation rows. Handles 5k+ entries at 60fps. */
export function TranslateList({ rows, targetLocale, filter }: Props) {
  const parentRef = useRef<HTMLDivElement>(null);

  // Apply client-side filter on top of backend-returned rows.
  const filtered = rows.filter((r) => {
    if (filter === "all") return true;
    const status = r.translation?.status ?? "Pending";
    return status.toLowerCase() === filter;
  });

  const virtualizer = useVirtualizer({
    count: filtered.length,
    getScrollElement: () => parentRef.current,
    estimateSize: () => 76,
    overscan: 10,
  });

  const items = virtualizer.getVirtualItems();

  return (
    <div
      ref={parentRef}
      className="flex-1 overflow-y-auto"
      style={{ scrollbarWidth: "thin", scrollbarColor: "#27272A transparent" }}
    >
      <div
        style={{
          height: virtualizer.getTotalSize(),
          position: "relative",
        }}
      >
        {items.map((vItem) => {
          const row = filtered[vItem.index];
          return (
            <div
              key={row.id}
              style={{
                position: "absolute",
                top: 0,
                left: 0,
                right: 0,
                transform: `translateY(${vItem.start}px)`,
              }}
            >
              <TranslateRow item={row} targetLocale={targetLocale} />
            </div>
          );
        })}
      </div>
    </div>
  );
}
