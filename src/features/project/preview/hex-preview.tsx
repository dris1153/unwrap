import { useRef } from "react";
import { useVirtualizer } from "@tanstack/react-virtual";
import { useQuery } from "@tanstack/react-query";
import { ipc } from "../../../lib/ipc";
import type { PreviewPayload } from "../../../lib/types";

type HexPayload = Extract<PreviewPayload, { type: "hex" }>;

const BYTES_PER_ROW = 16;
const CHUNK_ROWS = 4096; // rows per IPC chunk = 64KB per call
const CHUNK_BYTES = BYTES_PER_ROW * CHUNK_ROWS;

function toHex2(n: number): string {
  return n.toString(16).padStart(2, "0").toUpperCase();
}

function toHex8(n: number): string {
  return n.toString(16).padStart(8, "0").toUpperCase();
}

function isPrintable(b: number): boolean {
  return b >= 0x20 && b < 0x7f;
}

type HexRowProps = {
  path: string;
  rowIndex: number;
  fileSize: number;
};

function HexRow({ path, rowIndex, fileSize }: HexRowProps) {
  const offset = rowIndex * BYTES_PER_ROW;
  const chunkBase = Math.floor(offset / CHUNK_BYTES) * CHUNK_BYTES;

  const { data: chunk } = useQuery({
    queryKey: ["hex-chunk", path, chunkBase],
    queryFn: () => ipc.readFileChunk(path, chunkBase, CHUNK_BYTES),
    staleTime: 5 * 60 * 1000,
    gcTime: 10 * 60 * 1000,
  });

  const rowOffset = offset - chunkBase;
  const bytes: (number | null)[] = Array.from({ length: BYTES_PER_ROW }, (_, i) => {
    const abs = offset + i;
    if (abs >= fileSize) return null;
    if (!chunk) return null;
    return chunk[rowOffset + i] ?? null;
  });

  return (
    <div className="flex items-center gap-3 font-mono text-[11.5px] h-[22px] hover:bg-[#18181b] px-3">
      {/* Offset gutter */}
      <span className="text-tertiary flex-shrink-0 w-[72px]">{toHex8(offset)}</span>

      {/* Hex bytes — two groups of 8 */}
      <span className="flex gap-1 flex-shrink-0">
        {bytes.slice(0, 8).map((b, i) => (
          <span key={i} className={b === null ? "text-[#1c1c1f]" : "text-primary"}>
            {b === null ? ".." : toHex2(b)}
          </span>
        ))}
      </span>
      <span className="flex gap-1 flex-shrink-0">
        {bytes.slice(8, 16).map((b, i) => (
          <span key={i} className={b === null ? "text-[#1c1c1f]" : "text-primary"}>
            {b === null ? ".." : toHex2(b)}
          </span>
        ))}
      </span>

      {/* ASCII column */}
      <span className="text-secondary flex-shrink-0 tracking-widest">
        {bytes.map((b, i) =>
          b === null ? " " : isPrintable(b) ? String.fromCharCode(b) : (
            <span key={i} className="text-tertiary">.</span>
          ),
        )}
      </span>
    </div>
  );
}

export function HexPreview({ path, size }: HexPayload) {
  const scrollRef = useRef<HTMLDivElement>(null);
  const totalRows = Math.ceil(size / BYTES_PER_ROW);

  const rowVirtualizer = useVirtualizer({
    count: totalRows,
    getScrollElement: () => scrollRef.current,
    estimateSize: () => 22,
    overscan: 20,
  });

  return (
    <div className="flex-1 flex flex-col min-h-0 bg-base">
      {/* Header */}
      <div className="flex items-center gap-3 px-3 border-b border-border-default font-mono text-[10px] text-tertiary flex-shrink-0" style={{ height: 28 }}>
        <span className="w-[72px] flex-shrink-0">Offset</span>
        <span className="flex gap-1">
          {Array.from({ length: 8 }, (_, i) => (
            <span key={i} className="w-[22px]">{toHex2(i)}</span>
          ))}
        </span>
        <span className="flex gap-1">
          {Array.from({ length: 8 }, (_, i) => (
            <span key={i} className="w-[22px]">{toHex2(i + 8)}</span>
          ))}
        </span>
        <span>ASCII</span>
      </div>

      {/* Virtualized rows */}
      <div ref={scrollRef} className="flex-1 overflow-y-auto scroll-thin">
        <div style={{ height: rowVirtualizer.getTotalSize(), position: "relative" }}>
          {rowVirtualizer.getVirtualItems().map((vItem) => (
            <div
              key={vItem.index}
              style={{ position: "absolute", top: vItem.start, left: 0, right: 0 }}
            >
              <HexRow path={path} rowIndex={vItem.index} fileSize={size} />
            </div>
          ))}
        </div>
      </div>

      {/* Footer */}
      <div
        className="flex items-center justify-between px-3 border-t border-border-default font-mono text-[10px] text-tertiary flex-shrink-0"
        style={{ height: 24 }}
      >
        <span>{(size / (1024 * 1024)).toFixed(2)} MB</span>
        <span>{totalRows.toLocaleString()} rows</span>
      </div>
    </div>
  );
}
