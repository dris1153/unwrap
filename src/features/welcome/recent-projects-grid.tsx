import { useQuery, useQueryClient } from "@tanstack/react-query";
import {
  ArrowsClockwise,
  ClockCounterClockwise,
  Broom,
  Keyboard,
  GameController,
  Cube,
  Knife,
  MapTrifold,
} from "@phosphor-icons/react";
import { useRouter } from "@tanstack/react-router";
import { ipc } from "../../lib/ipc";
import type { RecentEntry } from "../../lib/types";
import { cn } from "../../lib/cn";

/* ------------------------------------------------------------------ */
/* Chip styles by variant                                               */
/* ------------------------------------------------------------------ */
type ChipVariant = "pinned" | "il2cpp" | "mono";

const chipStyles: Record<ChipVariant, string> = {
  pinned: "text-accent",
  il2cpp: "text-warning",
  mono: "text-info",
};

const chipDotStyles: Record<ChipVariant, string> = {
  pinned: "bg-accent",
  il2cpp: "bg-warning",
  mono: "bg-info",
};

/* ------------------------------------------------------------------ */
/* Icon + colour by iconTheme                                           */
/* ------------------------------------------------------------------ */
type IconTheme = "tex" | "mesh" | "script" | "txt";

function TileIcon({ theme }: { theme: IconTheme }) {
  const iconMap: Record<IconTheme, { icon: React.ReactNode; bg: string; border: string }> = {
    tex:    { icon: <GameController size={15} />, bg: "bg-tex/15",    border: "border-tex/30" },
    mesh:   { icon: <Cube size={15} />,           bg: "bg-mesh/15",   border: "border-mesh/30" },
    script: { icon: <Knife size={15} />,          bg: "bg-script/15", border: "border-script/30" },
    txt:    { icon: <MapTrifold size={15} />,     bg: "bg-txt/15",    border: "border-txt/30" },
  };

  const iconColorMap: Record<IconTheme, string> = {
    tex:    "text-tex",
    mesh:   "text-mesh",
    script: "text-script",
    txt:    "text-txt",
  };

  const { icon, bg, border } = iconMap[theme];

  return (
    <div
      className={cn(
        "w-[28px] h-[28px] rounded-md border flex items-center justify-center",
        bg,
        border,
        iconColorMap[theme],
      )}
    >
      {icon}
    </div>
  );
}

/* ------------------------------------------------------------------ */
/* Mapping helpers                                                      */
/* ------------------------------------------------------------------ */

/** Derive icon theme from scripting backend. */
function backendToIconTheme(backend: string): IconTheme {
  switch (backend) {
    case "mono":   return "script";
    case "il2cpp": return "mesh";
    default:       return "tex";
  }
}

/** Derive chip from entry. */
function deriveChip(entry: RecentEntry): { variant: ChipVariant; label: string } | null {
  if (entry.pinned) return { variant: "pinned", label: "pinned" };
  if (entry.project.scripting_backend === "il2cpp") return { variant: "il2cpp", label: "il2cpp" };
  if (entry.project.scripting_backend === "mono") return { variant: "mono", label: "mono" };
  return null;
}

/** Human-readable relative time from unix seconds. */
function relativeTime(unixSeconds: number): string {
  const diffSec = Date.now() / 1000 - unixSeconds;
  if (diffSec < 60) return "just now";
  if (diffSec < 3600) return `${Math.floor(diffSec / 60)}m ago`;
  if (diffSec < 86400) return `${Math.floor(diffSec / 3600)}h ago`;
  if (diffSec < 86400 * 7) return `${Math.floor(diffSec / 86400)}d ago`;
  return `${Math.floor(diffSec / (86400 * 7))}w ago`;
}

/** Extract last path segment as display name. */
function basename(path: string): string {
  const parts = path.replace(/\\/g, "/").split("/").filter(Boolean);
  return parts[parts.length - 1] ?? path;
}

/* ------------------------------------------------------------------ */
/* Single tile                                                          */
/* ------------------------------------------------------------------ */
function ProjectTile({ entry, onClick }: { entry: RecentEntry; onClick: () => void }) {
  const chip = deriveChip(entry);
  const iconTheme = backendToIconTheme(entry.project.scripting_backend);
  const isPinned = chip?.variant === "pinned";
  const name = basename(entry.project.root_path);
  const unityVersion = entry.project.engine_version ?? "Unknown";
  const openedLabel = relativeTime(entry.project.last_opened);

  return (
    <div
      onClick={onClick}
      className={cn(
        "group rounded-lg bg-surface border transition-all cursor-pointer relative overflow-hidden",
        isPinned
          ? "border-accent/30 hover:border-accent/40"
          : "border-border-default hover:border-border-strong",
      )}
      style={{ padding: 14, height: 138 }}
    >
      {/* Subtle tinted bg for pinned tile */}
      {isPinned && (
        <div className="absolute inset-0 bg-accent-muted pointer-events-none" />
      )}

      <div className="relative h-full flex flex-col justify-between">
        <div>
          <div className="flex items-start justify-between mb-2">
            <TileIcon theme={iconTheme} />
            {chip && (
              <span
                className={cn(
                  "font-mono text-[10px] flex items-center gap-1",
                  chipStyles[chip.variant],
                )}
              >
                <span
                  className={cn(
                    "w-[5px] h-[5px] rounded-full",
                    chipDotStyles[chip.variant],
                  )}
                />
                {chip.label}
              </span>
            )}
          </div>

          <div className="font-sans text-[14px] font-semibold text-primary leading-[18px] mb-1 tracking-tightish">
            {name}
          </div>
          <div className="font-mono text-[10px] text-tertiary truncate">
            {entry.project.root_path}
          </div>
        </div>

        <div className="flex items-center justify-between">
          <span className="font-mono text-[10px] text-secondary bg-elevated border border-border-default rounded-[4px] px-1.5 py-[1px]">
            {unityVersion}
          </span>
          <div className="text-right">
            {/* asset_count not available in v0.1 — deferred to v0.2 */}
            <div className="font-mono text-[10px] text-secondary">— assets</div>
            <div className="font-mono text-[10px] text-tertiary">
              {openedLabel}
            </div>
          </div>
        </div>
      </div>
    </div>
  );
}

/* ------------------------------------------------------------------ */
/* Skeleton tile (loading state)                                        */
/* ------------------------------------------------------------------ */
function SkeletonTile() {
  return (
    <div
      className="rounded-lg border border-border-default relative overflow-hidden"
      style={{ height: 138 }}
    >
      <div className="shimmer absolute inset-0 rounded-lg" />
    </div>
  );
}

/* ------------------------------------------------------------------ */
/* Header row                                                           */
/* ------------------------------------------------------------------ */
function Header({ count, onRefresh }: { count: number | null; onRefresh: () => void }) {
  return (
    <div className="flex items-center justify-between mb-4">
      <div className="flex items-center gap-2">
        <span className="font-mono text-[10px] tracking-[0.12em] uppercase text-tertiary">
          Recent
        </span>
        {count !== null && (
          <span className="font-mono text-[10px] text-secondary bg-elevated border border-border-default rounded-[4px] px-1.5 py-[1px]">
            {count}
          </span>
        )}
      </div>
      <button
        onClick={onRefresh}
        className="text-secondary hover:text-primary text-[12px] font-medium flex items-center gap-1 transition-colors"
      >
        <ArrowsClockwise size={12} />
        Refresh
      </button>
    </div>
  );
}

/* ------------------------------------------------------------------ */
/* Footer hints                                                         */
/* ------------------------------------------------------------------ */
function FooterHints() {
  return (
    <div className="mt-auto pt-6 flex items-center gap-2 font-mono text-[10px] text-tertiary">
      <span className="inline-flex items-center gap-1 px-1.5 py-[2px] border border-border-default rounded-[4px] bg-elevated">
        <Keyboard size={10} />
        ⌘O
      </span>
      <span>open folder</span>
      <span className="mx-1 text-overlay">·</span>
      <span className="inline-flex items-center gap-1 px-1.5 py-[2px] border border-border-default rounded-[4px] bg-elevated">
        ⌘K
      </span>
      <span>jump anywhere</span>
    </div>
  );
}

/* ------------------------------------------------------------------ */
/* Grid                                                                 */
/* ------------------------------------------------------------------ */
export function RecentProjectsGrid() {
  const router = useRouter();
  const qc = useQueryClient();

  const { data, isLoading, isError } = useQuery({
    queryKey: ["recents"],
    queryFn: () => ipc.listRecents(),
  });

  const recents = data ?? [];

  function handleOpen(entry: RecentEntry) {
    void (async () => {
      try {
        const handle = await ipc.openProject(entry.project.root_path);
        const projectId =
          typeof handle.id === "object" && "0" in handle.id
            ? (handle.id as { "0": string })["0"]
            : String(handle.id);
        router.navigate({ to: "/project/$projectId", params: { projectId } });
      } catch (err) {
        console.error("[RecentProjectsGrid] open failed:", err);
      }
    })();
  }

  function handleRefresh() {
    void qc.invalidateQueries({ queryKey: ["recents"] });
  }

  // Loading state — shimmer skeleton
  if (isLoading) {
    return (
      <section className="flex flex-col" style={{ width: "40%" }}>
        <Header count={null} onRefresh={handleRefresh} />
        <div className="grid grid-cols-2 gap-3 mb-5">
          {[1, 2, 3, 4].map((i) => (
            <SkeletonTile key={i} />
          ))}
        </div>
      </section>
    );
  }

  // Error state — minimal, not blocking
  if (isError) {
    return (
      <section className="flex flex-col" style={{ width: "40%" }}>
        <Header count={0} onRefresh={handleRefresh} />
        <div className="rounded-2xl border border-dashed border-border-default px-6 py-10 flex flex-col items-center justify-center text-center min-h-[280px]">
          <ClockCounterClockwise size={32} className="text-tertiary mb-3" />
          <div className="font-sans text-[14px] font-semibold text-secondary mb-1">
            Could not load recents
          </div>
          <div className="font-sans text-[12px] text-tertiary max-w-[260px]">
            Check the app data directory or restart Unwrap.
          </div>
        </div>
        <FooterHints />
      </section>
    );
  }

  // Empty state — DB has no entries yet
  if (recents.length === 0) {
    return (
      <section className="flex flex-col" style={{ width: "40%" }}>
        <Header count={0} onRefresh={handleRefresh} />
        <div className="rounded-2xl border border-dashed border-border-default px-6 py-10 flex flex-col items-center justify-center text-center min-h-[280px]">
          <ClockCounterClockwise size={32} className="text-tertiary mb-3" />
          <div className="font-sans text-[14px] font-semibold text-secondary mb-1">
            No recent projects yet
          </div>
          <div className="font-sans text-[12px] text-tertiary max-w-[260px]">
            Drop a Unity build folder on the left or press ⌘O to begin.
          </div>
        </div>
        <FooterHints />
      </section>
    );
  }

  // Populated state
  return (
    <section className="flex flex-col" style={{ width: "40%" }}>
      <Header count={recents.length} onRefresh={handleRefresh} />

      {/* 2×2 grid, max 4 tiles */}
      <div className="grid grid-cols-2 gap-3 mb-5">
        {recents.slice(0, 4).map((entry) => (
          <ProjectTile
            key={entry.project.id["0"]}
            entry={entry}
            onClick={() => handleOpen(entry)}
          />
        ))}
      </div>

      {/* Overflow hint */}
      {recents.length > 4 && (
        <div className="text-center text-tertiary text-[11px] font-mono mb-3">
          + {recents.length - 4} more (clear or pin to organize)
        </div>
      )}

      {/* No-more-projects footer row */}
      <div className="flex items-center justify-between rounded-lg border border-border-default border-dashed px-3.5 py-2.5">
        <div className="flex items-center gap-2.5">
          <ClockCounterClockwise size={14} className="text-tertiary" />
          <span className="font-sans text-[12px] text-tertiary">No more projects</span>
        </div>
        <button className="text-secondary hover:text-primary text-[12px] font-medium flex items-center gap-1 transition-colors">
          <Broom size={12} />
          Clear history
        </button>
      </div>

      <FooterHints />
    </section>
  );
}
