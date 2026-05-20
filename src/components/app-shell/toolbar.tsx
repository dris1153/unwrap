import {
  ArrowLeft,
  ArrowRight,
  ClockCounterClockwise,
  CaretDown,
  CaretRight,
  MagnifyingGlass,
  Export,
  Bell,
  GearSix,
  Folder,
} from "@phosphor-icons/react";
import { Kbd } from "../ui/kbd";
import type { AssetTree } from "../../lib/types";
import { useTreeStore } from "../../stores/use-tree-store";
import { buildPathLabel } from "../../features/project/tree/build-path-label";
import { usePaletteStore } from "../../features/command-palette/use-palette-store";

type ToolbarProps = {
  projectId?: string;
  tree?: AssetTree;
};

/**
 * 44px top toolbar — nav arrows, recents, breadcrumb, ⌘K search, bell/settings.
 * When projectId + tree are provided, breadcrumb reflects the selected node path.
 */
export function Toolbar({ projectId, tree }: ToolbarProps = {}) {
  const selectedId = useTreeStore(
    (s) => (projectId ? s.selectedId[projectId] : null) ?? null,
  );
  const openPalette = usePaletteStore((s) => s.openPalette);

  const breadcrumbParts: string[] = [];
  if (tree && selectedId) {
    const fullPath = buildPathLabel(tree, selectedId);
    breadcrumbParts.push(...fullPath.split("/"));
  }

  return (
    <div
      className="flex items-center bg-base border-b border-border-default px-3"
      style={{ height: 44, flexShrink: 0 }}
    >
      {/* Nav arrows */}
      <div className="flex items-center gap-0.5">
        <button className="w-7 h-7 rounded-[var(--radius-md)] hover:bg-overlay text-secondary hover:text-primary flex items-center justify-center transition-colors">
          <ArrowLeft size={14} />
        </button>
        <button className="w-7 h-7 rounded-[var(--radius-md)] hover:bg-overlay text-tertiary flex items-center justify-center transition-colors">
          <ArrowRight size={14} />
        </button>
      </div>

      <div className="w-px h-5 bg-border-default mx-3" />

      {/* Recents dropdown */}
      <button className="flex items-center gap-1.5 px-2.5 h-7 rounded-[var(--radius-md)] hover:bg-overlay text-secondary hover:text-primary text-[12px] font-medium transition-colors">
        <ClockCounterClockwise size={14} />
        Recents
        <CaretDown size={10} className="text-tertiary" />
      </button>

      {/* Breadcrumb trail */}
      <div className="flex items-center gap-1 ml-3 font-mono text-[11px] text-tertiary">
        {breadcrumbParts.length > 0 ? (
          <>
            <Folder size={12} className="flex-shrink-0" />
            {breadcrumbParts.map((part, i) => (
              <span key={i} className="flex items-center gap-1">
                {i > 0 && <CaretRight size={9} />}
                <span
                  className={
                    i === breadcrumbParts.length - 1
                      ? "text-primary"
                      : "hover:text-secondary cursor-pointer"
                  }
                >
                  {part}
                </span>
              </span>
            ))}
          </>
        ) : (
          <span className="hover:text-secondary cursor-pointer">Project</span>
        )}
      </div>

      {/* Spacer → search (centered) */}
      <div className="flex-1 flex justify-center px-8">
        <div className="flex items-center gap-2 bg-elevated border border-border-default hover:border-border-strong rounded-[var(--radius-md)] h-8 px-3 w-[480px] focus-within:border-accent focus-within:shadow-[0_0_0_2px_rgba(16,185,129,0.20)] transition-all">
          <MagnifyingGlass size={14} className="text-tertiary flex-shrink-0" />
          <input
            type="text"
            placeholder="Search assets, scripts, strings…"
            className="flex-1 bg-transparent text-[13px] text-primary placeholder:text-tertiary outline-none font-sans"
            readOnly
            onClick={() => openPalette()}
            onFocus={() => openPalette()}
            onKeyDown={(e) => {
              // Forward typed characters as initial query to the real palette input
              if (e.key.length === 1 && !e.ctrlKey && !e.metaKey) {
                openPalette(e.key);
              }
            }}
          />
          <Kbd>⌘K</Kbd>
        </div>
      </div>

      {/* Right cluster */}
      <div className="flex items-center gap-1">
        <button className="w-8 h-8 rounded-[var(--radius-md)] hover:bg-overlay text-secondary hover:text-primary flex items-center justify-center transition-colors">
          <Export size={16} />
        </button>
        <button className="w-8 h-8 rounded-[var(--radius-md)] hover:bg-overlay text-secondary hover:text-primary flex items-center justify-center relative transition-colors">
          <Bell size={16} />
          {/* notification dot */}
          <span className="absolute top-[6px] right-[7px] w-[6px] h-[6px] bg-accent rounded-full border border-base" />
        </button>
        <button className="w-8 h-8 rounded-[var(--radius-md)] hover:bg-overlay text-secondary hover:text-primary flex items-center justify-center transition-colors">
          <GearSix size={16} />
        </button>
      </div>
    </div>
  );
}
