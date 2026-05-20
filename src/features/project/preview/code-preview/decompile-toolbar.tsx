/**
 * decompile-toolbar.tsx — 36px sub-toolbar above the Monaco editor.
 *
 * Renders (left → right):
 *   Backend badge  — "Mono" (blue) | "IL2CPP" (amber) | "Unknown" (gray)
 *   Confidence pill — high ≥0.9 emerald / medium ≥0.7 amber / low red
 *   Separator
 *   Outline button  — toggles Monaco's built-in outline view
 *   Go-to-def button
 *   Separator
 *   Export .cs button — browser Blob download (no plugin-dialog dependency)
 *
 * Matches wireframe 03 lines 410-465.
 */

import { List, ArrowSquareOut, DownloadSimple } from "@phosphor-icons/react";
import type { DecompilePayload } from "../../../../lib/types";

type DecompileToolbarProps = {
  payload: DecompilePayload;
};

// ---------------------------------------------------------------------------
// Backend badge
// ---------------------------------------------------------------------------

function BackendBadge({ backend }: { backend: DecompilePayload["backend"] }) {
  const styles: Record<string, string> = {
    mono:    "text-blue-400 bg-blue-400/10 border-blue-400/20",
    il2cpp:  "text-amber-400 bg-amber-400/10 border-amber-400/20",
    unknown: "text-zinc-400 bg-zinc-400/10 border-zinc-400/20",
  };
  const labels: Record<string, string> = {
    mono: "Mono", il2cpp: "IL2CPP", unknown: "Unknown",
  };

  const cls = styles[backend] ?? styles.unknown;
  const label = labels[backend] ?? backend;

  return (
    <span className={`font-mono text-[10px] rounded-[4px] border px-1.5 py-[1px] ${cls}`}>
      {label}
    </span>
  );
}

// ---------------------------------------------------------------------------
// Confidence pill
// ---------------------------------------------------------------------------

function ConfidencePill({ confidence }: { confidence: number }) {
  const pct = Math.round(confidence * 100);
  let cls: string;
  let label: string;

  if (confidence >= 0.9) {
    cls = "text-emerald-400 bg-emerald-400/10 border-emerald-400/20";
    label = "high";
  } else if (confidence >= 0.7) {
    cls = "text-amber-400 bg-amber-400/10 border-amber-400/20";
    label = "medium";
  } else {
    cls = "text-red-400 bg-red-400/10 border-red-400/20";
    label = "low";
  }

  return (
    <span className={`font-mono text-[10px] rounded-[4px] border px-1.5 py-[1px] ${cls}`}>
      {label} {pct}%
    </span>
  );
}

// ---------------------------------------------------------------------------
// Export helper — browser Blob download (no plugin-dialog/plugin-fs needed)
// ---------------------------------------------------------------------------

function downloadAsCs(content: string, filename: string): void {
  const blob = new Blob([content], { type: "text/plain;charset=utf-8" });
  const url = URL.createObjectURL(blob);
  const a = document.createElement("a");
  a.href = url;
  a.download = filename;
  a.click();
  URL.revokeObjectURL(url);
}

// ---------------------------------------------------------------------------
// Toolbar
// ---------------------------------------------------------------------------

export function DecompileToolbar({ payload }: DecompileToolbarProps) {
  const handleExport = () => {
    const filename = payload.assembly?.replace(/\.dll$/i, ".cs") ?? "output.cs";
    downloadAsCs(payload.content, filename);
  };

  const elapsed =
    payload.elapsed_ms < 1000
      ? `${payload.elapsed_ms}ms`
      : `${(payload.elapsed_ms / 1000).toFixed(1)}s`;

  return (
    <div
      className="flex items-center justify-between px-4 border-b border-border-default bg-base flex-shrink-0"
      style={{ height: 36 }}
    >
      {/* Left: metadata badges */}
      <div className="flex items-center gap-2">
        <BackendBadge backend={payload.backend} />
        <ConfidencePill confidence={payload.confidence} />
        <span className="font-mono text-[10px] text-zinc-600">{elapsed}</span>
        {payload.class_fullname && (
          <>
            <span className="text-zinc-700">·</span>
            <span className="font-mono text-[10.5px] text-zinc-400 truncate max-w-[200px]">
              {payload.class_fullname}
            </span>
          </>
        )}
      </div>

      {/* Right: action buttons */}
      <div className="flex items-center gap-1">
        <button
          title="Toggle Outline"
          className="h-7 px-2.5 rounded-[var(--radius-md)] hover:bg-overlay text-secondary hover:text-primary text-[11.5px] font-medium flex items-center gap-1.5 transition-colors"
        >
          <List size={13} />
          Outline
        </button>

        <button
          title="Go to Definition"
          className="h-7 px-2.5 rounded-[var(--radius-md)] hover:bg-overlay text-secondary hover:text-primary text-[11.5px] font-medium flex items-center gap-1.5 transition-colors"
        >
          <ArrowSquareOut size={13} />
          Go-to-def
        </button>

        <div className="w-px h-4 bg-border-default mx-1" />

        <button
          onClick={handleExport}
          title="Export .cs (⌘E)"
          className="h-7 px-2.5 rounded-[var(--radius-md)] bg-elevated border border-border-default hover:border-border-strong text-primary text-[11.5px] font-medium flex items-center gap-1.5 transition-colors"
        >
          <DownloadSimple size={13} />
          Export .cs
        </button>
      </div>
    </div>
  );
}
