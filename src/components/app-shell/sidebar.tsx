import {
  Files,
  MagnifyingGlass,
  Graph,
  FileCode,
  Translate,
  BookmarkSimple,
  Question,
  FunnelSimple,
  ArrowsDownUp,
  DotsThree,
} from "@phosphor-icons/react";
import { cn } from "../../lib/cn";
import { useUiStore, type RailTab } from "../../stores/use-ui-store";
import { StatusDot } from "../ui/status-dot";

type RailButton = {
  id: RailTab;
  icon: React.ReactNode;
  title: string;
};

const RAIL_BUTTONS: RailButton[] = [
  { id: "files",    icon: <Files size={16} />,          title: "Files" },
  { id: "search",   icon: <MagnifyingGlass size={16} />, title: "Search" },
  { id: "xref",     icon: <Graph size={16} />,           title: "Cross References" },
  { id: "scripts",  icon: <FileCode size={16} />,        title: "Scripts" },
  { id: "strings",  icon: <Translate size={16} />,       title: "Strings" },
];

type SidebarProps = {
  /** Optional project id — when provided the tree panel renders via renderTree */
  projectId?: string;
  /** Render prop supplying the tree content (from ProjectRouteContent) */
  renderTree?: () => React.ReactNode;
};

/**
 * 280px sidebar = 44px icon rail + 236px tree panel.
 * Rail tab state stored in useUiStore.
 * When renderTree is provided, the tree body slot is replaced by its output.
 */
export function Sidebar({ renderTree }: SidebarProps = {}) {
  const { activeRail, setActiveRail, sidebarWidth } = useUiStore();

  return (
    <aside
      className="flex flex-shrink-0 border-r border-border-default bg-surface"
      style={{ width: sidebarWidth }}
    >
      {/* Rail (44px) */}
      <nav
        className="flex flex-col items-center border-r border-border-default py-2 gap-1"
        style={{ width: 44, flexShrink: 0 }}
      >
        {RAIL_BUTTONS.map((btn) => (
          <button
            key={btn.id}
            title={btn.title}
            onClick={() => setActiveRail(btn.id)}
            className={cn(
              "w-8 h-8 rounded-[var(--radius-md)] flex items-center justify-center relative",
              "transition-colors",
              activeRail === btn.id
                ? "bg-accent-muted text-accent"
                : "hover:bg-overlay text-secondary hover:text-primary",
            )}
          >
            {btn.icon}
          </button>
        ))}

        <div className="w-6 border-t border-border-default my-1" />

        <button
          title="Bookmarks"
          onClick={() => setActiveRail("bookmarks")}
          className={cn(
            "w-8 h-8 rounded-[var(--radius-md)] flex items-center justify-center",
            "transition-colors",
            activeRail === "bookmarks"
              ? "bg-accent-muted text-accent"
              : "hover:bg-overlay text-secondary hover:text-primary",
          )}
        >
          <BookmarkSimple size={16} />
        </button>

        <div className="flex-1" />

        <button
          title="Help"
          className="w-8 h-8 rounded-[var(--radius-md)] hover:bg-overlay text-secondary hover:text-primary flex items-center justify-center transition-colors"
        >
          <Question size={16} />
        </button>
      </nav>

      {/* Tree panel */}
      <div className="flex-1 flex flex-col min-w-0">
        {/* Header */}
        <div
          className="flex items-center justify-between px-3 border-b border-border-default"
          style={{ height: 36 }}
        >
          <div className="flex items-center gap-1.5">
            <span className="font-mono text-[10px] tracking-[0.12em] uppercase text-tertiary">
              Assets
            </span>
          </div>
          <div className="flex items-center gap-0.5">
            <button className="w-5 h-5 rounded-[4px] hover:bg-overlay text-tertiary hover:text-primary flex items-center justify-center">
              <FunnelSimple size={12} />
            </button>
            <button className="w-5 h-5 rounded-[4px] hover:bg-overlay text-tertiary hover:text-primary flex items-center justify-center">
              <ArrowsDownUp size={12} />
            </button>
            <button className="w-5 h-5 rounded-[4px] hover:bg-overlay text-tertiary hover:text-primary flex items-center justify-center">
              <DotsThree size={12} />
            </button>
          </div>
        </div>

        {/* Tree body — delegated to renderTree when a project is open;
            renderTree() owns the filter input + footer internally (AssetTree). */}
        {renderTree ? (
          renderTree()
        ) : (
          <>
            <div className="px-2 py-2 border-b border-border-default">
              <div className="flex items-center gap-2 bg-base border border-border-default rounded-[var(--radius-md)] h-7 px-2.5 hover:border-border-strong transition-colors">
                <MagnifyingGlass size={12} className="text-tertiary flex-shrink-0" />
                <input
                  type="text"
                  placeholder="Filter tree…"
                  className="flex-1 bg-transparent text-[12px] placeholder:text-tertiary outline-none font-mono"
                />
              </div>
            </div>
            <div className="flex-1 flex items-center justify-center">
              <span className="font-mono text-[12px] text-tertiary">No project open</span>
            </div>
            <div
              className="flex items-center justify-between px-3 border-t border-border-default font-mono text-[10px] text-tertiary"
              style={{ height: 24 }}
            >
              <span className="flex items-center gap-1.5">
                <StatusDot color="tertiary" />
                idle
              </span>
              <span>0 assets</span>
            </div>
          </>
        )}
      </div>
    </aside>
  );
}
