import { useQuery } from "@tanstack/react-query";
import { Plus } from "@phosphor-icons/react";
import { ipc } from "../../../lib/ipc";
import { useTabStore } from "../../../stores/use-tab-store";

type FilmstripProps = {
  projectId: string;
  nodeId: string;
};

const MAX_VISIBLE = 8;

/**
 * 104px filmstrip showing sibling texture nodes for the current image asset.
 * Matches wireframe 02 lines 641-728.
 */
export function Filmstrip({ projectId, nodeId }: FilmstripProps) {
  const { data: tree } = useQuery({
    queryKey: ["tree", projectId],
    queryFn: () => ipc.tree(projectId),
    staleTime: 60_000,
  });

  const { tabs } = useTabStore();
  const setActive = useTabStore((s) => s.setActive);
  const openTab = useTabStore((s) => s.openTab);

  if (!tree) return null;

  const currentNode = tree.nodes[nodeId];
  if (!currentNode) return null;

  const parentId = currentNode.parent?.["0"];
  if (!parentId) return null;

  // Siblings = nodes with same parent that are image-type
  const siblings = Object.entries(tree.nodes)
    .filter(([id, n]) => n.parent?.["0"] === parentId && n.kind === "texture" && id !== nodeId)
    .map(([id, n]) => ({ id, node: n }));

  const visible = siblings.slice(0, MAX_VISIBLE);
  const overflow = siblings.length - MAX_VISIBLE;

  if (siblings.length === 0) return null;

  return (
    <div
      className="bg-surface border-t border-border-default flex-shrink-0"
      style={{ height: 104 }}
    >
      <div className="flex items-center justify-between px-4 pt-2 pb-1">
        <div className="flex items-center gap-2">
          <span className="font-mono text-[10px] tracking-[0.12em] uppercase text-tertiary">
            Related
          </span>
          <span className="font-mono text-[10px] text-secondary bg-elevated border border-border-default rounded-[4px] px-1.5 py-[1px]">
            {siblings.length + 1} sprites
          </span>
        </div>
        <div className="flex items-center gap-1 font-mono text-[10px] text-tertiary">
          <span>scroll · drag to pan</span>
        </div>
      </div>

      <div className="flex items-center gap-2 px-4 pb-2 overflow-x-auto scroll-thin">
        {/* Current file thumb (always first, always active) */}
        <FilmThumb
          label={currentNode.name.replace(/\.[^.]+$/, "")}
          isActive
          onClick={() => {
            const existingTab = tabs.find(
              (t) => t.projectId === projectId && t.nodeId === nodeId,
            );
            if (existingTab) setActive(existingTab.id);
          }}
        />

        {/* Sibling thumbs */}
        {visible.map(({ id, node }) => (
          <FilmThumb
            key={id}
            label={node.name.replace(/\.[^.]+$/, "")}
            isActive={false}
            onClick={() => openTab(projectId, node)}
          />
        ))}

        {/* Overflow badge */}
        {overflow > 0 && (
          <div className="flex flex-col items-center gap-1 cursor-pointer flex-shrink-0">
            <div className="w-[64px] h-[52px] rounded-[var(--radius-md)] border border-border-default border-dashed bg-base/60 flex items-center justify-center">
              <div className="text-center">
                <Plus size={14} className="text-tertiary mx-auto" />
                <div className="font-mono text-[9px] text-tertiary mt-0.5">+ {overflow}</div>
              </div>
            </div>
            <span className="font-mono text-[9px] text-tertiary">more</span>
          </div>
        )}
      </div>
    </div>
  );
}

function FilmThumb({
  label,
  isActive,
  onClick,
}: {
  label: string;
  isActive: boolean;
  onClick: () => void;
}) {
  return (
    <div
      role="button"
      tabIndex={0}
      onClick={onClick}
      onKeyDown={(e) => e.key === "Enter" && onClick()}
      className="flex flex-col items-center gap-1 cursor-pointer flex-shrink-0"
    >
      <div
        className={`w-[64px] h-[52px] rounded-[var(--radius-md)] border checker overflow-hidden relative ${
          isActive
            ? "border-accent/60 bg-elevated"
            : "border-border-default hover:border-border-strong bg-elevated"
        }`}
      >
        {isActive && (
          <div className="absolute inset-0 bg-accent-muted" />
        )}
      </div>
      <span
        className={`font-mono text-[9px] truncate w-[64px] text-center ${
          isActive ? "text-primary" : "text-tertiary"
        }`}
      >
        {label}
      </span>
    </div>
  );
}
