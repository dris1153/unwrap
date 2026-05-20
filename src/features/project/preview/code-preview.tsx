/**
 * code-preview.tsx — decompile view for script asset nodes (phase-07).
 *
 * Queries `decompile(handle, nodeId)` via React Query.
 * Shows a shimmer skeleton while decompiling, renders Monaco on success,
 * and surfaces a retry card on error.
 *
 * MonacoHost is NOT imported here directly — it is loaded via React.lazy
 * inside this module so Monaco's ~2.5MB bundle stays out of the welcome bundle.
 */

import { lazy, Suspense } from "react";
import { useQuery } from "@tanstack/react-query";
import { ipc } from "../../../lib/ipc";
import { DecompileToolbar } from "./code-preview/decompile-toolbar";
import { ArrowCounterClockwise } from "@phosphor-icons/react";

// Lazy-load Monaco so the editor bundle is code-split from the welcome screen.
const MonacoHost = lazy(() =>
  import("./code-preview/monaco-host").then((m) => ({ default: m.MonacoHost }))
);

type CodePreviewProps = {
  /** Serialized ProjectHandle JSON string (passed as-is from the tab store). */
  projectHandle: string;
  nodeId: string;
};

// ---------------------------------------------------------------------------
// Loading skeleton
// ---------------------------------------------------------------------------

function DecompileSkeleton() {
  return (
    <div className="flex-1 flex flex-col bg-[#0C0C0E] overflow-hidden">
      {/* Toolbar placeholder */}
      <div className="flex items-center px-4 gap-3 border-b border-border-default flex-shrink-0" style={{ height: 36 }}>
        <div className="h-4 w-12 rounded bg-zinc-800 animate-pulse" />
        <div className="h-4 w-16 rounded bg-zinc-800 animate-pulse" />
      </div>
      {/* Code lines shimmer */}
      <div className="flex-1 p-4 space-y-2 overflow-hidden">
        {Array.from({ length: 18 }, (_, i) => (
          <div
            key={i}
            className="h-[14px] rounded bg-zinc-800/60 animate-pulse"
            style={{ width: `${45 + ((i * 37) % 45)}%`, animationDelay: `${i * 40}ms` }}
          />
        ))}
      </div>
    </div>
  );
}

// ---------------------------------------------------------------------------
// Error card
// ---------------------------------------------------------------------------

function DecompileError({ message, onRetry }: { message: string; onRetry: () => void }) {
  return (
    <div className="flex-1 flex flex-col items-center justify-center gap-4 bg-[#0C0C0E]">
      <div className="flex flex-col items-center gap-2 text-center max-w-[320px]">
        <span className="font-mono text-[12px] text-red-400">Decompile failed</span>
        <span className="font-mono text-[11px] text-zinc-500">{message}</span>
      </div>
      <button
        onClick={onRetry}
        className="h-8 px-4 rounded-[var(--radius-md)] bg-elevated border border-border-default hover:border-border-strong text-primary text-[12px] font-medium flex items-center gap-2 transition-colors"
      >
        <ArrowCounterClockwise size={13} />
        Retry
      </button>
    </div>
  );
}

// ---------------------------------------------------------------------------
// Main component
// ---------------------------------------------------------------------------

export function CodePreview({ projectHandle, nodeId }: CodePreviewProps) {
  // Parse handle — ipc.decompile expects a ProjectHandle object.
  let handle: ReturnType<typeof JSON.parse>;
  try {
    handle = JSON.parse(projectHandle);
  } catch {
    return (
      <div className="flex-1 flex items-center justify-center bg-[#0C0C0E]">
        <span className="font-mono text-[12px] text-red-400">Invalid project handle</span>
      </div>
    );
  }

  const { data, isLoading, isError, error, refetch } = useQuery({
    queryKey: ["decompile", projectHandle, nodeId],
    queryFn: () => ipc.decompile(handle, nodeId),
    staleTime: 5 * 60 * 1000, // cached decompile is stable
    retry: false,              // don't auto-retry ILSpy failures
  });

  if (isLoading) {
    return <DecompileSkeleton />;
  }

  if (isError || !data) {
    const msg =
      isError && error instanceof Error
        ? error.message
        : "Unknown error — check that ILSpy sidecar is installed.";
    return <DecompileError message={msg} onRetry={() => refetch()} />;
  }

  return (
    <div className="flex-1 flex flex-col min-h-0 overflow-hidden">
      <DecompileToolbar payload={data} />
      <Suspense
        fallback={
          <div className="flex-1 flex items-center justify-center bg-[#0C0C0E]">
            <span className="font-mono text-[12px] text-zinc-500">Loading editor…</span>
          </div>
        }
      >
        <MonacoHost content={data.content} language={data.language} />
      </Suspense>
    </div>
  );
}
