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
import { cn } from "../../lib/cn";
import { type RecentProject, MOCK_RECENTS } from "../../lib/mock-recents";

/* ------------------------------------------------------------------ */
/* Chip styles by variant                                               */
/* ------------------------------------------------------------------ */
type ChipVariant = NonNullable<RecentProject["chip"]>["variant"];

const chipStyles: Record<ChipVariant, string> = {
  pinned:  "text-accent",
  il2cpp:  "text-warning",
  mono:    "text-info",
};

const chipDotStyles: Record<ChipVariant, string> = {
  pinned:  "bg-accent",
  il2cpp:  "bg-warning",
  mono:    "bg-info",
};

/* ------------------------------------------------------------------ */
/* Icon + colour by iconTheme                                           */
/* ------------------------------------------------------------------ */
type IconTheme = RecentProject["iconTheme"];

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
/* Single tile                                                          */
/* ------------------------------------------------------------------ */
function ProjectTile({ project }: { project: RecentProject }) {
  const isPinned = project.chip?.variant === "pinned";

  return (
    <div
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
            <TileIcon theme={project.iconTheme} />
            {project.chip && (
              <span
                className={cn(
                  "font-mono text-[10px] flex items-center gap-1",
                  chipStyles[project.chip.variant],
                )}
              >
                <span
                  className={cn(
                    "w-[5px] h-[5px] rounded-full",
                    chipDotStyles[project.chip.variant],
                  )}
                />
                {project.chip.label}
              </span>
            )}
          </div>

          <div className="font-sans text-[14px] font-semibold text-primary leading-[18px] mb-1 tracking-tightish">
            {project.name}
          </div>
          <div className="font-mono text-[10px] text-tertiary truncate">
            {project.path}
          </div>
        </div>

        <div className="flex items-center justify-between">
          <span className="font-mono text-[10px] text-secondary bg-elevated border border-border-default rounded-[4px] px-1.5 py-[1px]">
            {project.unityVersion}
          </span>
          <div className="text-right">
            <div className="font-mono text-[10px] text-secondary">
              {project.assetCount.toLocaleString()} assets
            </div>
            <div className="font-mono text-[10px] text-tertiary">
              {project.openedLabel}
            </div>
          </div>
        </div>
      </div>
    </div>
  );
}

/* ------------------------------------------------------------------ */
/* Grid                                                                 */
/* ------------------------------------------------------------------ */
export function RecentProjectsGrid() {
  return (
    <section className="flex flex-col" style={{ width: "40%" }}>
      {/* Header row */}
      <div className="flex items-center justify-between mb-4">
        <div className="flex items-center gap-2">
          <span className="font-mono text-[10px] tracking-[0.12em] uppercase text-tertiary">
            Recent
          </span>
          <span className="font-mono text-[10px] text-secondary bg-elevated border border-border-default rounded-[4px] px-1.5 py-[1px]">
            {MOCK_RECENTS.length}
          </span>
        </div>
        <button className="text-secondary hover:text-primary text-[12px] font-medium flex items-center gap-1 transition-colors">
          <ArrowsClockwise size={12} />
          Refresh
        </button>
      </div>

      {/* 2×2 grid */}
      <div className="grid grid-cols-2 gap-3 mb-5">
        {MOCK_RECENTS.map((p) => (
          <ProjectTile key={p.id} project={p} />
        ))}
      </div>

      {/* Empty state row */}
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

      {/* Tip footer */}
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
    </section>
  );
}
